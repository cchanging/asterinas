// SPDX-License-Identifier: MPL-2.0

use alloc::format;
use core::sync::atomic::Ordering;

use aster_systree::{Error, Result, SysAttrSet, SysAttrSetBuilder, SysPerms, SysStr};
use ostd::mm::{VmReader, VmWriter};

use crate::{
    fs::cgroupfs::{
        controller::{CgroupSysNode, SubCtrlState},
        CgroupNode,
    },
    prelude::*,
    process::{process_table, Pid},
};

/// The basic controller in cgroup subsystem.
///
/// Each cgroup in the hierarchy enables the controller by default and cannot deactivate it.
/// The controller exposes the control interfaces for cgroup management operations.
pub struct CgroupController {
    attrs: SysAttrSet,
}

impl CgroupController {
    pub(super) fn new(is_root: bool) -> Self {
        let attrs = {
            let mut builder = SysAttrSetBuilder::new();
            if !is_root {
                builder.add(
                    SysStr::from("cgroup.events"),
                    SysPerms::DEFAULT_RO_ATTR_PERMS,
                );
                builder.add(
                    SysStr::from("cgroup.freeze"),
                    SysPerms::DEFAULT_RW_ATTR_PERMS,
                );
                builder.add(SysStr::from("cgroup.type"), SysPerms::DEFAULT_RW_ATTR_PERMS);
            }
            builder.add(
                SysStr::from("cgroup.controllers"),
                SysPerms::DEFAULT_RO_ATTR_PERMS,
            );
            builder.add(
                SysStr::from("cgroup.subtree_control"),
                SysPerms::DEFAULT_RW_ATTR_PERMS,
            );
            builder.add(
                SysStr::from("cgroup.max.depth"),
                SysPerms::DEFAULT_RW_ATTR_PERMS,
            );
            builder.add(
                SysStr::from("cgroup.procs"),
                SysPerms::DEFAULT_RW_ATTR_PERMS,
            );
            builder.add(
                SysStr::from("cgroup.threads"),
                SysPerms::DEFAULT_RW_ATTR_PERMS,
            );
            builder.build().expect("Failed to build attribute set")
        };

        Self { attrs }
    }
}

impl super::SubControl for CgroupController {
    fn attr_set(&self) -> &SysAttrSet {
        &self.attrs
    }

    fn read_attr(
        &self,
        name: &str,
        writer: &mut VmWriter,
        cgroup_node: &dyn CgroupSysNode,
    ) -> Result<usize> {
        if !self.attrs.contains(name) {
            return Err(Error::NotFound);
        }

        match name {
            "cgroup.controllers" => {
                let context = cgroup_node.cgroup_parent().map_or_else(
                    || SubCtrlState::all().show(),
                    |parent| parent.controller().show_state(),
                );

                writer
                    .write_fallible(&mut VmReader::from((context + "\n").as_bytes()))
                    .map_err(|_| Error::AttributeError)
            }
            "cgroup.subtree_control" => {
                let context = cgroup_node.controller().show_state();
                writer
                    .write_fallible(&mut VmReader::from((context + "\n").as_bytes()))
                    .map_err(|_| Error::AttributeError)
            }
            "cgroup.procs" => {
                let context =
                    if let Some(cgroup_node) = cgroup_node.as_any().downcast_ref::<CgroupNode>() {
                        cgroup_node.read_procs()
                    } else {
                        let process_table = process_table::process_table_mut();
                        process_table
                            .iter()
                            .filter_map(|process| {
                                if process.cgroup().is_none() {
                                    Some(process.pid().to_string())
                                } else {
                                    None
                                }
                            })
                            .collect::<Vec<String>>()
                            .join("\n")
                    };

                writer
                    .write_fallible(&mut VmReader::from((context + "\n").as_bytes()))
                    .map_err(|_| Error::AttributeError)
            }
            "cgroup.events" => {
                let cgroup_node = cgroup_node.as_any().downcast_ref::<CgroupNode>().unwrap();
                let res = if cgroup_node.populated_count().load(Ordering::Acquire) > 0 {
                    1
                } else {
                    0
                };
                // Currently we have not enabled the "frozen" attribute
                // so the "frozen" field is always zero.
                let output = format!(
                    "populated {}\nfrozen {}\n",
                    res,
                    cgroup_node.freeze.load(Ordering::Acquire) as u32
                );
                writer
                    .write_fallible(&mut VmReader::from(output.as_bytes()))
                    .map_err(|_| Error::AttributeError)
            }
            "cgroup.type" => {
                let output = "domain\n";
                writer
                    .write_fallible(&mut VmReader::from(output.as_bytes()))
                    .map_err(|_| Error::AttributeError)
            }
            "cgroup.freeze" => {
                let cgroup_node = cgroup_node.as_any().downcast_ref::<CgroupNode>().unwrap();
                let frozen = if cgroup_node.freeze.load(Ordering::Acquire) {
                    1
                } else {
                    0
                };
                let output = format!("{}\n", frozen);
                writer
                    .write_fallible(&mut VmReader::from(output.as_bytes()))
                    .map_err(|_| Error::AttributeError)
            }
            _ => {
                // TODO: Activate support for reading other attributes.
                Err(Error::AttributeError)
            }
        }
    }

