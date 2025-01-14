# KSDK User Reference

The Astros KSDK is a command line tool that can be used
as a subcommand of Cargo.
The common usage of KSDK is:

```bash
cargo ksdk <COMMAND>
```

You can use `cargo ksdk -h`
to see the full list of available commands.
For the specific usage of a subcommand,
you can use `cargo ksdk help <COMMAND>`.

## Manifest

The KSDK utilizes a manifest named `KSDK.toml`
to define its precise behavior regarding
how to run a kernel with QEMU.
The `KSDK.toml` file should be placed
in the same folder as the project's `Cargo.toml`.
The [Manifest documentation](manifest.md)
provides an introduction
to all the available configuration options.

The command line tool can also be used
to set the options in the manifest.
If both occur, the command line options
will always take priority over the options in the manifest.
For example, if the manifest defines the path of QEMU as:

```toml
[qemu]
path = "/usr/bin/qemu-system-x86_64"
```

But the user provides a new QEMU path
when running the project using:

```bash
cargo ksdk run --qemu.path="/usr/local/qemu-kvm"
```

Then, the actual path of QEMU should be `/usr/local/qemu-kvm`
since command line options have higher priority.
