use core::prelude::*;
use core::intrinsics;

//! Driver for the SPI hardware (seperate from the USARTS, described in chapter 26 of the
//! datasheet)

/// The registers used to interface with the hardware
#[repr(C, packed)]
struct SPIRegisters {
    cr: u32,
    mr: u32,
    rdr: u32,
    tdr: u32,
    sr: u32,
    ier: u32,
    idr: u32,
    imr: u32,
    reserved0: [u32, 5],
    csr0: u32,
    csr1: u32,
    csr2: u32,
    csr3: u32,
    reserved1: u32,
    
}
