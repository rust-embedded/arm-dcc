//! [Debug Communication Channel][dcc] (DCC) API
//!
//! [dcc]: https://developer.arm.com/products/software-development-tools/compilers/arm-compiler-5/docs/dui0471/latest/debug-communications-channel
//!
//! # Example
//!
//! ## Device side
//!
//! ``` no_run
//! use dcc::dprintln;
//!
//! fn main() {
//!     dprintln!("Hello, world!");
//! }
//! ```
//!
//! ## Host side
//!
//! ``` text
//! $ # XSDB = Xilinx System Debugger
//! $ xsdb
//!
//! (xsdb) # connect
//! (xsdb) conn
//!
//! (xsdb) # select a Cortex-R core
//! (xsdb) targets -set 0
//!
//! (xsdb) # hold the processor in reset state
//! (xsdb) rst -processor
//!
//! (xsdb) # load program
//! (xsdb) dow hello.elf
//!
//! (xsdb) # open a file
//! (xsdb) set f [open dcc.log w]
//!
//! (xsdb) # redirect DCC output to file handle `f`
//! (xsdb) readjtaguart -start -handle $f
//!
//! (xsdb) # start program execution
//! (xsdb) con
//! ```
//!
//! ``` text
//! $ # on another terminal
//! $ tail -f dcc.log
//! Hello, world!
//! ```
//!
//! # Supported Rust version
//!
//! - Rust >=1.31 when the target is one of the 4 ARMv7 Cortex-R targets.
//!
//! - All the other ARM targets require enabling the `inline-asm`, which requires a nightly
//! compiler.
//!
//! # Optional features
//!
//! ## `inline-asm`
//!
//! When this feature is enabled `dcc::write` is implemented using inline assembly (`asm!`) and
//! compiling this crate requires nightly. Note that this feature requires that the compilation
//! target is one of the 4 ARMv7 Cortex-R targets.
//!
//! When this feature is disabled `dcc::write` is implemented using FFI calls into an external
//! assembly file and compiling this crate works on stable and beta.

#![cfg_attr(feature = "inline-asm", feature(asm))]
#![deny(missing_docs)]
#![deny(warnings)]
#![no_std]

use core::fmt;

/// Macro for printing to the DCC
#[macro_export]
macro_rules! dprint {
    ($s:expr) => {
        $crate::write_str($s)
    };
    ($($tt:tt)*) => {
        $crate::write_fmt(format_args!($($tt)*))
    };
}

/// Macro for printing to the DCC, with a newline.
#[macro_export]
macro_rules! dprintln {
    () => {
        $crate::write_str("\n")
    };
    ($s:expr) => {
        $crate::write_str(concat!($s, "\n"))
    };
    ($s:expr, $($tt:tt)*) => {
        $crate::write_fmt(format_args!(concat!($s, "\n"), $($tt)*))
    };
}

/// Proxy struct that implements the `fmt::Write`
///
/// The main use case for this is using the `write!` macro
pub struct Writer;

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> Result<(), fmt::Error> {
        write_str(s);
        Ok(())
    }
}

/// Writes a single word to the DCC
#[allow(unused_variables)]
#[cfg(not(target_arch = "arm"))]
pub fn write(word: u32) {
    unimplemented!()
}

/// Writes a single word to the DCC
#[cfg(all(target_arch = "arm", feature = "inline-asm"))]
pub fn write(word: u32) {
    const W: u32 = 1 << 29;

    unsafe {
        let mut r: u32;
        loop {
            // busy wait until we can send data
            asm!("MRC p14, 0, $0, c0, c1, 0" : "=r"(r) : : : "volatile");
            if r & W == 0 {
                break;
            }
        }
        asm!("MCR p14, 0, $0, c0, c5, 0" : : "r"(word) : : "volatile");
    }
}

/// Writes a single word to the DCC
#[cfg(all(target_arch = "arm", not(feature = "inline-asm")))]
#[inline(always)]
pub fn write(word: u32) {
    extern "C" {
        fn __dcc_write(word: u32);
    }

    unsafe { __dcc_write(word) }
}

/// Writes the bytes to the DCC
///
/// NOTE: each byte will be word-extended before being `write`-n to the DCC
pub fn write_all(bytes: &[u8]) {
    bytes.iter().for_each(|byte| write(u32::from(*byte)))
}

#[doc(hidden)]
pub fn write_fmt(args: fmt::Arguments) {
    use core::fmt::Write;

    Writer.write_fmt(args).ok();
}

/// Writes the string to the DCC
pub fn write_str(string: &str) {
    write_all(string.as_bytes())
}
