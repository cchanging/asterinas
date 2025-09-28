// SPDX-License-Identifier: MPL-2.0

//! POSIX message queue filesystem implementation.
//!
//! This filesystem provides an interface for POSIX message queues through
//! the VFS layer. Message queues appear as files under /dev/mqueue.

mod fs;
mod inode;

use crate::fs::mqueue::fs::MqueueFsType;

/// Default block size for the mqueue filesystem.
const BLOCK_SIZE: usize = 4096;

/// Magic number for the mqueue filesystem.
const MQUEUE_FS_MAGIC: u64 = 0x4d515545;

pub(super) fn init() {
    super::registry::register(&MqueueFsType).unwrap();
}
