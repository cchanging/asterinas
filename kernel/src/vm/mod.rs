// SPDX-License-Identifier: MPL-2.0

//! Virtual memory (VM).
//!
//! There are two primary VM abstractions:
//!  * Virtual Memory Address Regions (VMARs) a type of capability that manages
//!    user address spaces.
//!  * Virtual Memory Objects (VMOs) are are a type of capability that
//!    represents a set of memory pages.
//!
//! The concepts of VMARs and VMOs are originally introduced by
//! [Zircon](https://fuchsia.dev/fuchsia-src/reference/kernel_objects/vm_object).
//! As capabilities, the two abstractions are aligned with our goal
//! of everything-is-a-capability, although their specifications and
//! implementations in C/C++ cannot apply directly to Astros.
//! In Astros, VMARs and VMOs, as well as other capabilities, are implemented
//! as zero-cost capabilities.

use ksdk_frame_allocator::FrameAllocator;
use ksdk_heap_allocator::{type_from_layout, HeapAllocator};

pub mod page_fault_handler;
pub mod perms;
pub mod util;
pub mod vmar;
pub mod vmo;

#[kstd::global_frame_allocator]
static FRAME_ALLOCATOR: FrameAllocator = FrameAllocator;

#[kstd::global_heap_allocator]
static HEAP_ALLOCATOR: HeapAllocator = HeapAllocator;

#[kstd::global_heap_allocator_slot_map]
const fn slot_type_from_layout(layout: core::alloc::Layout) -> Option<kstd::mm::heap::SlotInfo> {
    type_from_layout(layout)
}

/// Total physical memory in the entire system in bytes.
pub fn mem_total() -> usize {
    use kstd::boot::{boot_info, memory_region::MemoryRegionType};

    let regions = &boot_info().memory_regions;
    let total = regions
        .iter()
        .filter(|region| region.typ() == MemoryRegionType::Usable)
        .map(|region| region.len())
        .sum::<usize>();

    total
}
