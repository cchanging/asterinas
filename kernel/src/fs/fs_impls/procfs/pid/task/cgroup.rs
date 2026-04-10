// SPDX-License-Identifier: MPL-2.0

use aster_util::printer::VmPrinter;

use crate::{
    fs::{
        cgroupfs::{CgroupNamespace, CgroupSysNode, CgroupSystem},
        file::mkmod,
        procfs::{
            pid::TidDirOps,
            template::{FileOps, ProcFileBuilder},
        },
        vfs::inode::Inode,
    },
    prelude::*,
    process::posix_thread::AsPosixThread,
};

/// Represents the inode at `/proc/[pid]/task/[tid]/cgroup` (and also `/proc/[pid]/cgroup`).
pub struct CgroupFileOps(TidDirOps);

impl CgroupFileOps {
    pub fn new_inode(dir: &TidDirOps, parent: Weak<dyn Inode>) -> Arc<dyn Inode> {
        // Reference: <https://elixir.bootlin.com/linux/v6.16.5/source/fs/proc/base.c#L3379>
        ProcFileBuilder::new(Self(dir.clone()), mkmod!(a+r))
            .parent(parent)
            .build()
            .unwrap()
    }
}

impl FileOps for CgroupFileOps {
    fn read_at(&self, offset: usize, writer: &mut VmWriter) -> Result<usize> {
        let cgroup: Arc<dyn CgroupSysNode> = match self.0.process_ref.cgroup().get() {
            Some(cgroup) => cgroup.clone(),
            None => CgroupSystem::singleton().clone(),
        };

        let thread = current_thread!();
        let ns_proxy = thread.as_posix_thread().unwrap().ns_proxy().lock();
        let cgroup_ns = ns_proxy
            .as_ref()
            .map(|ns_proxy| ns_proxy.cgroup_ns())
            .unwrap_or(CgroupNamespace::get_init_singleton());
        let path = cgroup_ns.virtualize_path(cgroup);

        let mut printer = VmPrinter::new_skip(writer, offset);
        writeln!(printer, "0::{}", path)?;

        Ok(printer.bytes_written())
    }
}
