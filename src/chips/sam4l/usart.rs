use core::prelude::*;
use core::intrinsics;
use core::cmp::min;
use hil::{uart, spi_master, Controller};
use hil::uart::UART;
use hil::uart::Parity;

use nvic;
use pm::{self, Clock, PBAClock};

pub static mut USART3_INTERRUPT : bool = false;

#[repr(C, packed)]
struct UsartRegisters {
    cr: u32, // 0
    mr: u32, // 0x4
    ier: u32, // 0x8
    idr: u32, // 0xC
    imr: u32, // 0x10
    csr: u32, // 0x14
    rhr: u32, // 0x18
    thr: u32, // 0x1C
    brgr: u32, // 0x20
    rtor: u32, // 0x24
    ttgr: u32, // 0x28
    reserved0: [u32; 5], // 0x2C, 0x30, 0x34, 0x38, 0x3C
    fidi: u32, // 0x40
    ner: u32, // 0x44
    reserved1: u32, // 0x48
    ifr: u32, // 0x4C
    man: u32, // 0x50
    linmr: u32, // 0x54
    linir: u32, // 0x58
    linbrr: u32, // 0x5C
    reserved2: [u32; 34],
    wpmr: u32, // 0x60
    wpsr: u32, // 0x64
    version: u32 // 0x68
}

const SIZE: usize = 0x4000;
const BASE_ADDRESS: usize = 0x40024000;

#[derive(Copy,Clone)]
pub enum Location {
    USART0, USART1, USART2, USART3
}

pub struct USART {
    regs: &'static mut UsartRegisters,
    client: Option<&'static mut uart::Reader>,
    spi_client: Option<&'static mut spi_master::Reader>,
    clock: Clock,
    nvic: nvic::NvicIdx,
}

pub struct USARTParams {
    pub client: &'static mut uart::Reader,
    pub baud_rate: u32,
    pub data_bits: u8,
    pub parity: Parity
}

impl Controller for USART {
    type Config = USARTParams;

    fn configure(&mut self, params: USARTParams) {
        self.client = Some(params.client);
        let chrl = ((params.data_bits - 1) & 0x3) as u32;
        let mode = 0 /* mode */
            | 0 << 4 /*USCLKS*/
            | chrl << 6 /* Character Length */
            | (params.parity as u32) << 9 /* Parity */
            | 0 << 12; /* Number of stop bits = 1 */;

        self.enable_clock();
        self.set_baud_rate(params.baud_rate);
        self.set_mode(mode);
        volatile!(self.regs.ttgr = 4);
        self.enable_rx_interrupts();
    }
}

impl USART {
    pub fn new(location: Location) -> USART {
        let address = BASE_ADDRESS + (location as usize) * SIZE;

        let pba_clock = match location {
            Location::USART0 => PBAClock::USART0,
            Location::USART1 => PBAClock::USART1,
            Location::USART2 => PBAClock::USART2,
            Location::USART3 => PBAClock::USART3,
        };

        let nvic = match location {
            Location::USART0 => nvic::NvicIdx::USART0,
            Location::USART1 => nvic::NvicIdx::USART1,
            Location::USART2 => nvic::NvicIdx::USART2,
            Location::USART3 => nvic::NvicIdx::USART3
        };


        USART {
            regs: unsafe { intrinsics::transmute(address) },
            clock: Clock::PBA(pba_clock),
            nvic: nvic,
            client: None,
            spi_client: None,
        }
    }

    fn set_baud_rate(&mut self, baud_rate: u32) {

        let selected_clock = 48000000;

        let mode = volatile!(self.regs.mr);
        let synchronous = ((mode >> 8) & 1) != 0;
        let mode_only = mode & 0xF;
        let spi_mode = (mode_only == 0xE) || (mode_only == 0xF);
        if spi_mode || synchronous {
            // Use the SPI or synchronous formula
            // CD = selected clock / baud rate
            let cd = selected_clock / baud_rate;
            volatile!(self.regs.brgr = cd);
        }
        else {
            // Use the asynchronous non-SPI formula
            // CD = selected clock / (8 * baud rate * (2 - oversample))
            // Read oversample
            let oversample = (mode >> 19) & 1;
            let cd = selected_clock / (8 * baud_rate * (2 - oversample));
            volatile!(self.regs.brgr = cd);
        }
    }

    fn set_mode(&mut self, mode: u32) {
        volatile!(self.regs.mr = mode);
    }

    fn enable_clock(&self) {
        unsafe {
            pm::enable_clock(self.clock);
        }
    }

    fn enable_nvic(&self) {
        unsafe {
            nvic::enable(self.nvic);
        }
    }

    fn disable_nvic(&self) {
        unsafe {
            nvic::disable(self.nvic);
        }
    }

    #[inline(never)]
    pub fn enable_rx_interrupts(&mut self) {
        self.enable_nvic();
        volatile!(self.regs.ier = 1 as u32);
    }

    pub fn enable_tx_interrupts(&mut self) {
        self.enable_nvic();
        volatile!(self.regs.ier = 2 as u32);
    }

