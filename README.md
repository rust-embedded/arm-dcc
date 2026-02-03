# `arm-dcc`

> [Debug Communication Channel][dcc] (DCC) API

This project is developed and maintained by the [Embedded Devices Working Group's Arm team][team].

[team]: https://github.com/rust-embedded/wg#the-arm-team
[dcc]: https://developer.arm.com/docs/dui0471/latest/debug-communications-channel

## Contents

This repository contains two crates:

* [`arm-dcc`](./arm-dcc/) - a library for sending formatted text to the Arm DCC interface
* [`panic-dcc`](./panic-dcc/) - a library which implements a panic handler that uses `arm-dcc` to print the panic info

## Minimum Supported Rust Version (MSRV)

This crate is guaranteed to compile on stable Rust 1.59 and up. It *might*
compile with older versions but that may change in any new patch release.

## License

See the individual crates for licence details.

## Code of Conduct

Contribution to this crate is organized under the terms of the [Rust Code of
Conduct][CoC], the maintainer of this crate, the [Embedded Devices Working
Group's Arm team][team], promises to intervene to uphold that code of conduct.

[CoC]: CODE_OF_CONDUCT.md
