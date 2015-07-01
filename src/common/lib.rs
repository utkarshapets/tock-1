//! A library for common operations in the Tock OS.

#![crate_name = "common"]
#![crate_type = "rlib"]
#![feature(core,no_std,core_prelude,core_slice_ext)]
#![no_std]

extern crate core;
extern crate support;

pub mod shared;
pub mod ring_buffer;
