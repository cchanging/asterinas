// SPDX-License-Identifier: MPL-2.0

//! This module offers `/proc/cmdline` file support.

use ostd::cpu::num_cpus;

use crate::{
    fs::{
        procfs::template::{FileOps, ProcFileBuilder},
        utils::Inode,
    },
    prelude::*,
};

/// Represents the inode at `/proc/cmdline`.
pub struct CmdLineFileOps;

impl CmdLineFileOps {
    /// Create a new inode for `/proc/cmdline`.
    pub fn new_inode(parent: Weak<dyn Inode>) -> Arc<dyn Inode> {
        ProcFileBuilder::new(Self).parent(parent).build().unwrap()
    }
}

impl FileOps for CmdLineFileOps {
    /// Retrieve the data for `/proc/cpuinfo`.
    fn data(&self) -> Result<Vec<u8>> {
        Ok(Vec::new())
    }
}
