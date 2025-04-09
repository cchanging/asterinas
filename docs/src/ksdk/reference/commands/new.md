# cargo ksdk new

## Overview

The `cargo ksdk new` command
is used to create a kernel project
or a new library project.
The usage is as follows:

```bash
cargo ksdk new [OPTIONS] <name>
```

## Arguments

`<name>`: the name of the crate.

## Options

`--kernel`:
Use the kernel template.
If this option is not set,
the library template will be used by default.

`--library`:
Use the library template. This is the default option.

## Examples

- Create a new kernel named `myos`: 

```bash
cargo ksdk new --kernel myos
```

- Create a new library named `mylib`:

```bash
cargo ksdk new mylib
```
