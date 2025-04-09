# Summary

[Introduction](README.md)

# Astros Kernel

* [Getting Started](kernel/README.md)
* [Advanced Build and Test Instructions](kernel/advanced-instructions.md)
    * [Intel TDX](kernel/intel_tdx.md)
* [The Framekernel Architecture](kernel/the-framekernel-architecture.md)
* [Linux Compatibility](kernel/linux-compatibility.md)
* [Roadmap](kernel/roadmap.md)

# Astros KSTD

* [An Overview of KSTD](kstd/README.md)
* [Example: Writing a Kernel in 100 Lines of Safe Rust](kstd/a-100-line-kernel.md)
* [Example: Writing a Driver in 100 Lines of Safe Rust]()
* [Soundness Analysis]()

# Astros KSDK

* [KSDK User Guide](ksdk/guide/README.md)
    * [Why KSDK](ksdk/guide/why.md)
    * [Creating an OS Project](ksdk/guide/create-project.md)
    * [Testing or Running an OS Project](ksdk/guide/run-project.md)
    * [Working in a Workspace](ksdk/guide/work-in-workspace.md)
    * [Advanced Topics](ksdk/guide/advanced_topics.md)
        * [Intel TDX](ksdk/guide/intel-tdx.md)
* [KSDK User Reference](ksdk/reference/README.md)
    * [Commands](ksdk/reference/commands/README.md)
        * [cargo ksdk new](ksdk/reference/commands/new.md)
        * [cargo ksdk build](ksdk/reference/commands/build.md)
        * [cargo ksdk run](ksdk/reference/commands/run.md)
        * [cargo ksdk test](ksdk/reference/commands/test.md)
        * [cargo ksdk debug](ksdk/reference/commands/debug.md)
        * [cargo ksdk profile](ksdk/reference/commands/profile.md)
    * [Manifest](ksdk/reference/manifest.md)

# How to Contribute

* [Before You Contribute]()
* [Code Organization]()
* [Style Guidelines]()
    * [General Guidelines]() 
    * [Rust Guidelines](to-contribute/style-guidelines/rust-guidelines.md) 
    * [Git Guidelines]() 
* [Boterinas](to-contribute/boterinas.md)
* [Version Bump](to-contribute/version-bump.md)
* [Community]()
* [Code of Conduct]()

# Request for Comments (RFC)

* [RFC Overview]()
    * [RFC-0001: RFC Process]()
    * [RFC-0002: Kernel Software Development Kit (KSDK)]()
