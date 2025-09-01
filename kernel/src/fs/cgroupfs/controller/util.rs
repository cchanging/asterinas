// SPDX-License-Identifier: MPL-2.0

use core::str::FromStr;

use aster_systree::{Error, Result};

use crate::prelude::*;

pub(super) fn read_context_from_reader(reader: &mut VmReader) -> Result<(String, usize)> {
    let mut buffer = alloc::vec![0; reader.remain()];
    let len = reader
        .read_fallible(&mut VmWriter::from(buffer.as_mut_slice()))
        .map_err(|_| Error::AttributeError)?;

    let context = String::from_utf8(buffer).map_err(|_| Error::AttributeError)?;

    Ok((context, len))
}

pub(super) fn parse_context_to_val<T: FromStr>(context: String) -> Result<T> {
    let strip_string = context.trim();
    strip_string.parse::<T>().map_err(|_| Error::AttributeError)
}
