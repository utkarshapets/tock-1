#![crate_name = "platform"]
#![crate_type = "rlib"]
#![no_std]
#![feature(core,no_std)]

extern crate core;
extern crate common;
extern crate drivers;
extern crate hil;
extern crate sam4l;

use core::prelude::*;
use hil::adc::AdcInternal;
use hil::Controller;
use hil::spi_master::SPI;

pub static mut ADC  : Option<sam4l::adc::Adc> = None;
pub static mut CHIP : Option<sam4l::Sam4l> = None;

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
    chip: &'static mut sam4l::Sam4l,
    console: drivers::console::Console<sam4l::usart::USART>,
    gpio: drivers::gpio::GPIO<[&'static mut hil::gpio::GPIOPin; 14]>,
    tmp006: drivers::tmp006::TMP006<sam4l::i2c::I2CDevice>,
    // SPI for testing
    pub spi_master: &'static mut sam4l::usart::USART,
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
    CHIP = Some(sam4l::Sam4l::new());
    let chip = CHIP.as_mut().unwrap();

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
        // SPI using USART 2
        spi_master: &mut chip.usarts[0],
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


    // SPI test
    // Configure pins

    // PB14 as RXD
    chip.pb14.configure(Some(sam4l::gpio::PeripheralFunction::A));
    // PB15 as TXD
    chip.pb15.configure(Some(sam4l::gpio::PeripheralFunction::A));
    // PB11 as TXD
    chip.pb11.configure(Some(sam4l::gpio::PeripheralFunction::A));
    // PB13 as CLK
    chip.pb13.configure(Some(sam4l::gpio::PeripheralFunction::A));
    // PB12 as RTS
    chip.pb12.configure(Some(sam4l::gpio::PeripheralFunction::A));

    firestorm.console.putstr("Configuring SPI...");
    firestorm.spi_master.init(hil::spi_master::SPIParams {
        baud_rate: 9600,
        data_order: hil::spi_master::DataOrder::LSBFirst,
        clock_polarity: hil::spi_master::ClockPolarity::IdleHigh,
        clock_phase: hil::spi_master::ClockPhase::SampleLeading,
    });
    firestorm.console.putstr(" done.\nEnabling TX...");
    firestorm.spi_master.enable_tx();
    firestorm.console.putstr(" done.\nEnabling RX...");
    firestorm.spi_master.enable_rx();
    firestorm.console.putstr(" done.\nWriting something...");
    firestorm.spi_master.write(&[0b10101010], || {});
    firestorm.console.putstr(" done.");
    // End SPI test

    firestorm
}
