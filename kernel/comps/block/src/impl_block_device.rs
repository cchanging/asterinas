// SPDX-License-Identifier: MPL-2.0

use align_ext::AlignExt;
use io_util::batch::IoBatch;
use ostd::mm::{VmIo, VmReader, VmWriter};

use super::{
    BLOCK_SIZE, BlockDevice, SECTOR_SIZE,
    bio::{Bio, BioCompleteFn, BioEnqueueError, BioSegment, BioStatus, BioType},
    id::{Bid, Sid},
};
use crate::{
    bio::{BioDirection, is_sector_aligned},
    prelude::*,
};

/// Implements several commonly used APIs for the block device to conveniently
/// read and write block(s).
// TODO: Add API to submit bio with multiple segments in scatter/gather manner.
impl dyn BlockDevice {
    /// Synchronously reads contiguous blocks starting from the `bid`.
    pub fn read_blocks(
        &self,
        bid: Bid,
        bio_segment: BioSegment,
    ) -> Result<BioStatus, BioEnqueueError> {
        let bio = Bio::new(BioType::Read, Sid::from(bid), vec![bio_segment], None);
        let status = bio.submit_and_wait(self)?;
        Ok(status)
    }

    /// Asynchronously reads contiguous blocks starting from the `bid`.
    pub fn read_blocks_async(
        &self,
        bid: Bid,
        bio_segment: BioSegment,
        complete_fn: Option<BioCompleteFn>,
        io_batch: &mut IoBatch,
    ) -> Result<(), BioEnqueueError> {
        let bio = Bio::new(
            BioType::Read,
            Sid::from(bid),
            vec![bio_segment],
            complete_fn,
        );
        bio.submit(self, io_batch)
    }

    /// Synchronously writes contiguous blocks starting from the `bid`.
    pub fn write_blocks(
        &self,
        bid: Bid,
        bio_segment: BioSegment,
    ) -> Result<BioStatus, BioEnqueueError> {
        let bio = Bio::new(BioType::Write, Sid::from(bid), vec![bio_segment], None);
        let status = bio.submit_and_wait(self)?;
        Ok(status)
    }

    /// Asynchronously writes contiguous blocks starting from the `bid`.
    pub fn write_blocks_async(
        &self,
        bid: Bid,
        bio_segment: BioSegment,
        complete_fn: Option<BioCompleteFn>,
        io_batch: &mut IoBatch,
    ) -> Result<(), BioEnqueueError> {
        let bio = Bio::new(
            BioType::Write,
            Sid::from(bid),
            vec![bio_segment],
            complete_fn,
        );
        bio.submit(self, io_batch)
    }

    /// Issues a sync request
    pub fn sync(&self) -> Result<BioStatus, BioEnqueueError> {
        let bio = Bio::new(BioType::Flush, Sid::from(Bid::from_offset(0)), vec![], None);
        let status = bio.submit_and_wait(self)?;
        Ok(status)
    }
}

impl VmIo for dyn BlockDevice {
    /// Reads consecutive bytes of several sectors in size.
    fn read(&self, offset: usize, writer: &mut VmWriter) -> ostd::Result<()> {
        let read_len = writer.avail();
        if read_len == 0 {
            return Ok(());
        }

        let device_size = self.metadata().nr_sectors * SECTOR_SIZE;
        if offset >= device_size {
            return Ok(());
        }

        let effective_len = read_len.min(device_size - offset);
        let request_end = offset + effective_len;
        let aligned_offset = offset.align_down(SECTOR_SIZE);
        let aligned_end = request_end.align_up(SECTOR_SIZE);
        let aligned_len = aligned_end - aligned_offset;

        let (bio, bio_segment) = {
            let num_blocks = {
                let first = Bid::from_offset(aligned_offset).to_raw();
                let last = Bid::from_offset(aligned_end - 1).to_raw();
                (last - first + 1) as usize
            };
            let bio_segment = BioSegment::alloc_inner(
                num_blocks,
                aligned_offset % BLOCK_SIZE,
                aligned_len,
                BioDirection::FromDevice,
            );

            (
                Bio::new(
                    BioType::Read,
                    Sid::from_offset(aligned_offset),
                    vec![bio_segment.clone()],
                    None,
                ),
                bio_segment,
            )
        };

        let status = bio.submit_and_wait(self)?;
        match status {
            BioStatus::Complete => {
                let segment_offset = offset - aligned_offset;

                // `BioSegment`'s `VmIo::read` does not allow short reads,
                // so the writer must be precisely limited here.
                let mut limited_writer = writer.clone_exclusive();
                limited_writer.limit(effective_len);
                bio_segment.read(segment_offset, &mut limited_writer)?;
                writer.skip(effective_len);

                Ok(())
            }
            _ => Err(ostd::Error::IoError),
        }
    }

