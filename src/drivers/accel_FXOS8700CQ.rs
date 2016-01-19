use hil::{Driver, Callback};
use hil::i2c::I2C;
use hil::timer::*;

#[allow(dead_code)]
enum Registers {
    Status = 0x00,

    // XYZ FIFO data
    OutXMSB = 0x01,
    OutXLSB = 0x02,
    OutYMSB = 0x03,
    OutYLSB = 0x04,
    OutZMSB = 0x05,
    OutZLSB = 0x06,
    // FIFO Setup
    FSetup = 0x09,
    // FIFO event trigger config
    TrigCFG = 0x0A,
    // System Mode
    SysMod = 0x0B,
    // interrupt status
    IntSource = 0x0C,

    WhoAmI = 0x0D,

    // dynamic range and filter enable settings
    XYZDataCFG = 0x0E,

    // config registers
    // system ODR, accel OSR, operating mode
    CtrlReg1 = 0x2A,
    // reset, accelerometer OSR, sleep mode settings
    CtrlReg2 = 0x2B,
    // sleep mode interrupt, wake enable
    CtrlReg3 = 0x2C,
    // enable interrupt register
    CtrlReg4 = 0x2D,
}

pub struct AccelFXOS8700CQ<'a, I: I2C + 'a> {
    i2c: &'a I,
    timer: &'a Timer,
    address: u16,
}

impl<'a, I: I2C> AccelFXOS8700CQ<'a, I> {
    pub fn new(i2c: &'a I, timer: &'a Timer) -> AccelFXOS8700CQ<'a, I> {
        AccelFXOS8700CQ{
            i2c: i2c,
            timer: timer,
            address: 0x1C, // default on FireStorm
        }
    }

    // if the accelerometer's i2c address is different
    pub fn new_withaddress(i2c: &'a I, timer: &'a Timer, address: u16) -> AccelFXOS8700CQ<'a, I> {
        AccelFXOS8700CQ{
            i2c: i2c,
            timer: timer,
            address: address,
        }
    }
}

impl<'a, I: I2C> Driver for AccelFXOS8700CQ<'a, I> {
    fn command(&self, cmd_num: usize, _: usize) -> isize {
        match cmd_num {
            0 => { // enable the sensor
                self.i2c.enable();
                let mut buf: [u8; 2] = [0; 2];
        
                // check that the accelerometer isn't crazy
                self.i2c.read_sync(self.address, &mut buf[0..1]);
                if buf[0] != Registers::WhoAmI as u8 {
                    return -1;
                }

                // put into standby mode
                buf = [Registers::CtrlReg1 as u8, 0x00];
                self.i2c.write_sync(self.address, &buf);

                // config accelerometer
                buf = [Registers::XYZDataCFG as u8, 0x01];
                self.i2c.write_sync(self.address, &buf);

                // leave standby
                buf = [Registers::CtrlReg1 as u8, 0x0D];
                self.i2c.write_sync(self.address, &buf);
                0

                //self.i2c.write_sync
            },
            _ => -1
        }
    }
}
