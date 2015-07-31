use core::prelude::*;
use core::intrinsics;

use pm;
use hil::spi_master;

// Driver for the SPI hardware (seperate from the USARTS, described in chapter 26 of the
// datasheet)

/// The registers used to interface with the hardware
#[repr(C, packed)]
pub struct SPIRegisters {
    pub cr: u32, // 0x0
    pub mr: u32, // 0x4
    pub rdr: u32, // 0x8
    pub tdr: u32, // 0xC
    pub sr: u32, // 0x10
    ier: u32, // 0x14
    idr: u32, // 0x18
    imr: u32, // 0x1C
    reserved0: [u32; 4], // 0x20, 0x24, 0x28, 0x2C
    pub csr0: u32, // 0x30
    csr1: u32, // 0x34
    csr2: u32, // 0x38
    csr3: u32, // 0x3C
    reserved1: [u32; 41], // 0x40 - 0xE0
    pub wpcr: u32, // 0xE4
    wpsr: u32, // 0xE8
    reserved2: [u32; 3], // 0xEC - 0xF4
    features: u32, // 0xF8
    version: u32, // 0xFC
}

const BASE_ADDRESS: u32 = 0x40008000;

/// Values for selected peripherals
#[derive(Copy,Clone)]
pub enum Peripheral {
    Peripheral0 = 0b0000,
    Peripheral1 = 0b0001,
    Peripheral2 = 0b0011,
    Peripheral3 = 0b0111,
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
    pub regs: &'static mut SPIRegisters,
}

impl SPI {
    /// Creates a new SPI object, with peripheral 0 selected
    pub fn new() -> SPI {
        SPI {
            regs: unsafe{ intrinsics::transmute(BASE_ADDRESS) }
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
        let csr_mask: u32 = 0xFF00;
        // Clear, then write CSR bits
        csr |= !csr_mask;
        csr |= ((scbr & 0xFF) << 8);
        self.write_active_csr(csr);
    }

    /// Returns the currently active peripheral
    pub fn get_active_peripheral(&mut self) -> Peripheral {
        let mr = volatile!(self.regs.mr);
        let pcs = (mr >> 16) & 0xF;
        // Split into bits for matching
        let pcs_bits = ((pcs >> 3) & 1, (pcs >> 2) & 1, (pcs >> 1) & 1, pcs & 1);
        match pcs_bits {
            (_, _, _, 0) => Peripheral::Peripheral0,
            (_, _, 0, 1) => Peripheral::Peripheral1,
            (_, 0, 1, 1) => Peripheral::Peripheral2,
            (0, 1, 1, 1) => Peripheral::Peripheral3,
            _ => {
                // Invalid configuration
                // Reset to 0
                self.set_active_peripheral(Peripheral::Peripheral0);
                Peripheral::Peripheral0
            }
        }
    }
    /// Sets the active peripheral
    pub fn set_active_peripheral(&mut self, peripheral: Peripheral) {
        let mut mr = volatile!(self.regs.mr);
        // Clear and set MR.PCS
        mr |= !0x000F0000;
        mr |= (peripheral as u32) << 16;
        volatile!(self.regs.mr = mr);
    }

    /// Returns true if the SPI hardware is enabled, otherwise false
    pub fn is_enabled(&self) -> bool {
        ((volatile!(self.regs.sr) >> 16) & 1) == 1
    }

    /// Returns the value of CSR0, CSR1, CSR2, or CSR3, whichever corresponds to the active
    /// peripheral
    fn read_active_csr(&mut self) -> u32 {
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
        // Enable clock
        unsafe { pm::enable_clock(pm::Clock::PBA(pm::PBAClock::SPI)); }

        self.set_baud_rate(params.baud_rate);

        // Enable master mode
        let mut mode: u32 = 1;
        // Disable mode fault detection (open drain outputs do not seem to be supported)
        mode |= 1 << 4;
        volatile!(self.regs.mr = mode);
    }

    fn write_byte(&mut self, out_byte: u8) -> u8 {
        let tdr = out_byte as u32;
        volatile!(self.regs.tdr = tdr);
        // Wait for receive data register full
        while (volatile!(self.regs.sr & 1) != 1) {}
        // Return read value
        volatile!(self.regs.rdr) as u8
    }

    fn read_byte(&mut self) -> u8 {
        self.write_byte(0)
    }

    fn read(&mut self, buffer: &mut [u8]) {

    }

    fn write(&mut self, buffer: &[u8]) {

    }

    fn read_and_write(&mut self, read_buffer: &mut [u8], write_buffer: &[u8]) {

    }


    fn enable(&mut self) {
        volatile!(self.regs.cr = 0b1);
    }

    fn disable(&mut self) {
        volatile!(self.regs.cr = 0b10);
    }
}
