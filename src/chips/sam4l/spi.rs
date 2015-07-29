use core::prelude::*;
use core::intrinsics;

//! Driver for the SPI hardware (seperate from the USARTS, described in chapter 26 of the
//! datasheet)

/// The registers used to interface with the hardware
#[repr(C, packed)]
struct SPIRegisters {
    cr: u32, // 0x0
    mr: u32, // 0x4
    rdr: u32, // 0x8
    tdr: u32, // 0xC
    sr: u32, // 0x10
    ier: u32, // 0x14
    idr: u32, // 0x18
    imr: u32, // 0x1C
    reserved0: [u32; 4], // 0x20, 0x24, 0x28, 0x2C
    csr0: u32, // 0x30
    csr1: u32, // 0x34
    csr2: u32, // 0x38
    csr3: u32, // 0x3C
    reserved1: [u32; 41], // 0x40 - 0xE0
    wpcr: u32, // 0xE4
    wpsr: u32, // 0xE8
    reserved2: [u32; 3] // 0xEC - 0xF4
    features: u32, // 0xF8
    version: u32, // 0xFC
}

pub struct SPI {
    
}
