// SPDX-License-Identifier: MPL-2.0

use alloc::sync::Arc;

use aster_systree::{Error, Result, SysAttrSet, SysAttrSetBuilder, SysPerms, SysStr};
use ostd::mm::{VmReader, VmWriter};

use crate::fs::cgroupfs::controller::CgroupSysNode;

/// The controller responsible for memory resource management in the cgroup subsystem.
///
/// Note that even if the controller is inactive, it still provides some interfaces
/// like "memory.pressure" for usage.
pub struct MemoryController {
    attrs: SysAttrSet,
}

impl MemoryController {
    pub(super) fn new(ctrl_state: super::SubCtrlState, is_root: bool) -> Option<Arc<Self>> {
        let is_active = ctrl_state.contains(super::SubCtrlState::MEMORY_CTRLS);

        let mut builder = SysAttrSetBuilder::new();
        Self::init_attr_set(&mut builder, is_root, is_active);
        let attrs = builder.build().expect("Failed to build attribute set");

        Some(Arc::new(Self { attrs }))
    }

    pub(super) fn init_attr_set(builder: &mut SysAttrSetBuilder, is_root: bool, is_active: bool) {
        builder.add(
            SysStr::from("memory.pressure"),
            SysPerms::DEFAULT_RO_ATTR_PERMS,
        );
        if is_active {
            // These attributes only exist on the non-root cgroup nodes.
            // However, it seems that the `memory.stat` attribute is also present on the root node in practice.
            // Currently the implementation follows the documentation strictly.
            //
            // Reference: <https://www.kernel.org/doc/html/latest/admin-guide/cgroup-v2.html#memory-controller>
            if !is_root {
                builder.add(SysStr::from("memory.stat"), SysPerms::DEFAULT_RO_ATTR_PERMS);
                builder.add(SysStr::from("memory.max"), SysPerms::DEFAULT_RO_ATTR_PERMS);
                builder.add(
                    SysStr::from("memory.events"),
                    SysPerms::DEFAULT_RO_ATTR_PERMS,
                );
            }
        }
    }
}

impl super::SubControl for MemoryController {
    fn attr_set(&self) -> &SysAttrSet {
        &self.attrs
    }

    fn read_attr_at(
        &self,
        _name: &str,
        _offset: usize,
        _writer: &mut VmWriter,
        _cgroup_node: &dyn CgroupSysNode,
    ) -> Result<usize> {
        Err(Error::AttributeError)
    }

    fn write_attr(
        &self,
        _name: &str,
        _reader: &mut VmReader,
        _cgroup_node: &dyn CgroupSysNode,
    ) -> Result<usize> {
        Err(Error::AttributeError)
    }
}
