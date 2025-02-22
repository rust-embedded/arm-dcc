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
//! - Rust >=1.59

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
        core::hint::spin_loop();
    }
}
