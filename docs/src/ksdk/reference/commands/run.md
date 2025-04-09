# cargo ksdk run

## Overview

`cargo ksdk run` is used to run the kernel with QEMU.
The usage is as follows:

```bash
cargo ksdk run [OPTIONS]
```

## Options

Most options are the same as those of `cargo ksdk build`.
Refer to the [documentation](build.md) of `cargo ksdk build`
for more details.

Additionally, when running the kernel using QEMU, we can setup the QEMU as a
debug server using option `--gdb-server`. This option supports an additional
comma separated configuration list:

 - `addr=ADDR`: the network or unix socket address on which the GDB server listens
    (default: `.ksdk-gdb-socket`, a local UNIX socket);
 - `wait-client`: let the GDB server wait for the GDB client before execution;
 - `vscode`: generate a '.vscode/launch.json' for debugging with Visual Studio Code
    (Requires [CodeLLDB](https://marketplace.visualstudio.com/items?itemName=vadimcn.vscode-lldb)).

See [Debug Command](debug.md) to interact with the GDB server in terminal.

## Examples

Launch a debug server via QEMU with an unix socket stub, e.g. `.debug`:

```bash
cargo ksdk run --gdb-server addr=.debug

```bash
cargo ksdk run --gdb-server --gdb-server-addr .debug
```

Launch a debug server via QEMU with a TCP stub, e.g., `localhost:1234`:

```bash
cargo ksdk run --gdb-server addr=:1234
```

Launch a debug server via QEMU and use VSCode to interact with:

```bash
cargo ksdk run --gdb-server wait-client,vscode,addr=:1234
```

Launch a debug server via QEMU and use VSCode to interact with:

```bash
cargo ksdk run --gdb-server --gdb-vsc --gdb-server-addr :1234
```
