//! # Debug Communication Channel (DCC) API
//!
//! The Debug Communications Channel is a mechanism to get data from a *target*
//! and into a *host*, and vice-versa. It works over a JTAG interface and so
//! does not require a UART, or any dedicated I/O pins, and it does not stop the
//! CPU whilst being used (unlike [semihosting]).
//!
//! DCC was added to the Arm Architecture Reference Manual in [ARMv7][armv7].
//! Before that it was defined separately, usually as part of the Debug hardware
//! in a specific ARM processor's Technical Reference Manual (like for the
//! [ARM7TDMI][arm7tdmi]).
//!
//! This crate supports:
//!
//! * AArch64
//! * ARMv7 AArch32
//! * legacy ARM AArch32
//!
//! [semihosting]: https://crates.io/crates/semihosting
//! [armv7]:
//!     https://developer.arm.com/documentation/ddi0406/c/Debug-Architecture/The-Debug-Registers/Register-descriptions--in-register-order/DBGDSCR--Debug-Status-and-Control-Register?lang=en
//! [arm7tdmi]:
//!     https://developer.arm.com/documentation/ddi0210/c/Debug-Interface/Debug-Communications-Channel?lang=en
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
//! ### Xilinx System Debugger
//!
//! ```text
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
//! ### SEGGER J-Link
//!
//! Run J-Link:
//!
//! ```console
//! $ JLinkExe
//! SEGGER J-Link Commander V9.48 (Compiled Jun  3 2026 14:21:00)
//! DLL version V9.48, compiled Jun  3 2026 14:20:18
//!
//! Connecting to J-Link ...O.K.
//!
//! Type "connect" to establish a target connection, '?' for help
//! J-Link>device LPC2138
//! J-Link>si JTAG
//! J-Link>speed 1000
//! J-Link>jtagconf -1,-1
//! J-Link>connect
//! J-Link>r
//! J-Link>h
//! J-Link>loadfile target/file.hex
//! J-Link>go
//! J-Link>term
//! Please select terminal protocol:
//! B) Binary (raw) data (Default)
//! D) SEGGER DCC terminal
//! Protocol>D
//! Hello, world!
//! ```
//!
//! The `term` command activates the DCC terminal. Select 'D' for a DCC
//! terminal. We don't implement the SEGGER DCC Terminal protocol, but JLink
//! doesn't seem to mind.
//!
//! # Supported Rust version
//!
//! - Rust >=1.59
//!
//! # Optional features
//!
//! ## `nop`
//!
//! Turns `dcc::write` into a "no-operation" (not the instruction). This is
//! useful when the DCC is disabled as `dcc::write` blocks forever in that case.
//!
//! ## `legacy-mode`
//!
//! By default this crate uses the ARMv7 DCC registers (when `target_arch =
//! "arm"`). This feature selects the debug registers for the ARM7TDMI and
//! ARM9EJ-S instead.

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
        #[cfg(not(any(target_arch = "arm", target_arch = "aarch64")))]
        () => unimplemented!(),
        #[cfg(all(any(target_arch = "arm", target_arch = "aarch64"), feature = "nop"))]
        () => {}
        // See Arm ARM for R-profile AArch64 architecture, section E4.2 DCC and ITR registers for details
        #[cfg(all(target_arch = "aarch64", not(feature = "nop")))]
        () => {
            // "External Debug Status and Control Register (EDSCR) is architecturally mapped to
            // register MDSCR_EL1"
            const EDSCR_TXFULL: u64 = 1 << 29;

            // busy wait until the TX FIFO buffer is not full
            loop {
                let mut edscr: u64;
                // MDSCR = Monitor Debug System Control Register
                unsafe { core::arch::asm!("MRS {}, MDSCR_EL1", out(reg) edscr) }
                // if EDSCR_TXFULL is 0 we can proceed
                if edscr & EDSCR_TXFULL == 0 {
                    break;
                }
            }
            // DBGDTRTX = Debug Data Transfer Register, Transmit
            unsafe { core::arch::asm!("MSR DBGDTRTX_EL0, {}", in(reg) word as u64) }
        }
        #[cfg(all(
            target_arch = "arm",
            not(feature = "nop"),
            not(feature = "legacy-mode")
        ))]
        () => {
            // The DBGDSCR.TXfull bit
            const DBGDSCR_TXFULL: u32 = 1 << 29;

            unsafe {
                let mut r: u32;
                // busy wait until we can send data
                loop {
                    // Read DBGDSCR
                    core::arch::asm!("MRC p14, 0, {}, c0, c1, 0", out(reg) r);
                    if r & DBGDSCR_TXFULL == 0 {
                        break;
                    }
                }
                // ARMv7 DBGDTRTX
                core::arch::asm!("MCR p14, 0, {}, c0, c5, 0", in(reg) word);
            }
        }
        #[cfg(all(target_arch = "arm", not(feature = "nop"), feature = "legacy-mode"))]
        () => {
            const DCR_W: u32 = 1 << 1;

            // busy wait until we can send data
            unsafe {
                let mut r: u32;
                loop {
                    // Read Communications Channel Control Register
                    core::arch::asm!("MRC p14, 0, {}, c0, c0, 0", out(reg) r);
                    // "the processor must poll until W=0"
                    if r & DCR_W == 0 {
                        break;
                    }
                }
                core::arch::asm!("MCR p14, 0, {}, c1, c0, 0", in(reg) word);
            }
        }
    }
}

/// Writes the bytes to the DCC
///
/// NOTE: each byte will be word-extended before being `write`-n to the DCC
pub fn write_all(bytes: &[u8]) {
    // Send raw bytes
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
