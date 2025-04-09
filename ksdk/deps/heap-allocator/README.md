# ksdk-heap-allocator

This is the default slab-based global heap allocator shipped with
[KSDK](https://crates.io/crates/cargo-ksdk). It relies on the slab mechanism in
[KSTD](https://crates.io/crates/kstd) to provide a fast, memory-efficient
implementation of a global heap allocator for OS kernels. It also features
per-CPU caches for scalable allocations.

This crate is part of the [Astros](https://github.com/astros/astros)
project.
