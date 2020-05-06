//! Report panic messages to the host using the Debug Communication Channel (DCC)
//!
//! # Example
//!
//! ## Device side
//!
//! ``` ignore
//! use panic_dcc;
//!
//! fn main() {
//!     panic!("Oops");
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
//! panicked at 'Oops', src/hello.rs:4:4
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
//! When this feature is enabled `dcc::write` is implemented using inline assembly (`llvm_asm!`) and
//! compiling this crate requires nightly. Note that this feature requires that the compilation
//! target is one of the 4 ARMv7 Cortex-R targets.
//!
//! When this feature is disabled `dcc::write` is implemented using FFI calls into an external
//! assembly file and compiling this crate works on stable and beta.

#![deny(missing_docs)]
#![deny(warnings)]
#![no_std]

#[cfg(not(debug_assertions))]
use core::sync::atomic::{self, Ordering};
use core::{fmt::Write, panic::PanicInfo};

use arm_dcc::Writer;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // TODO uncomment
    // cortex_r::disable_fiq();
    // cortex_r::disable_irq();

    // NOTE this operation never returns `Err`
    writeln!(Writer, "{}", info).ok();

    loop {
        // NOTE the compiler_fence prevents this loop from turning into an abort instruction when
        // this crate is compiled with optimizations
        #[cfg(not(debug_assertions))]
        atomic::compiler_fence(Ordering::SeqCst)
    }
}
