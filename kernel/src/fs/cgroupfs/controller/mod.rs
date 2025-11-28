// SPDX-License-Identifier: MPL-2.0

use alloc::{string::String, sync::Arc, vec::Vec};

use aster_systree::{Error, Result, SysAttrSet, SysAttrSetBuilder};
use bitflags::bitflags;
use ostd::{
    mm::{VmReader, VmWriter},
    sync::{Mutex, MutexGuard, RcuOption},
    task::{atomic_mode::AsAtomicModeGuard, disable_preempt},
};

use crate::fs::cgroupfs::{
    controller::{cpuset::CpuSetController, memory::MemoryController, pids::PidsController},
    systree_node::CgroupSysNode,
    CgroupNode,
};

mod cpuset;
mod memory;
mod pids;

/// A trait to abstract all individual cgroup controllers.
trait SubControl {
    fn attr_set(&self) -> &SysAttrSet;

    fn read_attr_at(
        &self,
        name: &str,
        offset: usize,
        writer: &mut VmWriter,
        cgroup_node: &dyn CgroupSysNode,
    ) -> Result<usize>;

    fn write_attr(
        &self,
        name: &str,
        reader: &mut VmReader,
        cgroup_node: &dyn CgroupSysNode,
    ) -> Result<usize>;
}

/// The type of sub-controller in the cgroup subsystem.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ControllerType {
    Memory,
    CpuSet,
    Pids,
}

impl ControllerType {
    const ALL: [Self; 3] = [Self::Memory, Self::CpuSet, Self::Pids];
}

impl TryFrom<&str> for ControllerType {
    type Error = aster_systree::Error;

    fn try_from(value: &str) -> Result<Self> {
        match value {
            "memory" => Ok(ControllerType::Memory),
            "cpuset" => Ok(ControllerType::CpuSet),
            "pids" => Ok(ControllerType::Pids),
            _ => Err(Error::NotFound),
        }
    }
}

bitflags! {
    /// Bitflags representing active/deactive sub-control state.
    pub(super) struct SubCtrlState: u8 {
        const MEMORY_CTRLS = 1 << 0;
        const CPUSET_CTRLS = 1 << 1;
        const PIDS_CTRLS = 1 << 2;
    }
}

impl SubCtrlState {
    pub(super) fn control_bit(ctrl_type: ControllerType) -> Self {
        match ctrl_type {
            ControllerType::Memory => Self::MEMORY_CTRLS,
            ControllerType::CpuSet => Self::CPUSET_CTRLS,
            ControllerType::Pids => Self::PIDS_CTRLS,
        }
    }

    /// Checks if a sub-control is active in the current state.
    ///
    /// If the given name does not represent a supported controller,
    /// returns `None`.
    pub(super) fn is_active(&self, ctrl_type: ControllerType) -> bool {
        self.contains(Self::control_bit(ctrl_type))
    }

    fn activate(&mut self, ctrl_type: ControllerType) {
        *self |= Self::control_bit(ctrl_type)
    }

    fn deactivate(&mut self, ctrl_type: ControllerType) {
        *self -= Self::control_bit(ctrl_type)
    }

    pub(super) fn show(&self) -> String {
        let mut controllers = Vec::new();

        if self.contains(Self::MEMORY_CTRLS) {
            controllers.push("memory");
        }
        if self.contains(Self::CPUSET_CTRLS) {
            controllers.push("cpuset");
        }
        if self.contains(Self::PIDS_CTRLS) {
            controllers.push("pids");
        }

        controllers.join(" ")
    }

    /// Returns an iterator over the active controller types.
    pub(super) fn iter_active(&self) -> impl Iterator<Item = ControllerType> + '_ {
        ControllerType::ALL
            .into_iter()
            .filter(|&ctrl_type| self.is_active(ctrl_type))
    }
}

