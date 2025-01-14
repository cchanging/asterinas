# Accelerate OS development with Astros KSDK

[![Crates.io](https://img.shields.io/crates/v/cargo-ksdk.svg)](https://crates.io/crates/cargo-ksdk)
[![KSDK Test](https://github.com/astros/astros/actions/workflows/ksdk_test.yml/badge.svg?event=push)](https://github.com/astros/astros/actions/workflows/ksdk_test.yml)

### What is it?

KSDK (short for Kernel Software Development Kit) is designed to simplify the development of Rust operating systems. It aims to streamline the process by leveraging the framekernel architecture, originally proposed by [Astros](https://github.com/astros/astros).

`cargo-ksdk` is a command-line tool that facilitates project management for those developed on the framekernel architecture. Much like Cargo for Rust projects, `cargo-ksdk` enables building, running, and testing projects conveniently.

### Install the tool

#### Requirements

Currently, `cargo-ksdk` only supports x86_64 ubuntu system. 

To run a kernel with QEMU, `cargo-ksdk` requires the following tools to be installed: 
- Rust >= 1.75.0
- cargo-binutils
- gcc
- qemu-system-x86_64
- grub-mkrescue
- ovmf 
- xorriso

About how to install Rust, you can refer to the [official site](https://www.rust-lang.org/tools/install).

After installing Rust, you can install Cargo tools by
```bash
cargo install cargo-binutils
```

Other tools can be installed by
```bash
apt install build-essential grub2-common qemu-system-x86 ovmf xorriso
```

#### Install 

Then, `cargo-ksdk` can be installed by
```bash
cargo install cargo-ksdk
``` 

#### Upgrade

If `cargo-ksdk` is already installed, the tool can be upgraded by
```bash
cargo install --force cargo-ksdk
```

### Getting started

Here we provide a simple demo to demonstrate how to create and run a simple kernel with `cargo-ksdk`.

With `cargo-ksdk`, a kernel project can be created by one command
```bash
cargo ksdk new --kernel my-first-os
```

Then, you can run the kernel with
```bash
cd my-first-os && cargo ksdk run
```

You will see `Hello world from guest kernel!` from your console. 

### Basic usage

The basic usage of `cargo-ksdk` is
```bash
cargo ksdk <COMMAND>
```
Currently we support following commands:
- **new**: Create a new kernel package or library package
- **build**: Compile the project and its dependencies
- **run**: Run the kernel with a VMM
- **debug**: Debug a remote target via GDB
- **test**: Execute kernel mode unit test by starting a VMM
- **check**: Analyze the current package and report errors
- **clippy**: Check the current package and catch common mistakes
- **doc**: Build Rust documentations

The following command can be used to discover the available options for each command.
```bash
cargo ksdk help <COMMAND>
```

### The KSDK manifest

`cargo-ksdk` utilizes a configuration file named `KSDK.toml` to define its precise behavior. To learn more about the manifest specification, please refer to [the book](https://astros.github.io/book/ksdk/reference/manifest.html).

### Contributing

Astros KSDK is developed as a sub-project of [Astros](https://github.com/astros/astros). It shares the same repository and versioning rules with the kernel. Please contribute to KSDK according to the contribution guide of Astros.

#### Note for Visual Studio Code users

To enable advanced features of the editor on KSDK, please open the Astros repository as a workspace using the `File > Open Workspace from File...` menu entry, and select the file `.code-workspace` in the Astros repository root as the configuration.
