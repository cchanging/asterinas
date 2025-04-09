# ksdk-frame-allocator

This is the default buddy system frame allocator shipped with
[KSDK](https://crates.io/crates/cargo-ksdk). It relies on the physical frame
metadata system in [KSTD](https://crates.io/crates/kstd) to provide a heap-free
implementation of a buddy system allocator for OS kernels. It also features
per-CPU caches and pools for scalable allocations.

This crate is part of the [Astros](https://github.com/astros/astros)
project.
