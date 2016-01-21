use core::cell::Cell;
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
    enabled: Cell<bool>,
    last_accel: Cell<Option<[i16; 3]>>,
    callback: Cell<Option<Callback>>,
}

impl<'a, I: I2C> AccelFXOS8700CQ<'a, I> {
    pub fn new(i2c: &'a I, timer: &'a Timer) -> AccelFXOS8700CQ<'a, I> {
        AccelFXOS8700CQ{
            i2c: i2c,
            timer: timer,
            address: 0x1E, // default on FireStorm
            enabled: Cell::new(false),
            last_accel: Cell::new(None),
            callback: Cell::new(None),
        }
    }

    // if the accelerometer's i2c address is different
    pub fn new_withaddress(i2c: &'a I, timer: &'a Timer, address: u16) -> AccelFXOS8700CQ<'a, I> {
        let mut a = AccelFXOS8700CQ::new(i2c, timer);
        a.address = address;
        a
    }
}

impl<'a, I: I2C> TimerClient for AccelFXOS8700CQ<'a, I> {
    fn fired(&self, _: u32) {
        if !self.enabled.get() {
            return;
        }
        let mut accel_xyz: [i16; 3] = [0; 3];
        let mut rbuf: [u8; 6] = [0; 6];
        rbuf[0] = Registers::OutXMSB as u8;
        self.i2c.write_read_repeated_start(self.address, Registers::OutXMSB as u8, &mut rbuf[0..6]);
        accel_xyz[0]= (((rbuf[0] as u16) << 8) | (rbuf[1] as u16)) as i16;
        accel_xyz[1]= (((rbuf[2] as u16) << 8) | (rbuf[3] as u16)) as i16;
        accel_xyz[2]= (((rbuf[4] as u16) << 8) | (rbuf[5] as u16)) as i16;
        self.last_accel.set(Some(accel_xyz));
        self.callback.get().map(|mut cb| {
            cb.schedule(accel_xyz[0] as usize, accel_xyz[1] as usize, accel_xyz[2] as usize);
        });
    }
}

impl<'a, I: I2C> Driver for AccelFXOS8700CQ<'a, I> {
    fn subscribe(&self, subscribe_num: usize, mut callback: Callback) -> isize {
        match subscribe_num {
            0 => { // read all three accelerometer values
                // if chip hasn't been enabled yet (Driver::command(0))
                if !self.enabled.get() {
                    return -1;
                }
                match self.last_accel.get() {
                    Some(accel) => {
                        callback.schedule(accel[0] as usize, accel[1] as usize ,accel[2] as usize);
                    },
                    None => {
                        self.callback.set(Some(callback));
                    }
                }
                0
            },
            _ => -1
        }
    }

    // TODO
    // want to be able to disable timer. unsubscribe, or other command that disables.
    // unsubscribe, not disable
    /*
     * Discussion: what do these commands do? the subscription command shoud probably start
     * the timer. Think about maybe how to have multiple subscribers
     * Also fold in the magnetometer here. It might be the case that one app wants to use
     * the magnetometer, and one app wants to use the accelerometer
     * ALSO: write up the wiki page for a driver. Make a good example.
     * ALSO: write up an email on I2C driver interface: difference is between deferred
     * and synchronous read on the i2c. Need more arguments to configure things. Make
     * this list and make a decision on each one.
     */
    fn command(&self, cmd_num: usize, _: usize) -> isize {
        match cmd_num {
            0 => { // enable the sensor
                self.i2c.enable();

                let mut buf: [u8; 2] = [0; 2];

                // check that the accelerometer isn't crazy
                self.i2c.write_read_repeated_start(self.address, Registers::WhoAmI as u8, &mut buf[0..1]);
                if buf[0] != (0xC7 as u8) { // 0xC7 is the device identifier for WHOAMI
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

                self.timer.repeat(32768);

                // mark ourselves as enabled
                self.enabled.set(true);

                // success!
                0
            },
            _ => -1
        }
    }
}
