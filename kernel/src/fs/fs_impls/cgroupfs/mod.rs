// SPDX-License-Identifier: MPL-2.0

pub use controller::{CpuStatKind, charge_cpu_time};
use fs::CgroupFsType;
pub use systree_node::{CgroupMembership, CgroupNode, CgroupSysNode};

mod controller;
mod fs;
mod inode;
mod systree_node;

use crate::fs::cgroupfs::systree_node::CgroupSystem;

// This method should be called during kernel file system initialization,
// _after_ `aster_systree::init`.
pub(super) fn init() {
    crate::fs::vfs::registry::register(&CgroupFsType).unwrap();
}
