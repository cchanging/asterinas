// SPDX-License-Identifier: MPL-2.0

pub fn available_memory() -> usize {
    let regions = &kstd::boot::boot_info().memory_regions;
    regions.iter().map(|region| region.len()).sum()
}
