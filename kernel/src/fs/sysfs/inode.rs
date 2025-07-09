// SPDX-License-Identifier: MPL-2.0

use alloc::sync::{Arc, Weak};

use ostd::sync::RwLock;

use crate::{
    fs::{notify::FsnotifyCommon, utils::{FileSystem, InnerNode, Inode, InodeMode, KernelFsInode, Metadata}},
    Result,
};

/// An inode abstraction used in the sysfs filesystem.
pub struct SysFsInode {
    /// The corresponding node in the SysTree.
    inner_node: InnerNode,
    /// The metadata of this inode.
    ///
    /// Most of the metadata (e.g., file size, timestamps)
    /// can be determined upon the creation of an inode,
    /// and are thus kept intact inside the immutable `metadata` field.
    /// Currently, the only mutable metadata is `mode`,
    /// which allows user space to `chmod` an inode on sysfs.
    metadata: Metadata,
    /// The file mode (permissions) of this inode, protected by a lock.
    mode: RwLock<InodeMode>,
    /// Weak reference to the parent inode.
    parent: Weak<SysFsInode>,
    /// Weak self-reference for cyclic data structures.
    this: Weak<SysFsInode>,
    /// Fsnotify common.
    fsnotify_common: FsnotifyCommon,
}

impl KernelFsInode for SysFsInode {
    fn new_arc(
        inner_node: InnerNode,
        metadata: Metadata,
        mode: InodeMode,
        parent: Weak<Self>,
    ) -> Arc<Self>
    where
        Self: Sized,
    {
        Arc::new_cyclic(|this| Self {
            inner_node,
            metadata,
            mode: RwLock::new(mode),
            parent,
            this: this.clone(),
            fsnotify_common: FsnotifyCommon::new(),
        })
    }

    fn inner_node(&self) -> &InnerNode {
        &self.inner_node
    }

    fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    fn mode(&self) -> Result<InodeMode> {
        Ok(*self.mode.read())
    }

    fn set_mode(&self, mode: InodeMode) -> Result<()> {
        *self.mode.write() = mode;
        Ok(())
    }

    fn parent(&self) -> &Weak<Self> {
        &self.parent
    }

    fn this(&self) -> Arc<Self> {
        self.this.upgrade().expect("Weak ref invalid")
    }

    fn fsnotify_common(&self) -> &FsnotifyCommon {
        &self.fsnotify_common
    }
}

impl Inode for SysFsInode {
    fn fs(&self) -> Arc<dyn FileSystem> {
        super::singleton().clone()
    }
}
