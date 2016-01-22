use core::cell::Cell;
use hil::{Driver, Callback};
use hil::i2c::I2C;
use hil::timer::*;

#[allow(dead_code)]
enum Registers {
    Status = 0x00,

    // Accel XYZ FIFO data
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

    // dynamic range and filter enable settings for accel
    XYZDataCFG = 0x0E,

    // magnetometer configs
    MagnetCtrlREG1 = 0x5B,
    MagnetCtrlREG2 = 0x5C,

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

pub struct FXOS8700CQ<'a, I: I2C + 'a> {
    i2c: &'a I,
    timer: &'a Timer,
    address: u16,
    enabled: Cell<bool>,
    useAccelerometer: Cell<bool>,
    useMagnetometer: Cell<bool>,
    last_accel: Cell<Option<[i16; 3]>>,
    last_magnet: Cell<Option<[i16; 3]>>,
    callback: Cell<Option<Callback>>,
}

impl<'a, I: I2C> FXOS8700CQ<'a, I> {
    pub fn new(i2c: &'a I, timer: &'a Timer) -> FXOS8700CQ<'a, I> {
        FXOS8700CQ{
            i2c: i2c,
            timer: timer,
            address: 0x1E, // default on FireStorm
            enabled: Cell::new(false),
            useAccelerometer: Cell::new(false),
            useMagnetometer: Cell::new(false),
            last_accel: Cell::new(None),
            last_magnet: Cell::new(None),
            callback: Cell::new(None),
        }
    }

    // if the accelerometer's i2c address is different
    pub fn new_withaddress(i2c: &'a I, timer: &'a Timer, address: u16) -> FXOS8700CQ<'a, I> {
        let mut a = FXOS8700CQ::new(i2c, timer);
        a.address = address;
        a
    }
}

impl<'a, I: I2C> TimerClient for FXOS8700CQ<'a, I> {
    fn fired(&self, _: u32) {
        if !self.enabled.get() {
            return;
        }
        // the result array
        let mut res_xyz: [i16; 3] = [0; 3];
        // for retrieving the raw results
        let mut rbuf: [u8; 12] = [0; 12];
        // read out the current readings; this should be the same whether we are configured for
        // accel or magnetometer
        rbuf[0] = Registers::OutXMSB as u8;
        self.i2c.write_read_repeated_start(self.address, Registers::OutXMSB as u8, &mut rbuf[0..6]);
        // convert the MSB/LSB into the i16
        if self.useAccelerometer.get() == true { // use first 6
            // convert 14bit accelerometer data to 16bit
            res_xyz[0]= ((((rbuf[0] as u16) << 8) | (rbuf[1] as u16)) >> 2) as i16;
            res_xyz[1]= ((((rbuf[2] as u16) << 8) | (rbuf[3] as u16)) >> 2) as i16;
            res_xyz[2]= ((((rbuf[4] as u16) << 8) | (rbuf[5] as u16)) >> 2) as i16;
            self.last_accel.set(Some(res_xyz));
        } else if self.useMagnetometer.get() == true {
            res_xyz[0]= (((rbuf[0] as u16) << 8) | (rbuf[1] as u16)) as i16;
            res_xyz[1]= (((rbuf[2] as u16) << 8) | (rbuf[3] as u16)) as i16;
            res_xyz[2]= (((rbuf[4] as u16) << 8) | (rbuf[5] as u16)) as i16;
            self.last_magnet.set(Some(res_xyz));
        }
        // store the latest value
        // invoke the callback if we have one
        self.callback.get().map(|mut cb| {
            cb.schedule(res_xyz[0] as usize, res_xyz[1] as usize, res_xyz[2] as usize);
        });
    }
}

