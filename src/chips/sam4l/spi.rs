use core::prelude::*;
use core::intrinsics;

use pm;
use hil::spi_master;
use hil::spi_master::Reader;

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

const BASE_ADDRESS: u32 = 0x40008000;

/// Values for selected peripherals
#[derive(Copy,Clone)]
pub enum Peripheral {
    Peripheral0,
    Peripheral1,
    Peripheral2,
    Peripheral3,
}

/// Stores the status of the hardware
#[derive(Debug)]
pub struct Status {
    /// If the SPI hardware is enabled
    enabled: bool,
    /// If the transmit register and shifter are empty
    tx_empty: bool,
    /// If a receive overrun, in which the receive register is set before the software has
    /// read it, has occurred
    overrun: bool,
    /// If the transmit register is empty
    tx_reg_empty: bool,
    /// If data has been received since the receive register was read
    rx_full: bool,
}

///
/// SPI implementation using the SPI hardware
/// Supports four peripherals. Each peripheral can have different settings.
///
/// The init, read, and write methods act on the currently selected peripheral.
/// The init method can be safely called more than once to configure different peripherals:
///
///     spi.set_active_peripheral(Peripheral::Peripheral0);
///     spi.init(/* Parameters for peripheral 0 */);
///     spi.set_active_peripheral(Peripheral::Peripheral1);
///     spi.init(/* Parameters for peripheral 1 */);
///
pub struct SPI {
    /// Registers
    regs: &'static mut SPIRegisters,
    /// Current active peripheral
    active: Peripheral,
    /// Client
    client: Option<&'static mut Reader>,
}

impl SPI {
    /// Creates a new SPI object, with peripheral 0 selected
    pub fn new() -> SPI {
        // Enable clock
        // Setting registers does not work correctly with the clock disabled
        unsafe { pm::enable_clock(pm::Clock::PBA(pm::PBAClock::SPI)); }
        SPI {
            regs: unsafe{ intrinsics::transmute(BASE_ADDRESS) },
            active: Peripheral::Peripheral0,
            client: None,
        }
    }

    /// Sets the approximate baud rate for the active peripheral
    ///
    /// Since the only supported baud rates are 48 MHz / n where n is an integer from 1 to 255,
    /// the exact baud rate may not be available. In that case, the next lower baud rate will be
    /// selected.
    ///
    /// The lowest available baud rate is 188235 baud.
    pub fn set_baud_rate(&mut self, mut rate: u32) {
        // Main clock frequency
        let clock = 48000000;

        if rate < 188235 {
            rate = 188235;
        }
        if rate > clock {
            rate = clock;
        }

        // Divide and truncate, resulting in a n value that might be too low
        let mut scbr = clock / rate;
        // If the division was not exact, increase the n to get a slower baud rate
        if clock % rate != 0 {
            scbr += 1;
        }
        let mut csr = self.read_active_csr();
        let csr_mask: u32 = 0xFFFF00FF;
        // Clear, then write CSR bits
        csr &= csr_mask;
        csr |= ((scbr & 0xFF) << 8);
        self.write_active_csr(csr);
    }

    /// Returns the currently active peripheral
    pub fn get_active_peripheral(&self) -> Peripheral {
        self.active
    }
    /// Sets the active peripheral
    pub fn set_active_peripheral(&mut self, peripheral: Peripheral) {
        self.active = peripheral;

        let peripheral_number: u32 = match peripheral {
            Peripheral::Peripheral0 => 0b0000,
            Peripheral::Peripheral1 => 0b0001,
            Peripheral::Peripheral2 => 0b0011,
            Peripheral::Peripheral3 => 0b0111,
        };

        let mut mr = volatile!(self.regs.mr);
        // Clear and set MR.PCS
        let pcs_mask: u32 = 0xFFF0FFFF;
        mr &= pcs_mask;
        mr |= peripheral_number << 16;
        volatile!(self.regs.mr = mr);
    }

    /// Returns true if the SPI hardware is enabled, otherwise false
    pub fn is_enabled(&self) -> bool {
        self.get_status().enabled
    }
    /// Returns the status of the SPI hardware
    pub fn get_status(&self) -> Status {
        let sr = volatile!(self.regs.sr);
        Status {
            enabled: ((sr >> 16) & 1) == 1,
            tx_empty: ((sr >> 9) & 1) == 1,
            overrun: ((sr >> 3) & 1) == 1,
            tx_reg_empty: ((sr >> 1) & 1) == 1,
            rx_full: (sr & 1) == 1,
        }
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
        self.client = params.client;
        self.set_baud_rate(params.baud_rate);

        let mut csr = self.read_active_csr();
        // Clock polarity
        match params.clock_polarity {
            spi_master::ClockPolarity::IdleHigh => csr |= 1, // Set bit 0
            spi_master::ClockPolarity::IdleLow => csr &= 0xFFFFFFFE, // Clear bit 0
        };
        // Clock phase
        match params.clock_phase {
            spi_master::ClockPhase::SampleTrailing => csr |= (1 << 1), // Set bit 1
            spi_master::ClockPhase::SampleLeading => csr &= 0xFFFFFFFD, // Clear bit 1
        }
        self.write_active_csr(csr);

        let mut mode = volatile!(self.regs.mr);
        // Enable master mode
        mode |= 1;
        // Disable mode fault detection (open drain outputs do not seem to be supported)
        mode |= 1 << 4;
        volatile!(self.regs.mr = mode);
    }

    fn write_byte(&mut self, out_byte: u8) -> u8 {
        let tdr = out_byte as u32;
        volatile!(self.regs.tdr = tdr);
        // Wait for receive data register full
        while (volatile!(self.regs.sr) & 1) != 1 {}
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
