#![crate_name = "process"]
#![crate_type = "rlib"]
#![no_std]
#![feature(core,no_std,core_prelude,raw,core_intrinsics,core_slice_ext)]

extern crate core;
extern crate common;

pub mod process;
pub mod callback;

pub use process::{Process,State};
pub use callback::Callback;
