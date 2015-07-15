#![crate_name = "rf230"]
#![crate_type = "rlib"]
#![feature(no_std,core,core_panic)]
#![no_std]
#![allow(dead_code)]

mod registers;
mod frame;

extern crate core;
use core::slice::SliceExt;
use core::ops::Drop;
extern crate common;
extern crate hil;
use hil::spi_master::*;
use hil::gpio::GPIOPin;

///
/// Implements a driver for the Atmel AT86RF230 2.4 GHz transceiver
///

/// 8 MHz, the maximum supported frequency (also defines the bit rate)
const BAUD_RATE: u32 = 8000000;
/// Bit ordering with the most significant bit first
const ORDERING: DataOrder = DataOrder::MSBFirst;
/// Clock polarity: Normally low
const POLARITY: ClockPolarity = ClockPolarity::IdleLow;
/// Clock phase: Sample on rising (leading) clock edge
const PHASE: ClockPhase = ClockPhase::SampleLeading;

/// Possible states in basic operating mode
#[allow(non_camel_case_types)]
#[derive(PartialEq)]
enum State {
    P_ON,
    BUSY_RX,
    BUSY_TX,
    RX_ON,
    TRX_OFF,
    PLL_ON,
    SLEEP,
    RX_ON_NOCLK,
    STATE_TRANSITION_IN_PROGRESS,
}

impl State {
    /// Converts a State into a u8 value as used by the hardware in the TRX_STATE register
    fn as_byte(&self) -> u8 {
        match *self {
            State::P_ON => 0x0,
            State::BUSY_RX => 0x1,
            State::BUSY_TX => 0x2,
            State::RX_ON => 0x6,
            State::TRX_OFF => 0x8,
            State::PLL_ON => 0x9,
            State::SLEEP => 0xF,
            State::RX_ON_NOCLK => 0x1C,
            State::STATE_TRANSITION_IN_PROGRESS => 0x1F,
        }
    }
}

/// Commands sent as (or part of) the first byte of an SPI session
enum SPICommand {
    /// Read a register
    RegisterRead = 0b10000000,
    /// Write a register
    RegisterWrite = 0b11000000,
    /// Read the framebuffer
    FrameBufferRead = 0b00100000,
    /// Write to the framebuffer
    FrameBufferWrite = 0b01100000,
    /// Read SRAM
    SRAMRead = 0b00000000,
    /// Write SRAM
    SRAMWrite = 0b01000000,
}

/// Sets the slave select pin low when constructed and returns it to high when destructed.
/// Allows the use of RAII to ensure that the slave select pin is set correctly.
struct SPITransaction<'a, GPIO: GPIOPin + 'a> {
    slave_select: &'a mut GPIO,
}
impl<'a, GPIO: GPIOPin> SPITransaction<'a, GPIO> {
    /// Creates a new transaction and sets the provided slave select output to low (active)
    pub fn new(slave_select: &'a mut GPIO) -> SPITransaction<'a, GPIO> {
        // Set low
        slave_select.enable_output();
        slave_select.clear();
        SPITransaction{ slave_select: slave_select }
    }
}
impl<'a, GPIO: GPIOPin> Drop for SPITransaction<'a, GPIO> {
    /// Sets the slave select output to high (inactive)
    fn drop(&mut self) {
        self.slave_select.set();
    }
}

///
/// Provides access to an RF230
///
pub struct RF230<S: SPI, GPIO: GPIOPin> {
    /// SPI communication
    spi: S,
    /// SPI slave select pin
    slave_select: GPIO,
    /// IRQ signal (for interrupts sent by the RF230 to the processor)
    // TODO: Verify that interrupts can be received with this
    irq: GPIO,
    /// Multi-purpose control signal (SLP_TR)
    control: GPIO,
    /// Reset signal
    reset: GPIO,
}

impl<S: SPI, GPIO: GPIOPin> RF230<S, GPIO> {
    /// Creates an RF230 object using the provided SPI object and input/output pins
    pub fn new(mut spi: S, mut slave_select: GPIO, irq: GPIO, control: GPIO, reset: GPIO) -> RF230<S, GPIO> {

        // Set slave select high (not selected)
        slave_select.enable_output();
        slave_select.set();

        // Set up SPI
        spi.init(SPIParams{ baud_rate: BAUD_RATE, data_order: ORDERING, clock_polarity: POLARITY, clock_phase: PHASE });

        // TODO: Reset

        RF230{ spi: spi, slave_select: slave_select, irq: irq, control: control, reset: reset }
    }

    /// Returns the RF230 part number
    pub fn get_part_number(&mut self) -> u8 {
        self.read_register(registers::PART_NUM)
    }
    /// Returns the RF230 version number
    pub fn get_version_number(&mut self) -> u8 {
        self.read_register(registers::VERSION_NUM)
    }
    /// Returns the lower 16 bits of the RF230's 32-bit JEDEC manufacturer ID
    pub fn get_manufacturer_id(&mut self) -> u16 {
        let lower_bits = self.read_register(registers::MAN_ID_0);
        let upper_bits = self.read_register(registers::MAN_ID_1);
        (lower_bits as u16) | ((upper_bits as u16) << 8)
    }