impl<'a, I: I2C> Driver for FXOS8700CQ<'a, I> {
    fn subscribe(&self, subscribe_num: usize, mut callback: Callback) -> isize {
        match subscribe_num {
            0 => { // read all three values
                -1 // not implemented yet
            },
            1 => { // subscribe to accelerometer
                // if chip hasn't been enabled yet (Driver::command(0))
                if !self.enabled.get() {
                    return -1;
                }
                if self.useAccelerometer.get() == true {
                    match self.last_accel.get() {
                        // if we have a value, invoke the callback to return it
                        Some(accel) => {
                            callback.schedule(accel[0] as usize, accel[1] as usize ,accel[2] as usize);
                        },
                        // if not, we
                        None => {}
                    }
                    // save the callback if we don't have one already
                    match self.callback.get() {
                        Some(_) => {},
                        None => {
                            self.callback.set(Some(callback));
                        }
                    }
                } else {
                    return -1; // accelerometer not configured!
                }
                0 // success
            },
            2 => { // subscribe to magnetometer
                // if chip hasn't been enabled yet (Driver::command(0))
                if !self.enabled.get() {
                    return -1;
                }
                if self.useMagnetometer.get() == true {
                    match self.last_magnet.get() {
                        // if we have a value, invoke the callback to return it
                        Some(magnet) => {
                            callback.schedule(magnet[0] as usize, magnet[1] as usize ,magnet[2] as usize);
                        },
                        // if not, we
                        None => {}
                    }
                    // save the callback if we don't have one already
                    match self.callback.get() {
                        Some(_) => {},
                        None => {
                            self.callback.set(Some(callback));
                        }
                    }

                }
                0 // success
            }
            _ => -1
        }
    }

    fn command(&self, cmd_num: usize, _: usize) -> isize {
        let mut buf: [u8; 2] = [0; 2];

        if self.enabled.get() == true {
            return -1; // can only be configured once
        }
        self.i2c.enable();

        // check that the accelerometer isn't crazy
        self.i2c.write_read_repeated_start(self.address, Registers::WhoAmI as u8, &mut buf[0..1]);
        if buf[0] != (0xC7 as u8) { // 0xC7 is the device identifier for WHOAMI
            return -1;
        }

        // put into standby mode
        buf = [Registers::CtrlReg1 as u8, 0x00];
        self.i2c.write_sync(self.address, &buf);

        match cmd_num {
            0 => { // enable all sensors
                // config accelerometer
                buf = [Registers::XYZDataCFG as u8, 0x01];
                self.i2c.write_sync(self.address, &buf);

                // config magnetometer
                // Using the default configurations from old Firestorm stuff
                buf = [Registers::MagnetCtrlREG1 as u8, 0x1f];
                self.i2c.write_sync(self.address, &buf);
                buf = [Registers::MagnetCtrlREG2 as u8, 0x20];
                self.i2c.write_sync(self.address, &buf);

                // mark both accelerometer and magnetometer as configured
                self.useAccelerometer.set(true);
                self.useMagnetometer.set(true);
            },
            1 => { // enable accelerometer
                // config accelerometer
                buf = [Registers::XYZDataCFG as u8, 0x01];
                self.i2c.write_sync(self.address, &buf);

                // mark accelerometer as configured
                self.useAccelerometer.set(true);
            },
            2 => { // enable magnetometer
                // config magnetometer
                // Using the default configurations from old Firestorm stuff
                buf = [Registers::MagnetCtrlREG1 as u8, 0x1f];
                self.i2c.write_sync(self.address, &buf);
                buf = [Registers::MagnetCtrlREG2 as u8, 0x20];
                self.i2c.write_sync(self.address, &buf);

                // mark magnetometer as configured
                self.useMagnetometer.set(true);
            }
            _ => {return -1;} // unrecognized command
        }

        // leave standby
        buf = [Registers::CtrlReg1 as u8, 0x0D];
        self.i2c.write_sync(self.address, &buf);

        // mark ourselves as enabled
        self.enabled.set(true);

        self.timer.repeat(32768);

        0 // success!
    }
}
