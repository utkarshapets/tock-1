#![crate_name = "drivers"]
#![crate_type = "rlib"]
#![feature(const_fn, raw)]
#![no_std]

extern crate common;
extern crate hil;

pub mod console;
pub mod gpio;
pub mod timer;
pub mod tmp006;
#[allow(non_snake_case)] // rust being stupid
pub mod accel_FXOS8700CQ;
pub mod virtual_alarm;