    /// Responds to an interrupt from the RF230
    pub fn handle_interrupt(&mut self) {
        let status = self.read_register(registers::IRQ_STATUS);
        if ((status >> 7) & 1) == 1 {
            // Low supply voltage
            // TODO
        }
        else if ((status >> 6) & 1) == 1 {
            // Frame buffer access violation
            // TODO
        }
        else if ((status >> 3) & 1) == 1 {
            // Send or receive completed
            // TODO
        }
        else if ((status >> 2) & 1) == 1 {
            // Receive started
            // TODO
        }
        else if ((status >> 1) & 1) == 1 {
            // PLL unlock
            // TODO
        }
        else if (status & 1) == 1 {
            // PLL lock
            // TODO
        }
    }

    /// Writes the specified value to the specified register
    fn write_register(&mut self, register: registers::Register, value: u8) {
        // Byte 1: 1, 0, register address
        let byte1 = SPICommand::RegisterWrite as u8 | register.address;
        let byte2 = register.clean_for_write(value);
        // Send two bytes, ignore returned values

        let _transaction = SPITransaction::new(&mut self.slave_select);
        self.spi.write(byte1);
        self.spi.write(byte2);
    }
    /// Reads the specified register and returns its value
    fn read_register(&mut self, register: registers::Register) -> u8 {
        // Byte 1: 1, 0, register address
        let byte1 = SPICommand::RegisterRead as u8 | register.address;
        // Send the byte with the register address, read the value in the next byte
        let _transaction = SPITransaction::new(&mut self.slave_select);
        self.spi.write(byte1);
        let result = self.spi.read();
        result
    }

    /// Writes a frame to the framebuffer
    /// The data must not contain more than 127 bytes. If the data contains more than 127 bytes,
    /// the frame will not be transmitted.
    fn write_frame_buffer(&mut self, data: &[u8]) {
        // TOOD: Redo

        let length = data.len() as u8;
        let _transaction = SPITransaction::new(&mut self.slave_select);
        // Write command
        self.spi.write(SPICommand::FrameBufferWrite as u8);
        // Write length
        self.spi.write(length);
        // Write data
        for &byte in data {
            self.spi.write(byte);
        }
    }

    /// Reads a frame from the framebuffer
    /// Returns the frame that was read.
    fn read_frame_buffer(&mut self) {

    }

    /// Reads `data.len()` bytes from the RF230 SRAM starting at the specified address and stores
    /// them in `data`
    fn read_sram(&mut self, address: u8, data: &mut [u8]) {
        let _transaction = SPITransaction::new(&mut self.slave_select);
        // Send read request
        self.spi.write(SPICommand::SRAMRead as u8);
        // Send address
        self.spi.write(address);
        // Read data
        for index in 0..data.len() {
            data[index] = self.spi.read();
        }
    }

    /// Writes `data.len()` bytes from `data` to the RF230 SRAM starting at the specified address
    fn write_sram(&mut self, address: u8, data: &[u8]) {
        let _transaction = SPITransaction::new(&mut self.slave_select);
        // Send write command
        self.spi.write(SPICommand::SRAMWrite as u8);
        self.spi.write(address);
        for &byte in data {
            self.spi.write(byte);
        }
    }

    /// Returns the current state of the RF230
    fn get_state(&mut self) -> State {
        match self.read_register(registers::TRX_STATUS) {
            0x0 => State::P_ON,
            0x1 => State::BUSY_RX,
            0x2 => State::BUSY_TX,
            0x6 => State::RX_ON,
            0x8 => State::TRX_OFF,
            0x9 => State::PLL_ON,
            0xF => State::SLEEP,
            0x1C => State::RX_ON_NOCLK,
            0x1F => State::STATE_TRANSITION_IN_PROGRESS,
            _ => {
                static _MSG_FILE_LINE: (&'static str, &'static str, u32) = ("Unexpected state", file!(), line!());
                ::core::panicking::panic(&_MSG_FILE_LINE)
            }
        }
    }

    /// Writes the specified state to the TRX_STATE register.
    /// Note that this is is only valid for some state transitions, as defined in the state diagram.
    fn write_state_register(&mut self, state: State) {
        self.write_register(registers::TRX_STATUS, state.as_byte());
    }

    /// Sets the state of the RF230 to State::RX_ON
    fn set_state_rx_on(&mut self) {
        loop {
            let state = self.get_state();
            match state {
                State::P_ON => self.write_state_register(State::TRX_OFF),
                State::BUSY_RX => { /* Wait for receive completion */ },
                State::BUSY_TX => { /* Wait for send completion */ },
                State::TRX_OFF => self.write_state_register(State::RX_ON),
                State::PLL_ON => self.write_state_register(State::RX_ON),
                State::SLEEP => self.control.clear(), // Set SLP_TR low
                State::RX_ON_NOCLK => self.control.clear(), // Set SLP_TR low
                State::STATE_TRANSITION_IN_PROGRESS => { /* Wait for state transition to end */ },

                State::RX_ON => return,
            }
        }
    }

    /// Sets the state of the RF230 to State::PLL_ON (used to send frames)
    fn set_state_pll_on(&mut self) {
        loop {
            let state = self.get_state();
            match state {
                State::P_ON => self.write_state_register(State::TRX_OFF),
                State::BUSY_RX => { /* Wait for receive completion */ },
                State::BUSY_TX => { /* Wait for send completion */ },
                State::TRX_OFF => self.write_state_register(State::PLL_ON),
                State::RX_ON => self.write_state_register(State::PLL_ON),
                State::SLEEP => self.control.clear(), // Set SLP_TR low
                State::RX_ON_NOCLK => self.control.clear(), // Set SLP_TR low
                State::STATE_TRANSITION_IN_PROGRESS => { /* Wait for state transition to end */ },

                State::PLL_ON => return,
            }
        }
    }
}