/// The controller for a single cgroup.
///
/// This struct can manage the activation state of each sub-control, and dispatches read/write
/// operations to the appropriate sub-controllers.
///
/// The following is an explanation of the activation for sub-controls and controllers.
/// When a cgroup activates a specific sub-control (e.g., memory, io), it means this control
/// capability is being delegated to its children. Consequently, the corresponding controller
/// within the child nodes will be activated.
///
/// The root node serves as the origin for all these control capabilities, so the controllers
/// it possesses are always active. For any other node, only if its parent node first enables
/// a sub-control, its corresponding controller will be activated.
///
/// Among all nodes, the fundamental cgroup controller is always active.
pub(super) struct Controller {
    sub_ctrl_state: Mutex<SubCtrlState>,

    memory: RcuOption<Arc<MemoryController>>,
    cpuset: RcuOption<Arc<CpuSetController>>,
    pids: RcuOption<Arc<PidsController>>,
}

impl Controller {
    /// Creates a new controller manager for a cgroup.
    pub(super) fn new(ctrl_state: SubCtrlState, is_root: bool) -> Self {
        let memory_controller = MemoryController::new(ctrl_state, is_root);
        let cpuset_controller = CpuSetController::new(ctrl_state, is_root);
        let pids_controller = PidsController::new(ctrl_state, is_root);

        Self {
            sub_ctrl_state: Mutex::new(SubCtrlState::empty()),
            memory: RcuOption::new(memory_controller),
            cpuset: RcuOption::new(cpuset_controller),
            pids: RcuOption::new(pids_controller),
        }
    }

    pub(super) fn init_attr_set(builder: &mut SysAttrSetBuilder, is_root: bool, is_active: bool) {
        MemoryController::init_attr_set(builder, is_root, is_active);
        CpuSetController::init_attr_set(builder, is_root);
        PidsController::init_attr_set(builder, is_root);
    }

    pub(super) fn lock(&self) -> LockedController {
        LockedController {
            sub_ctrl_state: self.sub_ctrl_state.lock(),
            controller: self,
        }
    }

    /// Returns a string representation of the current `subtree_control` state.
    pub(super) fn show_state(&self) -> String {
        self.sub_ctrl_state.lock().show()
    }

    fn obtain_sub_controller_with<'a, G>(
        &'a self,
        ctrl_type: ControllerType,
        guard: &'a G,
    ) -> Option<&'a dyn SubControl>
    where
        G: AsAtomicModeGuard + ?Sized,
    {
        match ctrl_type {
            ControllerType::Memory => self
                .memory
                .read_with(guard)
                .map(|inner| inner.deref_target() as _),
            ControllerType::CpuSet => self
                .cpuset
                .read_with(guard)
                .map(|inner| inner.deref_target() as _),
            ControllerType::Pids => self
                .pids
                .read_with(guard)
                .map(|inner| inner.deref_target() as _),
        }
    }

    /// Whether the attribute with the given name is absent in this controller.
    pub(super) fn is_attr_absent(&self, name: &str) -> bool {
        let Some((subsys, _)) = name.split_once('.') else {
            return false;
        };
        let Ok(ctrl_type) = ControllerType::try_from(subsys) else {
            return false;
        };

        let guard = disable_preempt();
        let Some(sub_controller) = self.obtain_sub_controller_with(ctrl_type, &guard) else {
            // If the sub-controller is not active, all its attributes are considered absent.
            return true;
        };

        sub_controller.attr_set().get(name).is_none()
    }

    pub(super) fn read_attr_at(
        &self,
        name: &str,
        offset: usize,
        writer: &mut VmWriter,
        cgroup_node: &dyn CgroupSysNode,
    ) -> Result<usize> {
        let Some((subsys, _)) = name.split_once('.') else {
            return Err(Error::NotFound);
        };
        let ctrl_type = ControllerType::try_from(subsys)?;

        let guard = disable_preempt();
        let Some(sub_controller) = self.obtain_sub_controller_with(ctrl_type, &guard) else {
            return Err(Error::NotFound);
        };

        sub_controller.read_attr_at(name, offset, writer, cgroup_node)
    }
}

