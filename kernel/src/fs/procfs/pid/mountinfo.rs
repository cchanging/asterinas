// SPDX-License-Identifier: MPL-2.0

use crate::{
    fs::{
        procfs::template::{FileOps, ProcFileBuilder},
        utils::Inode,
    },
    prelude::*,
    process::posix_thread::AsPosixThread,
    thread::Thread,
};

/// Represents the inode at `/proc/[pid]/mountinfo`.
pub struct MountInfoFileOps {
    thread_ref: Arc<Thread>,
}

impl MountInfoFileOps {
    pub fn new_inode(thread_ref: Arc<Thread>, parent: Weak<dyn Inode>) -> Arc<dyn Inode> {
        ProcFileBuilder::new(Self { thread_ref })
            .parent(parent)
            .build()
            .unwrap()
    }
}

impl FileOps for MountInfoFileOps {
    fn data(&self) -> Result<Vec<u8>> {
        let posix_thread = self.thread_ref.as_posix_thread().unwrap();
        let fs = posix_thread.fs();
        let fs_resolver = fs.resolver().read();
        let root_mount = fs_resolver.root().mount_node();

        let bytes = crate::fs::path::generate_mountinfo(root_mount).into_bytes();
        Ok(bytes)
    }
}
