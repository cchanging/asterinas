// SPDX-License-Identifier: MPL-2.0

use core::sync::atomic::{AtomicUsize, Ordering};

use aster_systree::{Error, Result, SysAttrSet, SysAttrSetBuilder, SysPerms, SysStr};
use ostd::mm::{VmReader, VmWriter};

use crate::{fs::cgroupfs::controller::CgroupSysNode, util::MultiWrite};

/// The controller responsible for PID in the cgroup subsystem.
///
/// This controller will only provide interfaces in non-root cgroup node.
pub struct PidsController {
    max_pid: AtomicUsize,
    attrs: SysAttrSet,
}

impl PidsController {
    pub(super) fn new() -> Self {
        let mut builder = SysAttrSetBuilder::new();

        builder.add(SysStr::from("pids.max"), SysPerms::DEFAULT_RW_ATTR_PERMS);

        let attrs = builder.build().expect("Failed to build attribute set");
        Self {
            max_pid: AtomicUsize::new(usize::MAX),
            attrs,
        }
    }
}

impl super::SubControl for PidsController {
    fn attr_set(&self) -> &SysAttrSet {
        &self.attrs
    }

    fn read_attr(
        &self,
        name: &str,
        writer: &mut VmWriter,
        _cgroup_node: &dyn CgroupSysNode,
    ) -> Result<usize> {
        match name {
            "pid.max" => {
                let max_pid = self.max_pid.load(Ordering::Relaxed);
                let max_pid_str = alloc::format!("{}", max_pid);
                let context = if max_pid == usize::MAX {
                    "max"
                } else {
                    max_pid_str.as_str()
                };

                let len = writer
                    .write(&mut VmReader::from(context.as_bytes()))
                    .map_err(|_| Error::AttributeError)?;
                Ok(len)
            }
            _ => Err(Error::AttributeError),
        }
    }

    fn write_attr(
        &self,
        name: &str,
        reader: &mut VmReader,
        _cgroup_node: &dyn CgroupSysNode,
    ) -> Result<usize> {
        match name {
            "pid.max" => {
                let (context, len) = super::util::read_context_from_reader(reader)?;
                let value = if context.trim() == "max" {
                    usize::MAX
                } else {
                    super::util::parse_context_to_val::<usize>(context)?
                };

                self.max_pid.store(value, Ordering::Relaxed);

                Ok(len)
            }
            _ => Err(Error::AttributeError),
        }
    }
}