/// A locked controller for a cgroup.
///
/// Holding this lock indicates exclusive access to modify the sub-control state.
pub(super) struct LockedController<'a> {
    sub_ctrl_state: MutexGuard<'a, SubCtrlState>,
    controller: &'a Controller,
}

impl LockedController<'_> {
    /// Activates a sub-control with given name.
    pub(super) fn activate(
        &mut self,
        ctrl_type: ControllerType,
        current_node: &dyn CgroupSysNode,
        parent_controller: Option<&LockedController>,
    ) -> Result<()> {
        if self.sub_ctrl_state.is_active(ctrl_type) {
            return Ok(());
        }

        // A cgroup can activate the sub-control only if this
        // sub-control has been activated in its parent cgroup.
        if parent_controller
            .is_some_and(|controller| !controller.sub_ctrl_state.is_active(ctrl_type))
        {
            return Err(Error::NotFound);
        }

        self.sub_ctrl_state.activate(ctrl_type);
        self.updata_sub_controller(ctrl_type, current_node);

        Ok(())
    }

    /// Deactivates a sub-control with given name.
    pub(super) fn deactivate(
        &mut self,
        ctrl_type: ControllerType,
        current_node: &dyn CgroupSysNode,
    ) -> Result<()> {
        if !self.sub_ctrl_state.is_active(ctrl_type) {
            return Ok(());
        }

        // If any child node has activated this sub-control,
        // the deactivation operation will be rejected.
        let mut can_deactivate = true;
        // This is race-free because if a child wants to activate a sub-controller, it should first
        // acquire the lock of the parent controller, which is held here.
        current_node.visit_children_with(0, &mut |child| {
            let cgroup_child = child.as_any().downcast_ref::<CgroupNode>().unwrap();
            let child_controller = cgroup_child.controller().lock();
            if child_controller.sub_ctrl_state().is_active(ctrl_type) {
                can_deactivate = false;
                None
            } else {
                Some(())
            }
        });
        if !can_deactivate {
            return Err(Error::InvalidOperation);
        }

        self.sub_ctrl_state.deactivate(ctrl_type);
        self.updata_sub_controller(ctrl_type, current_node);

        Ok(())
    }

    fn updata_sub_controller(&self, ctrl_type: ControllerType, current_node: &dyn CgroupSysNode) {
        current_node.visit_children_with(0, &mut |node| {
            let cgroup_node = node.as_any().downcast_ref::<CgroupNode>().unwrap();
            match ctrl_type {
                ControllerType::Memory => {
                    let new_controller = MemoryController::new(*self.sub_ctrl_state, false);
                    cgroup_node.controller().memory.update(new_controller);
                }
                ControllerType::CpuSet => {
                    let new_controller = CpuSetController::new(*self.sub_ctrl_state, false);
                    cgroup_node.controller().cpuset.update(new_controller);
                }
                ControllerType::Pids => {
                    let new_controller = PidsController::new(*self.sub_ctrl_state, false);
                    cgroup_node.controller().pids.update(new_controller);
                }
            }

            Some(())
        });
    }

    pub(super) fn write_attr(
        &self,
        name: &str,
        reader: &mut VmReader,
        cgroup_node: &dyn CgroupSysNode,
    ) -> Result<usize> {
        let Some((subsys, _)) = name.split_once('.') else {
            return Err(Error::NotFound);
        };
        let ctrl_type = ControllerType::try_from(subsys)?;

        let guard = disable_preempt();
        let Some(sub_controller) = self
            .controller
            .obtain_sub_controller_with(ctrl_type, &guard)
        else {
            return Err(Error::NotFound);
        };

        sub_controller.write_attr(name, reader, cgroup_node)
    }

    pub(super) fn sub_ctrl_state(&self) -> SubCtrlState {
        *self.sub_ctrl_state
    }
}