    fn write_attr(
        &self,
        name: &str,
        reader: &mut VmReader,
        cgroup_node: &dyn CgroupSysNode,
    ) -> Result<usize> {
        match name {
            "cgroup.procs" => {
                let (context, context_len) = super::util::read_context_from_reader(reader)?;
                let (pid, pid_len) = (
                    super::util::parse_context_to_val::<Pid>(context)?,
                    context_len,
                );

                // According to "no internal processes" rule of cgroupv2, if a non-root
                // cgroup node has activated some sub-controls, it cannot bind any process.
                //
                // Ref: https://man7.org/linux/man-pages/man7/cgroups.7.html
                if !cgroup_node.is_root()
                    && !cgroup_node.controller().sub_ctrl_state.lock().is_empty()
                {
                    return Err(Error::ResourceUnavailable);
                }

                let process = if pid == 0 {
                    current!()
                } else {
                    process_table::get_process(pid).ok_or(Error::AttributeError)?
                };

                if let Some(cgroup_node) = cgroup_node.as_any().downcast_ref::<CgroupNode>() {
                    cgroup_node.move_process(process);
                } else {
                    let rcu_old_cgroup = process.cgroup();
                    let old_cgroup = rcu_old_cgroup.get();
                    if let Some(old_cgroup) = old_cgroup {
                        old_cgroup.remove_process(&process);
                    }
                }

                Ok(pid_len)
            }
            "cgroup.subtree_control" => {
                let (actions, len) = read_subtree_control_from_reader(reader)?;

                if let Some(cgroup_node) = cgroup_node.as_any().downcast_ref::<CgroupNode>() {
                    // According to "no internal processes" rule of cgroupv2, if a non-root
                    // cgroup node has bound processes, it cannot activate any sub-control.
                    //
                    // Ref: https://man7.org/linux/man-pages/man7/cgroups.7.html
                    if cgroup_node.have_processes() {
                        return Err(Error::ResourceUnavailable);
                    }
                }

                let parent_node = cgroup_node.cgroup_parent();
                let parent_ctrls_state = parent_node
                    .as_ref()
                    .map(|parent_node| parent_node.controller().sub_ctrl_state.lock());
                for action in actions {
                    match action {
                        SubControlAction::Activate(name) => {
                            // A cgroup can activate the sub-control only if this
                            // sub-control has been activated in its parent cgroup.
                            let can_activate =
                                parent_ctrls_state
                                    .as_ref()
                                    .is_none_or(|parent_ctrls_state| {
                                        parent_ctrls_state.is_enabled(&name).unwrap()
                                    });

                            if !can_activate {
                                return Err(Error::NotFound);
                            }

                            cgroup_node.controller().activate(&name, cgroup_node)?;
                        }
                        SubControlAction::Deactivate(name) => {
                            let mut can_deactivate = true;
                            // If any child node has activated this sub-control,
                            // the deactivation operation will be rejected.
                            cgroup_node.visit_children_with(0, &mut |child| {
                                let cgroup_child =
                                    child.as_any().downcast_ref::<CgroupNode>().unwrap();
                                if cgroup_child
                                    .controller()
                                    .sub_ctrl_state
                                    .lock()
                                    .is_enabled(&name)
                                    .unwrap()
                                {
                                    can_deactivate = false;
                                    None
                                } else {
                                    Some(())
                                }
                            });

                            if !can_deactivate {
                                return Err(Error::InvalidOperation);
                            }

                            cgroup_node.controller().deactivate(&name, cgroup_node)?;
                        }
                    }
                }

                Ok(len)
            }
            "cgroup.type" => {
                let (context, context_len) = super::util::read_context_from_reader(reader)?;
                println!("Warning! Ignoring write to cgroup.type: {}", context.trim());
                Ok(context_len)
            }
            "cgroup.freeze" => {
                let (context, context_len) = super::util::read_context_from_reader(reader)?;
                let value = super::util::parse_context_to_val::<u32>(context)?;

                if let Some(cgroup_node) = cgroup_node.as_any().downcast_ref::<CgroupNode>() {
                    match value {
                        0 => {
                            cgroup_node.clear_freeze();
                        }
                        1 => {
                            cgroup_node.set_freeze();
                        }
                        _ => return Err(Error::InvalidOperation),
                    }
                }
                Ok(context_len)
            }
            _ => {
                // TODO: Activate support for reading other attributes.
                Err(Error::AttributeError)
            }
        }
    }
}

/// Reads the actions for sub-control from the given reader.
///
/// Returns a tuple containing vector of actions and the number of bytes read.
fn read_subtree_control_from_reader(
    reader: &mut VmReader,
) -> Result<(Vec<SubControlAction>, usize)> {
    let (context, len) = super::util::read_context_from_reader(reader)?;

    let mut actions_vec = Vec::new();
    let actions = context.split_whitespace();
    for action in actions {
        if action.len() < 2 {
            return Err(Error::AttributeError);
        }

        let action = match action.chars().next() {
            Some('+') => {
                let name = action[1..].to_string();
                if SubCtrlState::control_bit(&name).is_none() {
                    return Err(Error::InvalidOperation);
                }

                SubControlAction::Activate(name)
            }
            Some('-') => {
                let name = action[1..].to_string();
                if SubCtrlState::control_bit(&name).is_none() {
                    return Err(Error::InvalidOperation);
                }

                SubControlAction::Deactivate(name)
            }
            _ => return Err(Error::AttributeError),
        };
        actions_vec.push(action);
    }

    Ok((actions_vec, len))
}

enum SubControlAction {
    Activate(String),
    Deactivate(String),
}
