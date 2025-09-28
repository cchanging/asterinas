// SPDX-License-Identifier: MPL-2.0

use crate::{
    fs::{
        fs_resolver::{FsPath, FsResolver},
        utils::{InodeMode, InodeType},
    },
    prelude::*,
};

pub(super) fn init_in_first_process(fs_resolver: &FsResolver) -> Result<()> {
    let dev = fs_resolver.lookup(&FsPath::try_from("/dev")?)?;

    dev.new_fs_child(
        "mqueue",
        InodeType::Dir,
        InodeMode::from_bits_truncate(0o1777),
    )?;

    Ok(())
}
