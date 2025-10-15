// SPDX-License-Identifier: MPL-2.0

use alloc::{
    borrow::Cow,
    string::ToString,
    sync::{Arc, Weak},
};
use core::{
    fmt::Debug,
    sync::atomic::{AtomicUsize, Ordering},
};

use aster_systree::{
    inherit_sys_branch_node, AttrLessBranchNodeFields, Error, Result, SysAttr, SysAttrSet,
    SysBranchNode, SysObj, SysPerms, SysStr,
};
use inherit_methods_macro::inherit_methods;
use ostd::mm::{VmReader, VmWriter};
use spin::Once;

use crate::{
    fs::cgroupfs::controller::{CgroupSysNode, Controller, SubCtrlState},
    prelude::*,
    process::{process_table, Pid, Process},
};

/// A type that provides exclusive, synchronized access to modify cgroup membership.
///
/// This struct encapsulates the logic for moving processes between cgroups.
/// By calling `CgroupMembership::lock()`, a thread can attempt to acquire a lock
/// on the global instance. Upon success, it returns a guard that provides mutable
/// access, allowing for safe cgroup membership modifications.
///
/// # Usage
///
/// ```rust,ignore
/// // Acquire the lock.
/// let membership = CgroupMembership::lock();
///
/// // Move a process to a new cgroup node.
/// membership.move_process_to_node(process, &new_cgroup);
///
/// // The lock is automatically released when `membership` is dropped.
/// ```
pub struct CgroupMembership {
    _private: (),
}

impl CgroupMembership {
    /// Acquires the lock on the global instance.
    ///
    /// Returns a guard that provides mutable access to modify cgroup membership.
    pub fn lock() -> MutexGuard<'static, Self> {
        static CGROUP_MEMBERSHIP: Mutex<CgroupMembership> =
            Mutex::new(CgroupMembership { _private: () });

        CGROUP_MEMBERSHIP.lock()
    }

    /// Moves a process to the new cgroup node.
    ///
    /// A process can only belong to one cgroup at a time.
    /// When moved to a new cgroup, it's automatically removed from the
    /// previous one.
    pub fn move_process_to_node(&mut self, process: Arc<Process>, new_cgroup: &CgroupNode) {
        if let Some(old_cgroup) = process.cgroup().get() {
            // Fast path: If the process is already in this cgroup, do nothing.
            if new_cgroup.id() == old_cgroup.id() {
                return;
            }

            let mut old_cgroup_process_set = old_cgroup.processes.lock();
            if old_cgroup_process_set.remove(&process.pid()).is_some()
                && old_cgroup_process_set.is_empty()
            {
                let old_count = old_cgroup.populated_count.fetch_sub(1, Ordering::Relaxed);
                if old_count == 1 {
                    old_cgroup.propagate_sub_populated();
                }
            }
        };

        let mut current_process_set = new_cgroup.processes.lock();
        if current_process_set.is_empty() {
            let old_count = new_cgroup.populated_count.fetch_add(1, Ordering::Relaxed);
            if old_count == 0 {
                new_cgroup.propagate_add_populated();
            }
        }

        process.set_cgroup(Some(new_cgroup.fields.weak_self().upgrade().unwrap()));
        current_process_set.insert(process.pid(), Arc::downgrade(&process));
    }

    /// Moves a process to the root cgroup.
    pub fn move_process_to_root(&mut self, process: &Process) {
        let process_cgroup = process.cgroup();
        let Some(old_cgroup) = process_cgroup.get() else {
            return;
        };

        let mut processes = old_cgroup.processes.lock();
        if processes.remove(&process.pid()).is_none() {
            return;
        }

        process.set_cgroup(None);

        if processes.is_empty() {
            let old_count = old_cgroup.populated_count.fetch_sub(1, Ordering::Relaxed);
            if old_count == 1 {
                old_cgroup.propagate_sub_populated();
            }
        }
    }
}

/// The root of a cgroup hierarchy, serving as the entry point to
/// the entire cgroup control system.
///
/// The cgroup system provides v2 unified hierarchy, and is also used as a root
/// node in the cgroup systree.
pub struct CgroupSystem {
    fields: AttrLessBranchNodeFields<CgroupNode, Self>,
    controller: Controller,
}

