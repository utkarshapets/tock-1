use core::prelude::*;
use core::intrinsics;

use hil::spi_master;

// Driver for the SPI hardware (seperate from the USARTS, described in chapter 26 of the
// datasheet)

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
    reserved2: [u32; 3], // 0xEC - 0xF4
    features: u32, // 0xF8
    version: u32, // 0xFC
}

const SPI_BASE_ADDRESS: u32 = 0x40008000;

/// Values for selected peripherals
#[derive(Copy,Clone)]
pub enum Peripheral {
    Peripheral0,
    Peripheral1,
    Peripheral2,
    Peripheral3,
}

///
/// SPI implementation using the SPI hardware
/// Supports four peripherals. Each peripheral can have different settings.
///
/// The init, read, and write methods act on the currently selected peripheral.
///
///
///
pub struct SPI {
    /// Registers
    regs: &'static mut SPIRegisters,
    /// The current active peripheral
    active: Peripheral,
}

impl SPI {
    /// Creates a new SPI object, with peripheral 0 selected
    pub fn new() -> SPI {
        SPI {
            regs: unsafe{ intrinsics::transmute(SPI_BASE_ADDRESS) },
            active: Peripheral::Peripheral0
        }
    }

    /// Sets the approximate baud rate for the active peripheral
    ///
    /// Since the only supported baud rates are 48 MHz / n where n is an integer from 1 to 255,
    /// the exact baud rate may not be available. In that case, the next lower baud rate will be
    /// selected.
    pub fn set_baud_rate(&mut self, rate: u32) {
        // Main clock frequency
        let clock = 48000000;
        // Divide and truncate, resulting in a n value that might be too low
        let mut scbr = clock / rate;
        // If the division was not exact, increase the n to get a slower baud rate
        if clock % rate != 0 {
            scbr += 1;
        }
        let mut csr = self.read_active_csr();
        let csr_mask: u32 = 0b00000000000000001111111100000000;
        // Clear, then write CSR bits
        csr |= !csr_mask;
        csr |= ((scbr & 0b11111111) << 8);
        self.write_active_csr(csr);
    }

    /// Returns the currently active peripheral
    pub fn get_active_peripheral(&self) -> Peripheral {
        self.active
    }
    /// Sets the active peripheral
    pub fn set_active_peripheral(&mut self, peripheral: Peripheral) {
        self.active = peripheral;
    }

    /// Returns the value of CSR0, CSR1, CSR2, or CSR3, whichever corresponds to the active
    /// peripheral
    fn read_active_csr(&self) -> u32 {
        match self.get_active_peripheral() {
            Peripheral::Peripheral0 => volatile!(self.regs.csr0),
            Peripheral::Peripheral1 => volatile!(self.regs.csr1),
            Peripheral::Peripheral2 => volatile!(self.regs.csr2),
            Peripheral::Peripheral3 => volatile!(self.regs.csr3),
        }
    }
    /// Sets the value of CSR0, CSR1, CSR2, or CSR3, whichever corresponds to the active
    /// peripheral
    fn write_active_csr(&mut self, value: u32) {
        match self.get_active_peripheral() {
            Peripheral::Peripheral0 => volatile!(self.regs.csr0 = value),
            Peripheral::Peripheral1 => volatile!(self.regs.csr1 = value),
            Peripheral::Peripheral2 => volatile!(self.regs.csr2 = value),
            Peripheral::Peripheral3 => volatile!(self.regs.csr3 = value),
        };
    }
}

impl spi_master::SPI for SPI {
    fn init(&mut self, params: spi_master::SPIParams) {
        self.set_baud_rate(params.baud_rate);
    }

    fn write_byte(&mut self, out_byte: u8) -> u8 {
        0
    }

    fn read_byte(&mut self) -> u8 {
        0
    }

    fn read(&mut self, buffer: &mut [u8]) {

    }

    fn write(&mut self, buffer: &[u8]) {

    }

    fn read_and_write(&mut self, read_buffer: &mut [u8], write_buffer: &[u8]) {

    }


    fn enable_rx(&mut self) {

    }

    fn disable_rx(&mut self) {

    }

    fn enable_tx(&mut self) {

    }

    fn disable_tx(&mut self) {

    }
}
