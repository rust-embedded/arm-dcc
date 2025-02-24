//! [Debug Communication Channel][dcc] (DCC) API
//!
//! [dcc]: https://developer.arm.com/products/software-development-tools/compilers/arm-compiler-5/docs/dui0471/latest/debug-communications-channel
//!
//! # Example
//!
//! ## Device side
//!
//! ``` no_run
//! use arm_dcc::dprintln;
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
//! xsdb% # connect
//! xsdb% conn
//!
//! xsdb% # select a Cortex-R core
//! xsdb% targets -set 0
//!
//! xsdb% # hold the processor in reset state
//! xsdb% rst -processor
//!
//! xsdb% # load program
//! xsdb% dow hello.elf
//!
//! xsdb% # open a file
//! xsdb% set f [open dcc.log w]
//!
//! xsdb% # redirect DCC output to file handle `f`
//! xsdb% readjtaguart -start -handle $f
//!
//! xsdb% # start program execution
//! xsdb% con
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
//! - Rust >=1.59
//!
//! # Optional features
//!
//! ## `nop`
//!
//! Turns `dcc::write` into a "no-operation" (not the instruction). This is useful when the DCC is
//! disabled as `dcc::write` blocks forever in that case.

#![deny(missing_docs)]
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
///
/// **NOTE:** This operation is blocking
#[allow(unused_variables)]
#[inline(always)]
pub fn write(word: u32) {
    match () {
        #[cfg(not(target_arch = "arm"))]
        () => unimplemented!(),
        #[cfg(all(target_arch = "arm", feature = "nop"))]
        () => {}
        #[cfg(all(target_arch = "arm", not(feature = "nop")))]
        () => {
            const W: u32 = 1 << 29;

            unsafe {
                let mut r: u32;
                loop {
                    // busy wait until we can send data
                    core::arch::asm!("MRC p14, 0, {}, c0, c1, 0", out(reg) r);
                    if r & W == 0 {
                        break;
                    }
                }
                core::arch::asm!("MCR p14, 0, {}, c0, c5, 0", in(reg) word);
            }
        }
    }
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

#[cfg(target_arch = "arm")]
core::arch::global_asm!(
    r#"
    // Routine for putting data in the DCC register
    .section .text.__dcc_write
    .global __dcc_write
__dcc_write:
1:  mrc     p14, 0, r1, c0, c1, 0
    tst     r1, #536870912      /* 0x20000000 */
    bne     1b
    mcr     p14, 0, r0, c0, c5, 0
    bx      lr
    "#
);
