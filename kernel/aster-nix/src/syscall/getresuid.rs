// SPDX-License-Identifier: MPL-2.0

use super::SyscallReturn;
use crate::{prelude::*, process::credentials, util::write_val_to_user};

pub fn sys_getresuid(ruid_ptr: Vaddr, euid_ptr: Vaddr, suid_ptr: Vaddr) -> Result<SyscallReturn> {
    debug!("ruid_ptr = 0x{ruid_ptr:x}, euid_ptr = 0x{euid_ptr:x}, suid_ptr = 0x{suid_ptr:x}");

    let credentials = credentials();

    let ruid = credentials.ruid();
    write_val_to_user(ruid_ptr, &ruid)?;

    let euid = credentials.euid();
    write_val_to_user(euid_ptr, &euid)?;

    let suid = credentials.suid();
    write_val_to_user(suid_ptr, &suid)?;

    Ok(SyscallReturn::Return(0))
}
