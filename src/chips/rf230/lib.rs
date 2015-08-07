#![crate_name = "rf230"]
#![crate_type = "rlib"]
#![feature(no_std,core)]
#![no_std]
#![allow(dead_code)]

pub mod registers;

extern crate core;
extern crate common;
extern crate hil;
extern crate sam4l;
use sam4l::eic;
use core::prelude::*;
use hil::spi_master::*;
use hil::gpio::GPIOPin;
use hil::ieee802154;

///
/// Implements a driver for the Atmel AT86RF230 2.4 GHz transceiver
///

/// 8 MHz, the maximum supported frequency (also defines the bit rate)
const BAUD_RATE: u32 = 4000000;
/// Bit ordering with the most significant bit first
const ORDERING: DataOrder = DataOrder::MSBFirst;
/// Clock polarity: Normally low
const POLARITY: ClockPolarity = ClockPolarity::IdleLow;
/// Clock phase: Sample on rising (leading) clock edge
const PHASE: ClockPhase = ClockPhase::SampleLeading;

/// Possible states in basic operating mode
#[allow(non_camel_case_types)]
#[derive(PartialEq)]
pub enum State {
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

///
/// Provides access to an RF230
///
pub struct RF230<GPIO: 'static + GPIOPin> {
    /// SPI communication
    spi: &'static mut SPI,
    /// Multi-purpose control signal (SLP_TR)
    control: &'static mut GPIO,
    /// Reset signal
    reset: &'static mut GPIO,

    /// Reader
    client: Option<&'static mut ieee802154::Reader>,
}

impl<GPIO: 'static + GPIOPin> RF230<GPIO> {
    /// Creates an RF230 object using the provided SPI object and input/output pins
    ///
    /// `spi` is the SPI object used to communicate with the RF230. It must set its slave select
    /// pin automatically. The corresponding pins must have already been configured with the
    /// correct peripheral.
    ///
    /// `irq` is an external interrupt that the RF230's IRQ pin. The corresponding pin must have
    /// already been configured with the correct EIC peripheral.
    ///
    /// `control` and `reset` are GPIO pins connected to the RF230's SLP_TR and RST pins.
    ///
    pub fn new(mut spi: &'static mut SPI, irq: eic::Interrupt, control: &'static mut GPIO, reset: &'static mut GPIO) -> RF230<GPIO> {

        let rf230 = RF230{ spi: spi, control: control, reset: reset, client: None };
        rf230
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
    pub fn write_register(&mut self, register: registers::Register, value: u8) {
        let bytes = [(SPICommand::RegisterWrite as u8) | register.address,
                    register.clean_for_write(value) ];
        // Send two bytes, ignore returned values
        self.spi.write(&bytes);
    }
    /// Reads the specified register and returns its value
    pub fn read_register(&mut self, register: registers::Register) -> u8 {
        // Byte 1: 1, 0, register address
        let bytes: [u8; 2] = [SPICommand::RegisterRead as u8 | register.address, 0x0];
        let mut read_bytes: [u8; 2] = [0x0; 2];
        // Send the byte with the register address, read the value in the next byte
        self.spi.read_and_write(&mut read_bytes, &bytes);
        let result = read_bytes[1];
        result
    }

    /// Writes a frame to the framebuffer
    /// The data must not contain more than 127 bytes. If the data contains more than 127 bytes,
    /// the frame will not be transmitted.
    fn write_frame_buffer(&mut self, data: &[u8]) {
        // TOOD: Redo

        let length = data.len() as u8;
        // Write command
        self.spi.write_byte(SPICommand::FrameBufferWrite as u8);
        // Write length
        self.spi.write_byte(length);
        // Write data
        for &byte in data {
            self.spi.write_byte(byte);
        }
    }

    /// Reads a frame from the framebuffer
    /// Returns the frame that was read.
    fn read_frame_buffer(&mut self) {
        // TODO
    }

    /// Reads `data.len()` bytes from the RF230 SRAM starting at the specified address and stores
    /// them in `data`
    fn read_sram(&mut self, address: u8, data: &mut [u8]) {
        // Send read request
        self.spi.write_byte(SPICommand::SRAMRead as u8);
        // Send address
        self.spi.write_byte(address);
        // Read data
        for index in 0..data.len() {
            data[index] = self.spi.read_byte();
        }
    }

    /// Writes `data.len()` bytes from `data` to the RF230 SRAM starting at the specified address
    fn write_sram(&mut self, address: u8, data: &[u8]) {
        // Send write command
        self.spi.write_byte(SPICommand::SRAMWrite as u8);
        self.spi.write_byte(address);
        for &byte in data {
            self.spi.write_byte(byte);
        }
    }

    /// Returns the current state of the RF230
    pub fn get_state(&mut self) -> State {
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

impl<GPIO: 'static + GPIOPin> Reader for RF230<GPIO> {
    fn write_done(&mut self) {

    }
    fn read_done(&mut self) {

    }
    fn read_write_done(&mut self) {

    }
}

impl<GPIO: 'static + GPIOPin> ieee802154::Transceiver for RF230<GPIO> {
    fn init(&mut self, params: ieee802154::Params) {

        // Set up SPI
        self.spi.init(SPIParams{ baud_rate: BAUD_RATE, data_order: ORDERING, clock_polarity: POLARITY, clock_phase: PHASE, client: None });
        self.spi.enable();

        // Enable pins
        self.control.enable_output();
        self.control.set();
        self.reset.enable_output();
        self.reset.set();

    }

    fn enable_rx(&mut self) {

    }
    fn disable_rx(&mut self) {

    }

    fn send(&mut self, frame: ieee802154::Frame) {

    }
}
