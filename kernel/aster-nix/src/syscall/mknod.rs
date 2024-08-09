// SPDX-License-Identifier: MPL-2.0

use super::SyscallReturn;
use crate::{
    device::get_device,
    fs::{
        file_table::FileDesc,
        fs_resolver::{FsPath, AT_FDCWD},
        utils::{InodeMode, InodeType},
    },
    prelude::*,
    syscall::{constants::MAX_FILENAME_LEN, stat::FileType},
};

pub fn sys_mknodat(
    dirfd: FileDesc,
    path_addr: Vaddr,
    mode: u16,
    dev: usize,
) -> Result<SyscallReturn> {
    let path = CurrentUserSpace::get().read_cstring(path_addr, MAX_FILENAME_LEN)?;
    let current = current!();
    let inode_mode = {
        let mask_mode = mode & !current.umask().read().get();
        InodeMode::from_bits_truncate(mask_mode)
    };
    let file_type = FileType::from_mode(mode);
    debug!(
        "dirfd = {}, path = {:?}, inode_mode = {:?}, file_type = {:?}, dev = {}",
        dirfd, path, inode_mode, file_type, dev
    );

    let (dir_dentry, name) = {
        let path = path.to_string_lossy();
        if path.is_empty() {
            return_errno_with_message!(Errno::ENOENT, "path is empty");
        }
        let fs_path = FsPath::new(dirfd, path.as_ref())?;
        current
            .fs()
            .read()
            .lookup_dir_and_new_basename(&fs_path, false)?
    };

    match file_type {
        FileType::RegularFile => {
            let _ = dir_dentry.new_fs_child(&name, InodeType::File, inode_mode)?;
        }
        FileType::CharacterDevice | FileType::BlockDevice => {
            let device_inode = get_device(dev)?;
            let _ = dir_dentry.mknod(&name, inode_mode, device_inode)?;
        }
        FileType::Fifo | FileType::Socket => {
            return_errno_with_message!(Errno::EINVAL, "unsupported file types")
        }
        _ => return_errno_with_message!(Errno::EPERM, "unimplemented file types"),
    }

    Ok(SyscallReturn::Return(0))
}

pub fn sys_mknod(path_addr: Vaddr, mode: u16, dev: usize) -> Result<SyscallReturn> {
    self::sys_mknodat(AT_FDCWD, path_addr, mode, dev)
}