    /// Writes consecutive bytes of several sectors in size.
    fn write(&self, offset: usize, reader: &mut VmReader) -> ostd::Result<()> {
        let write_len = reader.remain();
        if write_len == 0 {
            return Ok(());
        }

        let device_size = self.metadata().nr_sectors * SECTOR_SIZE;
        if offset >= device_size {
            return Err(ostd::Error::InvalidArgs);
        }

        let effective_len = write_len.min(device_size - offset);
        let request_end = offset + effective_len;
        let aligned_offset = offset.align_down(SECTOR_SIZE);
        let aligned_end = request_end.align_up(SECTOR_SIZE);
        let aligned_len = aligned_end - aligned_offset;

        let bio_segment = {
            let num_blocks = {
                let first = Bid::from_offset(aligned_offset).to_raw();
                let last = Bid::from_offset(aligned_end - 1).to_raw();
                (last - first + 1) as usize
            };
            BioSegment::alloc_inner(
                num_blocks,
                aligned_offset % BLOCK_SIZE,
                aligned_len,
                BioDirection::ToDevice,
            )
        };

        // If the write range is not sector-aligned, preserve the bytes in the
        // surrounding sectors that are outside the user-requested range.
        if !is_sector_aligned(offset) || !is_sector_aligned(effective_len) {
            let read_bio = Bio::new(
                BioType::Read,
                Sid::from_offset(aligned_offset),
                vec![bio_segment.clone()],
                None,
            );
            if read_bio.submit_and_wait(self)? != BioStatus::Complete {
                return Err(ostd::Error::IoError);
            }
        }

        let segment_offset = offset - aligned_offset;

        // `BioSegment`'s `VmIo::write` does not allow short writes,
        // so the reader must be precisely limited here.
        let mut limited_reader = reader.clone();
        limited_reader.limit(effective_len);
        bio_segment.write(segment_offset, &mut limited_reader)?;
        reader.skip(effective_len);

        let bio = Bio::new(
            BioType::Write,
            Sid::from_offset(aligned_offset),
            vec![bio_segment],
            None,
        );
        let status = bio.submit_and_wait(self)?;
        match status {
            BioStatus::Complete => Ok(()),
            _ => Err(ostd::Error::IoError),
        }
    }
}

impl dyn BlockDevice {
    /// Asynchronously writes consecutive bytes of several sectors in size.
    pub fn write_bytes_async(
        &self,
        offset: usize,
        buf: &[u8],
        io_batch: &mut IoBatch,
    ) -> ostd::Result<()> {
        let write_len = buf.len();
        if !is_sector_aligned(offset) || !is_sector_aligned(write_len) {
            return Err(ostd::Error::InvalidArgs);
        }
        if write_len == 0 {
            return Ok(());
        }

        let bio = {
            let num_blocks = {
                let first = Bid::from_offset(offset).to_raw();
                let last = Bid::from_offset(offset + write_len - 1).to_raw();
                (last - first + 1) as usize
            };
            let bio_segment = BioSegment::alloc_inner(
                num_blocks,
                offset % BLOCK_SIZE,
                write_len,
                BioDirection::ToDevice,
            );
            bio_segment.write(0, &mut VmReader::from(buf).to_fallible())?;
            Bio::new(
                BioType::Write,
                Sid::from_offset(offset),
                vec![bio_segment],
                None,
            )
        };

        bio.submit(self, io_batch)?;
        Ok(())
    }
}

pub(super) fn general_complete_fn(
    bio_type: BioType,
    bio_status: BioStatus,
    complete_fn: Option<BioCompleteFn>,
) {
    if bio_status != BioStatus::Complete {
        ostd::error!(
            "failed to do {:?} on the device with error status: {:?}",
            bio_type,
            bio_status
        );
    }
    if let Some(complete_fn) = complete_fn {
        complete_fn(bio_status);
    }
}
