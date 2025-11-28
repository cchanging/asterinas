// SPDX-License-Identifier: MPL-2.0

use alloc::sync::Arc;

use aster_systree::{Error, Result, SysAttrSet, SysAttrSetBuilder, SysPerms, SysStr};
use ostd::mm::{VmReader, VmWriter};

use crate::fs::cgroupfs::controller::CgroupSysNode;

/// The controller responsible for PID resource management in the cgroup subsystem.
///
/// This controller will only provide interfaces in non-root cgroup nodes.
pub struct PidsController {
    attrs: SysAttrSet,
}

impl PidsController {
    pub(super) fn new(ctrl_state: super::SubCtrlState, is_root: bool) -> Option<Arc<Self>> {
        if !ctrl_state.contains(super::SubCtrlState::PIDS_CTRLS) || is_root {
            return None;
        }

        let mut builder = SysAttrSetBuilder::new();
        Self::init_attr_set(&mut builder, false);
        let attrs = builder.build().expect("Failed to build attribute set");

        Some(Arc::new(Self { attrs }))
    }

    pub(super) fn init_attr_set(builder: &mut SysAttrSetBuilder, is_root: bool) {
        if !is_root {
            builder.add(SysStr::from("pids.max"), SysPerms::DEFAULT_RW_ATTR_PERMS);
        }
    }
}

impl super::SubControl for PidsController {
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
