<p align="center">
    <img src="docs/src/images/logo_en.svg" alt="astros-logo" width="620"><br>
    A secure, fast, and general-purpose OS kernel written in Rust and compatible with Linux<br/>
    <a href="https://github.com/astros/astros/actions/workflows/test_x86.yml"><img src="https://github.com/astros/astros/actions/workflows/test_x86.yml/badge.svg?event=push" alt="Test x86-64" style="max-width: 100%;"></a>
    <a href="https://github.com/astros/astros/actions/workflows/test_x86_tdx.yml"><img src="https://github.com/astros/astros/actions/workflows/test_x86_tdx.yml/badge.svg" alt="Test Intel TDX" style="max-width: 100%;"></a>
    <a href="https://astros.github.io/benchmark/"><img src="https://github.com/astros/astros/actions/workflows/benchmark_x86.yml/badge.svg" alt="Benchmark x86-64" style="max-width: 100%;"></a>
    <br/>
</p>

English | [中文版](README_CN.md) | [日本語](README_JP.md)

## Introducing Astros

Astros is a _secure_, _fast_, and _general-purpose_ OS kernel
that provides _Linux-compatible_ ABI.
It can serve as a seamless replacement for Linux
while enhancing _memory safety_ and _developer friendliness_.

* Astros prioritizes memory safety
by employing Rust as its sole programming language
and limiting the use of _unsafe Rust_
to a clearly defined and minimal Trusted Computing Base (TCB).
This innovative approach,
known as [the framekernel architecture](https://astros.github.io/book/kernel/the-framekernel-architecture.html),
establishes Astros as a more secure and dependable kernel option.

* Astros surpasses Linux in terms of developer friendliness.
It empowers kernel developers to
(1) utilize the more productive Rust programming language,
(2) leverage a purpose-built toolkit called [KSDK](https://astros.github.io/book/ksdk/guide/index.html) to streamline their workflows,
and (3) choose between releasing their kernel modules as open source
or keeping them proprietary,
thanks to the flexibility offered by [MPL](#License).

While the journey towards a production-grade OS kernel is challenging,
we are steadfastly progressing towards this goal.
Over the course of 2024,
we significantly enhanced Astros's maturity,
as detailed in [our end-year report](https://astros.github.io/2025/01/20/astros-in-2024.html).
In 2025, our primary goal is to make Astros production-ready on x86-64 virtual machines
and attract real users!

## Getting Started

Get yourself an x86-64 Linux machine with Docker installed.
Follow the three simple steps below to get Astros up and running.

1. Download the latest source code.

```bash
git clone https://github.com/astros/astros
```

2. Run a Docker container as the development environment.

```bash
docker run -it --privileged --network=host --device=/dev/kvm -v $(pwd)/astros:/root/astros astros/astros:0.14.1-20250326
```

3. Inside the container, go to the project folder to build and run Astros.

```bash
make build
make run
```

If everything goes well, Astros is now up and running inside a VM.

## The Book

See [The Astros Book](https://astros.github.io/book/) to learn more about the project.

## License

Astros's source code and documentation primarily use the 
[Mozilla Public License (MPL), Version 2.0](https://github.com/astros/astros/blob/main/LICENSE-MPL).
Select components are under more permissive licenses,
detailed [here](https://github.com/astros/astros/blob/main/.licenserc.yaml). For the rationales behind the choice of MPL, see [here](https://astros.github.io/book/index.html#licensing).
