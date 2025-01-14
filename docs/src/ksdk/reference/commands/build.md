# cargo ksdk build

## Overview

The `cargo ksdk build` command is used to
compile the project and its dependencies.
The usage is as follows:

```bash
cargo ksdk build [OPTIONS]
```

## Options
The options can be divided into two types:
Cargo options that can be accepted by Cargo,
and Manifest options that can also be defined
in the manifest named `KSDK.toml`.

### Cargo options

- `--profile <PROFILE>`:
Build artifacts with the specified Cargo profile
(built-in candidates are 'dev', 'release', 'test', and 'bench')
[default: dev]

- `--release`:
Build artifacts in release mode, with optimizations

- `--features <FEATURES>`:
Space or comma separated list of features to activate

- `--no-default-features`:
Do not activate the `default` features

- `--config <KEY=VALUE>`:
Override a configuration value

More Cargo options will be supported in future versions of KSDK.

### Manifest options

These options can also be defined
in the project's manifest named `KSDK.toml`.
Command line options are used to override
or append values in `KSDK.toml`.
The allowed values for each option can be found
in the [Manifest Documentation](../manifest.md).

- `--kcmd-args <ARGS>`:
Command line arguments for the guest kernel
- `--init-args <ARGS>`:
Command line arguments for the init process
- `--initramfs <PATH>`:
Path of the initramfs
- `--boot-method <METHOD>`:
The method to boot the kernel
- `--grub-mkrescue <PATH>`:
Path of grub-mkrescue
- `--grub-boot-protocol <PROTOCOL>`:
The boot protocol for booting the kernel
- `--display-grub-menu`:
To display the GRUB menu if booting with GRUB
- `--qemu-exe <FILE>`:
The QEMU executable file
- `--qemu-args <ARGS>`:
Extra arguments for running QEMU
- `--strip-elf`:
Whether to strip the built kernel ELF using `rust-strip`
- `--scheme <SCHEME>`:
Select the specific configuration scheme provided in the KSDK manifest
- `--encoding <FORMAT>`:
Denote the encoding format for kernel self-decompression

## Examples

- Build a project with `./initramfs.cpio.gz`
as the initramfs and `multiboot2` as the boot protocol used by GRUB:

```bash
cargo ksdk build --initramfs="./initramfs.cpio.gz" --grub-boot-protocol="multiboot2"
```

- Build a project and append `sh`, `-l`
to init process arguments:

```bash
cargo ksdk build --init_args="sh" --init_args="-l"
```
