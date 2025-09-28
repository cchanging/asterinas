// SPDX-License-Identifier: MPL-2.0

//! POSIX message queue filesystem implementation.

use alloc::sync::Arc;
use core::sync::atomic::{AtomicU64, Ordering};

use super::inode::MqueueInode;
use crate::{
    fs::{
        registry::{FsProperties, FsType},
        utils::{FileSystem, FsFlags, Inode, SuperBlock},
    },
    prelude::*,
};

/// POSIX message queue filesystem.
pub struct MqueueFs {
    /// Superblock information.
    sb: SuperBlock,
    /// Root inode of the filesystem.
    root_inode: Arc<MqueueInode>,
    /// Global inode number counter.
    next_ino: AtomicU64,
}

impl MqueueFs {
    /// Creates a new mqueue filesystem instance.
    pub fn new() -> Arc<Self> {
        Arc::new_cyclic(|weak_self| {
            let sb = SuperBlock::new(super::MQUEUE_FS_MAGIC, 4096, 4096);
            Self {
                sb,
                root_inode: MqueueInode::new_root(weak_self.clone()), // Root inode number is 1
                next_ino: AtomicU64::new(2), // Start from 2 since root is 1
            }
        })
    }

    /// Allocates a new inode number.
    #[expect(dead_code)]
    pub fn alloc_ino(&self) -> u64 {
        self.next_ino.fetch_add(1, Ordering::Relaxed)
    }
}

impl FileSystem for MqueueFs {
    fn sync(&self) -> Result<()> {
        // Message queues are typically in-memory, no sync needed
        Ok(())
    }

    fn root_inode(&self) -> Arc<dyn Inode> {
        self.root_inode.clone()
    }

    fn sb(&self) -> SuperBlock {
        self.sb.clone()
    }

    fn flags(&self) -> FsFlags {
        FsFlags::empty()
    }
}

pub(super) struct MqueueFsType;

impl FsType for MqueueFsType {
    fn name(&self) -> &'static str {
        "mqueue"
    }

    fn properties(&self) -> FsProperties {
        FsProperties::empty()
    }

    fn create(
        &self,
        _args: Option<CString>,
        _disk: Option<Arc<dyn aster_block::BlockDevice>>,
    ) -> Result<Arc<dyn FileSystem>> {
        Ok(MqueueFs::new())
    }

    fn sysnode(&self) -> Option<Arc<dyn aster_systree::SysNode>> {
        None
    }
}
