// SPDX-License-Identifier: MPL-2.0

//! Inode implementations for mqueue filesystem.

use alloc::{string::String, sync::Arc};
use core::time::Duration;

use super::fs::MqueueFs;
use crate::{
    fs::utils::{Inode, InodeMode, InodeType, Metadata},
    prelude::*,
    process::{Gid, Uid},
};

/// Root inode of the mqueue filesystem.
#[expect(dead_code)]
pub struct MqueueInode {
    /// Name of the inode.
    name: String,
    /// Inode metadata.
    metadata: RwLock<Metadata>,
    /// Reference to the filesystem.
    fs: Weak<MqueueFs>,
    /// Inode number.
    ino: u64,
}

impl MqueueInode {
    pub(super) fn new_root(fs: Weak<MqueueFs>) -> Arc<Self> {
        let metadata =
            Metadata::new_dir(1, InodeMode::from_bits_truncate(0o1777), super::BLOCK_SIZE);
        Arc::new(Self {
            name: String::from("/"),
            metadata: RwLock::new(metadata),
            fs,
            ino: 1,
        })
    }
}

impl Inode for MqueueInode {
    fn size(&self) -> usize {
        // TODO: This should return the number of child inodes
        0
    }

    fn resize(&self, new_size: usize) -> Result<()> {
        if new_size != 0 {
            return_errno_with_message!(Errno::EINVAL, "cannot resize directory");
        }

        Ok(())
    }

    fn metadata(&self) -> Metadata {
        *self.metadata.read()
    }

    fn ino(&self) -> u64 {
        self.ino
    }

    fn type_(&self) -> InodeType {
        if self.ino() == 1 {
            InodeType::Dir
        } else {
            InodeType::File
        }
    }

    fn mode(&self) -> Result<InodeMode> {
        Ok(self.metadata.read().mode)
    }

    fn set_mode(&self, mode: InodeMode) -> Result<()> {
        self.metadata.write().mode = mode;
        Ok(())
    }

    fn owner(&self) -> Result<Uid> {
        Ok(self.metadata.read().uid)
    }

    fn set_owner(&self, uid: Uid) -> Result<()> {
        self.metadata.write().uid = uid;
        Ok(())
    }

    fn group(&self) -> Result<Gid> {
        Ok(self.metadata.read().gid)
    }

    fn set_group(&self, gid: Gid) -> Result<()> {
        self.metadata.write().gid = gid;
        Ok(())
    }

    fn atime(&self) -> Duration {
        self.metadata.read().atime
    }

    fn set_atime(&self, time: Duration) {
        self.metadata.write().atime = time;
    }

    fn mtime(&self) -> Duration {
        self.metadata.read().mtime
    }

    fn set_mtime(&self, time: Duration) {
        self.metadata.write().mtime = time;
    }

    fn ctime(&self) -> Duration {
        self.metadata.read().ctime
    }

    fn set_ctime(&self, time: Duration) {
        self.metadata.write().ctime = time;
    }

    fn fs(&self) -> Arc<dyn crate::fs::utils::FileSystem> {
        self.fs.upgrade().unwrap()
    }
}