impl Debug for CgroupSystem {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("CgroupSystem")
            .field("fields", &self.fields)
            .finish()
    }
}

/// A control group node in the cgroup systree.
///
/// Each node can bind a group of processes together for purpose of resource
/// management. Except for the root node, all nodes in the cgroup tree are of
/// this type.
pub struct CgroupNode {
    fields: AttrLessBranchNodeFields<CgroupNode, Self>,
    /// The controller of this cgroup node.
    controller: Controller,
    /// Processes bound to this node.
    processes: Mutex<BTreeMap<Pid, Weak<Process>>>,
    /// The depth of the node in the cgroupfs [`SysTree`], where the child of
    /// the root node has a depth of 1.
    depth: usize,
    /// Tracks the "populated" status of this node and its direct children.
    ///
    /// The count is the sum of:
    /// - The number of its direct children that are populated.
    /// - A value of 1 if this node itself contains processes.
    ///
    /// "populated": A node is considered populated if it has bound processes
    /// either on itself or in any of its descendant nodes. Consequently,
    /// a count > 0 indicates that this node is populated.
    populated_count: AtomicUsize,
}

impl Debug for CgroupNode {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("CgroupNormalNode")
            .field("fields", &self.fields)
            .finish()
    }
}

#[inherit_methods(from = "self.fields")]
impl CgroupSystem {
    /// Adds a child node.
    fn add_child(&self, new_child: Arc<CgroupNode>) -> Result<()>;
}

#[inherit_methods(from = "self.fields")]
impl CgroupNode {
    /// Adds a child node.
    fn add_child(&self, new_child: Arc<CgroupNode>) -> Result<()>;
}

impl CgroupSystem {
    /// Returns the `CgroupSystem` singleton.
    pub fn singleton() -> &'static Arc<CgroupSystem> {
        static SINGLETON: Once<Arc<CgroupSystem>> = Once::new();

        SINGLETON.call_once(Self::new)
    }

    fn new() -> Arc<Self> {
        let name = SysStr::from("cgroup");

        Arc::new_cyclic(|weak_self| {
            let fields = AttrLessBranchNodeFields::new(name, weak_self.clone());
            CgroupSystem {
                fields,
                controller: Controller::new(SubCtrlState::all(), true),
            }
        })
    }
}

impl CgroupSysNode for CgroupSystem {
    fn controller(&self) -> &Controller {
        &self.controller
    }
}

impl CgroupNode {
    pub(super) fn new(name: SysStr, depth: usize, sub_ctrl_state: SubCtrlState) -> Arc<Self> {
        Arc::new_cyclic(|weak_self| {
            let fields = AttrLessBranchNodeFields::new(name, weak_self.clone());
            CgroupNode {
                fields,
                controller: Controller::new(sub_ctrl_state, false),
                processes: Mutex::new(BTreeMap::new()),
                depth,
                populated_count: AtomicUsize::new(0),
            }
        })
    }
}

impl CgroupSysNode for CgroupNode {
    fn controller(&self) -> &Controller {
        &self.controller
    }
}

// For process management
impl CgroupNode {
    fn propagate_add_populated(&self) {
        if self.depth <= 1 {
            return;
        }

        let mut current_parent = Arc::downcast::<CgroupNode>(self.parent().unwrap()).unwrap();
        loop {
            let old_count = current_parent
                .populated_count
                .fetch_add(1, Ordering::AcqRel);
            if old_count > 0 {
                break;
            }

            if current_parent.depth == 1 {
                break;
            }

            current_parent = Arc::downcast::<CgroupNode>(current_parent.parent().unwrap()).unwrap();
        }
    }

    fn propagate_sub_populated(&self) {
        if self.depth <= 1 {
            return;
        }

        let mut current_parent = Arc::downcast::<CgroupNode>(self.parent().unwrap()).unwrap();
        loop {
            let old_count = current_parent
                .populated_count
                .fetch_sub(1, Ordering::Relaxed);
            if old_count != 1 {
                break;
            }

            if current_parent.depth == 1 {
                break;
            }

            current_parent = Arc::downcast::<CgroupNode>(current_parent.parent().unwrap()).unwrap();
        }
    }

