// SPDX-License-Identifier: MPL-2.0

// SPDX-License-Identifier: MPL-2.0

//! Provides [`SegmentSlice`] for quick duplication and slicing over [`USegment`].

use alloc::sync::Arc;
use core::ops::Range;

use ostd::{
    mm::{
        FallibleVmRead, FallibleVmWrite, Infallible, Paddr, UFrame, USegment, UntypedMem, VmIo,
        VmReader, VmWriter, PAGE_SIZE,
    },
    Error, Result,
};

/// A reference to a slice of a [`USegment`].
///
/// Cloning a [`SegmentSlice`] is cheap, as it only increments one reference
/// count. While cloning a [`USegment`] will increment the reference count of
/// many underlying pages.
///
/// The downside is that the [`SegmentSlice`] requires heap allocation. Also,
/// if any [`SegmentSlice`] of the original [`USegment`] is alive, all pages in
/// the original [`USegment`], including the pages that are not referenced, will
/// not be freed.
#[derive(Debug, Clone)]
pub struct SegmentSlice {
    inner: Arc<USegment>,
    range: Range<usize>,
}

impl SegmentSlice {
    /// Returns a part of the `USegment`.
    ///
    /// # Panics
    ///
    /// If `range` is not within the range of this `USegment`,
    /// then the method panics.
    pub fn range(&self, range: Range<usize>) -> Self {
        let orig_range = &self.range;
        let adj_range = (range.start + orig_range.start)..(range.end + orig_range.start);
        assert!(!adj_range.is_empty() && adj_range.end <= orig_range.end);

        Self {
            inner: self.inner.clone(),
            range: adj_range,
        }
    }

    /// Returns the start physical address.
    pub fn start_paddr(&self) -> Paddr {
        self.start_frame_index() * PAGE_SIZE
    }

    /// Returns the end physical address.
    pub fn end_paddr(&self) -> Paddr {
        (self.start_frame_index() + self.nframes()) * PAGE_SIZE
    }

    /// Returns the number of page frames.
    pub fn nframes(&self) -> usize {
        self.range.len()
    }

    /// Returns the number of bytes.
    pub fn nbytes(&self) -> usize {
        self.nframes() * PAGE_SIZE
    }

    /// Gets a reader for the slice.
    pub fn reader(&self) -> VmReader<'_, Infallible> {
        let mut reader = self.inner.reader();
        reader
            .skip(self.start_paddr() - self.inner.start_paddr())
            .limit(self.nbytes());
        reader
    }

    /// Gets a writer for the slice.
    pub fn writer(&self) -> VmWriter<'_, Infallible> {
        let mut writer = self.inner.writer();
        writer
            .skip(self.start_paddr() - self.inner.start_paddr())
            .limit(self.nbytes());
        writer
    }

    fn start_frame_index(&self) -> usize {
        self.inner.start_paddr() / PAGE_SIZE + self.range.start
    }
}

impl VmIo for SegmentSlice {
    fn read(&self, offset: usize, writer: &mut VmWriter) -> Result<()> {
        let read_len = writer.avail();
        // Do bound check with potential integer overflow in mind
        let max_offset = offset.checked_add(read_len).ok_or(Error::Overflow)?;
        if max_offset > self.nbytes() {
            return Err(Error::InvalidArgs);
        }
        let len = self
            .reader()
            .skip(offset)
            .read_fallible(writer)
            .map_err(|(e, _)| e)?;
        debug_assert!(len == read_len);
        Ok(())
    }

    fn write(&self, offset: usize, reader: &mut VmReader) -> Result<()> {
        let write_len = reader.remain();
        // Do bound check with potential integer overflow in mind
        let max_offset = offset.checked_add(reader.remain()).ok_or(Error::Overflow)?;
        if max_offset > self.nbytes() {
            return Err(Error::InvalidArgs);
        }
        let len = self
            .writer()
            .skip(offset)
            .write_fallible(reader)
            .map_err(|(e, _)| e)?;
        debug_assert!(len == write_len);
        Ok(())
    }
}

impl From<USegment> for SegmentSlice {
    fn from(segment: USegment) -> Self {
        let range = 0..segment.size() / PAGE_SIZE;
        Self {
            inner: Arc::new(segment),
            range,
        }
    }
}

impl From<SegmentSlice> for USegment {
    fn from(slice: SegmentSlice) -> Self {
        let start = slice.range.start * PAGE_SIZE;
        let end = slice.range.end * PAGE_SIZE;
        slice.inner.slice(&(start..end))
    }
}

impl From<UFrame> for SegmentSlice {
    fn from(frame: UFrame) -> Self {
        SegmentSlice::from(USegment::from(frame))
    }
}
