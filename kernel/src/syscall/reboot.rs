// SPDX-License-Identifier: MPL-2.0

use super::SyscallReturn;
use crate::{
    prelude::*, process::{kill, process_table::process_table_mut, signal::{constants::SIGKILL, sig_num::SigNum, signals::{kernel::KernelSignal, user::UserSignal}}},
};

pub fn sys_reboot(
    magic1: i32,
    magic2: i32,
    op: i32,
    arg: Vaddr,
    ctx: &Context,
) -> Result<SyscallReturn>  {
    if op == 0x4321fedc {
        let table = process_table_mut();
        let process = table.get(1).unwrap();
        process.enqueue_signal(KernelSignal::new(SIGKILL));
    }

    Ok(SyscallReturn::Return(0))
}