    /// Attempts to run the provided closure if this cgroup node is empty.
    ///
    /// A cgroup node is considered empty if it has no child nodes and no
    /// processes bound to it.
    pub(super) fn try_run_if_empty<F>(&self, f: F) -> crate::Result<()>
    where
        F: FnOnce() -> crate::Result<()>,
    {
        let children = self.fields.children_ref().read();
        if !children.is_empty() {
            return_errno_with_message!(
                Errno::ENOTEMPTY,
                "only an empty cgroup hierarchy can be removed"
            );
        }

        let processes = self.processes.lock();
        if !processes.is_empty() {
            return_errno_with_message!(Errno::EBUSY, "the cgroup hierarchy still has processes");
        }

        f()
    }

    /// Whether this cgroup node has any processes bound to it.
    pub fn have_processes(&self) -> bool {
        !self.processes.lock().is_empty()
    }

    /// Reads the PID of the processes bound to this cgroup node.
    pub(super) fn read_procs(&self) -> String {
        self.processes
            .lock()
            .keys()
            .map(|pid| pid.to_string())
            .collect::<Vec<String>>()
            .join("\n")
    }

    pub(super) fn populated_count(&self) -> &AtomicUsize {
        &self.populated_count
    }
}

inherit_sys_branch_node!(CgroupSystem, fields, {
    fn is_root(&self) -> bool {
        true
    }

    fn init_parent(&self, _parent: Weak<dyn SysBranchNode>) {
        // This method should be a no-op for `RootNode`.
    }

    fn attr(&self, name: &str) -> Option<SysAttr> {
        self.controller.attr(name)
    }

    fn node_attrs(&self) -> Cow<SysAttrSet> {
        Cow::Owned(self.controller.node_attrs())
    }

    fn read_attr_at(&self, name: &str, offset: usize, writer: &mut VmWriter) -> Result<usize> {
        self.controller.read_attr_at(name, offset, writer, self)
    }

    fn write_attr(&self, name: &str, reader: &mut VmReader) -> Result<usize> {
        self.controller.write_attr(name, reader, self)
    }

    fn perms(&self) -> SysPerms {
        SysPerms::DEFAULT_RW_PERMS
    }

    fn create_child(&self, name: &str) -> Result<Arc<dyn SysObj>> {
        let sub_ctrl_state = self.controller().sub_ctrl_state();
        let new_child = CgroupNode::new(name.to_string().into(), 1, *sub_ctrl_state);
        self.add_child(new_child.clone())?;
        Ok(new_child)
    }
});

inherit_sys_branch_node!(CgroupNode, fields, {
    fn attr(&self, name: &str) -> Option<SysAttr> {
        self.controller.attr(name)
    }

    fn node_attrs(&self) -> Cow<SysAttrSet> {
        Cow::Owned(self.controller.node_attrs())
    }

    fn read_attr_at(&self, name: &str, offset: usize, writer: &mut VmWriter) -> Result<usize> {
        self.controller.read_attr_at(name, offset, writer, self)
    }

    fn write_attr(&self, name: &str, reader: &mut VmReader) -> Result<usize> {
        self.controller.write_attr(name, reader, self)
    }

    fn perms(&self) -> SysPerms {
        SysPerms::DEFAULT_RW_PERMS
    }

    fn create_child(&self, name: &str) -> Result<Arc<dyn SysObj>> {
        let sub_ctrl_state = self.controller().sub_ctrl_state();
        let new_child = CgroupNode::new(name.to_string().into(), self.depth + 1, *sub_ctrl_state);
        self.add_child(new_child.clone())?;
        Ok(new_child)
    }
});

/// A helper function to safely perform an operation on a process's cgroup.
///
/// The given `pid` means the PID of the target process. A PID of 0 refers to the
/// current process.
///
/// Returns `Error::InvalidOperation` if the PID is not found or if the target
/// process is a zombie.
pub(super) fn with_process_cgroup_locked<F>(pid: Pid, op: F) -> Result<()>
where
    F: FnOnce(Arc<Process>, &mut CgroupMembership) -> Result<()>,
{
    let process = if pid == 0 {
        current!()
    } else {
        process_table::get_process(pid).ok_or(Error::InvalidOperation)?
    };

    let mut cgroup_guard = CgroupMembership::lock();
    if process.status().is_zombie() {
        return Err(Error::InvalidOperation);
    }

    op(process, &mut cgroup_guard)
}