    pub fn disable_rx_interrupts(&mut self) {
        self.disable_nvic();
        volatile!(self.regs.idr = 1 as u32);
    }

    pub fn handle_interrupt(&mut self) {
        use hil::uart::UART;
        if self.rx_ready() {
            let c = volatile!(self.regs.rhr) as u8;
            match self.client {
                Some(ref mut client) => {client.read_done(c)},
                None => {}
            }
        }
    }

    pub fn reset_rx(&mut self) {
        volatile!(self.regs.cr = 1 << 2);
    }

}

impl uart::UART for USART {
    fn init(&mut self, params: uart::UARTParams) {
        let chrl = ((params.data_bits - 1) & 0x3) as u32;
        let mode = 0 /* mode */
            | 0 << 4 /*USCLKS*/
            | chrl << 6 /* Character Length */
            | (params.parity as u32) << 9 /* Parity */
            | 0 << 12; /* Number of stop bits = 1 */;

        self.enable_clock();
        self.set_baud_rate(params.baud_rate);
        self.set_mode(mode);
        volatile!(self.regs.ttgr = 4);
    }

    fn send_byte(&mut self, byte: u8) {
        while !self.tx_ready() {}
        volatile!(self.regs.thr = byte as u32);
    }

    fn rx_ready(&self) -> bool {
        volatile!(self.regs.csr) & 0b1 != 0
    }

    fn tx_ready(&self) -> bool {
        volatile!(self.regs.csr) & 0b10 != 0
    }


    fn read_byte(&self) -> u8 {
        while !self.rx_ready() {}
        volatile!(self.regs.rhr) as u8
    }

    fn enable_rx(&mut self) {
        volatile!(self.regs.cr = 1 << 4);
    }

    fn disable_rx(&mut self) {
        volatile!(self.regs.cr = 1 << 5);
    }

    fn enable_tx(&mut self) {
        volatile!(self.regs.cr = 1 << 6);
    }

    fn disable_tx(&mut self) {
        volatile!(self.regs.cr = 1 << 7);
    }

}

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern fn USART3_Handler() {
    volatile!(USART3_INTERRUPT = true);
    nvic::disable(nvic::NvicIdx::USART3);
}

// SPI master implementation
impl spi_master::SPI for USART {
    fn init(&mut self, params: spi_master::SPIParams) {
        let mut mode: u32 = 0xE; // SPI master mode
        // Data order (endianness)
        // TODO: Find where to set this
        // MR.MSBF is used for clock polarity in SPI mode, so it cannot be used for data order

        // Clock polarity
        match params.clock_polarity {
            spi_master::ClockPolarity::IdleHigh => mode = mode | (1 << 16),
            spi_master::ClockPolarity::IdleLow => { /* Defaults to 0 (IdleLow) */ },
        }
        // Clock phase
        match params.clock_phase {
            spi_master::ClockPhase::SampleTrailing => mode = mode | (1 << 8),
            spi_master::ClockPhase::SampleLeading => { /* Defaults to 0 */ },
        }
        // 8-bit character length (this includes the parity bit)
        mode = mode | (0b11 << 6);
        // No parity
        mode = mode | (0b100 << 9);
        // Drive clock pin
        mode = mode | (0b1 << 18);

        self.enable_clock();
        self.set_baud_rate(params.baud_rate);
        self.set_mode(mode);
        volatile!(self.regs.ttgr = 4);
        //self.enable_rx_interrupts();
    }
    fn write_byte(&mut self, out_byte: u8) -> u8 {
        // Wait for readiness
        while !self.tx_ready() {}
        // Load byte to write
        volatile!(self.regs.thr = out_byte as u32);

        // Does anything need to be done to actually do the read/write?

        // Return read value
        volatile!(self.regs.rhr) as u8
    }
    fn read_byte(&mut self) -> u8 {
        // Wait for readiness
        while !self.tx_ready() {}
        // Load byte to write (0)
        volatile!(self.regs.thr = 0 as u32);

        // Does anything need to be done to actually do the read/write?

        // Return read value
        volatile!(self.regs.rhr) as u8
    }

    fn read(&mut self, buffer: &mut [u8]) {
        // TODO: Make asynchronous
        for byte in buffer {
            *byte = self.read_byte();
        }
        // TODO: Callback
    }

    fn write(&mut self, buffer: &[u8]) {
        // TODO: Make asynchronous
        for &byte in buffer {
            self.write_byte(byte);
        }
        // TODO: Callback
    }

    fn read_and_write(&mut self, read_buffer: &mut [u8], write_buffer: &[u8]) {
        // TODO: Make asynchronous
        let count = min(read_buffer.len(), write_buffer.len());
        for i in 0..(count - 1) {
            read_buffer[i] = self.write_byte(write_buffer[i]);
        }
        // TODO: Callback
    }

    fn enable(&mut self) {
        volatile!(self.regs.cr = 1 << 4);
        volatile!(self.regs.cr = 1 << 6);
    }

    fn disable(&mut self) {
        volatile!(self.regs.cr = 1 << 5);
        volatile!(self.regs.cr = 1 << 7);
    }

}
