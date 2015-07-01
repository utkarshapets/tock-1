#![crate_name = "drivers"]
#![crate_type = "rlib"]
#![feature(core,no_std,core_prelude,core_slice_ext,core_str_ext)]
#![no_std]

extern crate core;
extern crate hil;

mod std {
   pub use core::*;
}

pub mod gpio;
pub mod console;
pub mod tmp006;
