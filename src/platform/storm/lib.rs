#![crate_name = "platform"]
#![crate_type = "rlib"]
#![no_std]
#![feature(core,no_std)]

extern crate core;
extern crate common;
extern crate drivers;
extern crate hil;
extern crate sam4l;
extern crate rf230;

use core::prelude::*;
use hil::adc::AdcInternal;
use hil::Controller;
use hil::spi_master::SPI;
use sam4l::*;

pub static mut ADC  : Option<adc::Adc> = None;

pub struct TestRequest {
  chan: u8
}

impl hil::adc::Request for TestRequest {
  fn read_done(&mut self, val: u16) {
    // Do something with this reading!
  }
  fn channel(&mut self) -> u8 {
    self.chan
  }
}

pub static mut REQ: TestRequest = TestRequest {
  chan: 0
};


pub static mut FIRESTORM : Option<Firestorm> = None;

pub struct Firestorm {
    chip: &'static mut chip::Sam4l,
    console: drivers::console::Console<sam4l::usart::USART>,
    gpio: drivers::gpio::GPIO<[&'static mut hil::gpio::GPIOPin; 14]>,
    tmp006: drivers::tmp006::TMP006<sam4l::i2c::I2CDevice>,
    // Non-USART SPI for testing
    pub spi: &'static mut sam4l::spi::SPI,
    // RF230 for testing
    pub rf230: rf230::RF230<sam4l::gpio::GPIOPin>,
}

impl Firestorm {
    pub unsafe fn service_pending_interrupts(&mut self) {
        self.chip.service_pending_interrupts()
    }

    pub fn has_pending_interrupts(&mut self) -> bool {
        self.chip.has_pending_interrupts()
    }

    pub fn with_driver<F, R>(&mut self, driver_num: usize, mut f: F) -> R where
            F: FnMut(Option<&mut hil::Driver>) -> R {

        f(match driver_num {
            0 => Some(&mut self.console),
            1 => Some(&mut self.gpio),
            2 => Some(&mut self.tmp006),
            _ => None
        })
    }

}

pub unsafe fn init() -> &'static mut Firestorm {
    chip::CHIP = Some(chip::Sam4l::new());
    let chip = chip::CHIP.as_mut().unwrap();

    FIRESTORM = Some(Firestorm {
        chip: chip,
        console: drivers::console::Console::new(&mut chip.usarts[3]),
        gpio: drivers::gpio::GPIO::new(
            [ &mut chip.pc10, &mut chip.pc19, &mut chip.pc13
            , &mut chip.pa09, &mut chip.pa17, &mut chip.pc20
            , &mut chip.pa19, &mut chip.pa14, &mut chip.pa16
            , &mut chip.pa13, &mut chip.pa11, &mut chip.pa10
            , &mut chip.pa12, &mut chip.pc09]),
        tmp006: drivers::tmp006::TMP006::new(&mut chip.i2c[2]),
        spi: &mut chip.spi,

        // RF230 connections:
        // SLP_TR PC14
        // RSTN   PC15
        // SELN   PC01 (Slave select 3 from the SPI hardware when peripheral A is selected)
        // IRQ    PA20 (EIC EXTINT5 when peripheral A is selected)
        rf230: rf230::RF230::new(&mut chip.spi, &mut chip.pc01, &mut chip.pa20, &mut chip.pc14, &mut chip.pc15),
    });

    let firestorm : &'static mut Firestorm = FIRESTORM.as_mut().unwrap();

    chip.usarts[3].configure(sam4l::usart::USARTParams {
        client: &mut firestorm.console,
        baud_rate: 115200,
        data_bits: 8,
        parity: hil::uart::Parity::None
    });

    chip.pb09.configure(Some(sam4l::gpio::PeripheralFunction::A));
    chip.pb10.configure(Some(sam4l::gpio::PeripheralFunction::A));

    chip.pa21.configure(Some(sam4l::gpio::PeripheralFunction::E));
    chip.pa22.configure(Some(sam4l::gpio::PeripheralFunction::E));

    ADC = Some(sam4l::adc::Adc::new());
    let adc = ADC.as_mut().unwrap();
    adc.initialize();
    REQ.chan = 1;
    adc.sample(&mut REQ);

    firestorm.console.initialize();


    // Pin note: SPI_CS2 and SPI_CS1 on the Firestorm schematic are swapped.
    // The primary function column of the Storm pin reference is incorrect,
    // but the A column is correct.
    // PC02 is chip select 1, and PC00 is chip select 2.

    firestorm
}
