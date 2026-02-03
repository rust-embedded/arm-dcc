# `arm-dcc`

> a panic handler that uses [`arm-dcc`] to print the panic info over an Arm [Debug Communication Channel][dcc] (DCC) interface

This project is developed and maintained by the [Embedded Devices Working Group's Arm team][team].

See the docs at <https://docs.rs/panic-dcc>

[`arm-dcc`]: https://crates.io/crates/arm-dcc
[dcc]: https://developer.arm.com/docs/dui0471/latest/debug-communications-channel
[team]: https://github.com/rust-embedded/wg#the-arm-team

## Minimum Supported Rust Version (MSRV)

This crate is guaranteed to compile on stable Rust 1.59 and up. It *might*
compile with older versions but that may change in any new patch release.

## License

The `panic-dcc` crate is distributed under the terms of both the MIT license and
the Apache License (Version 2.0).

See [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT) for details.
