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
use hil::gpio::GPIOPin;

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
    // Non-USART SPI for testing
    pub spi: &'static mut sam4l::spi::SPI,
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

fn print_binary(value: u32) -> [u8; 32] {
    let mut string: [u8; 32] = ['0' as u8; 32];
    for i in 0..31 {
        let bit = (value >> i) & 1;
        if bit == 1 {
            string[31 - i] = '1' as u8;
        }
    }
    string
}

fn print_register<U: hil::uart::UART>(console: &mut drivers::console::Console<U>, name: &str, reg: &'static u32) {
    use core::intrinsics;
    console.putstr(name);
    console.putstr(": ");
    let bits = print_binary( unsafe { intrinsics::volatile_load(reg) });
    for &bit in bits.iter() {
        console.putbytes(&[bit]);
    }
    console.putstr("\n");
}

pub unsafe fn init() -> &'static mut Firestorm {
    use core::intrinsics;
    use sam4l::pm;

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
        // SPI using USART 1
        spi_master: &mut chip.usarts[1],
        spi: &mut chip.spi,
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



    // Set pins for SPI testing
    // PC06 as SCLK
    chip.pc06.configure(Some(sam4l::gpio::PeripheralFunction::A));
    // PC04 as MISO
    chip.pc04.configure(Some(sam4l::gpio::PeripheralFunction::A));
    // PC05 as MOSI
    chip.pc05.configure(Some(sam4l::gpio::PeripheralFunction::A));
    // PC00 as slave select 2
    chip.pc00.configure(Some(sam4l::gpio::PeripheralFunction::A));
    // PC02 as slave select 1
    chip.pc02.configure(Some(sam4l::gpio::PeripheralFunction::A));
    // Pin note: SPI_CS2 and SPI_CS1 on the Firestorm schematic are swapped.
    // The primary function column of the Storm pin reference is incorrect,
    // but the A column is correct.
    // PC02 is chip select 1, and PC00 is chip select 2.

    firestorm.spi.set_active_peripheral(sam4l::spi::Peripheral::Peripheral1);
    firestorm.spi.init(hil::spi_master::SPIParams {
        baud_rate: 8000000,
        data_order: hil::spi_master::DataOrder::LSBFirst,
        clock_polarity: hil::spi_master::ClockPolarity::IdleHigh,
        clock_phase: hil::spi_master::ClockPhase::SampleLeading,
        client: None,
    });
    firestorm.spi.enable();

    let mut i: u8 = 0;
    loop {
        firestorm.spi.write_byte(i);
        i += 1;
    }

    // End SPI testing


    firestorm
}
