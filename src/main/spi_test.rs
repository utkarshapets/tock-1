
#[allow(improper_ctypes)]
extern {
    fn __subscribe(driver_num: usize, subnum: usize, cb: usize);
    fn __command(driver_num: usize, cmdnum: usize, arg1: usize);
    fn __wait(a: usize, b: usize, c: usize);
}

fn command(driver_num: usize, cmdnum: usize, arg1: usize) {
    unsafe {
        __command(driver_num, cmdnum, arg1);
    }
}

fn subscribe(driver_num: usize, cmdnum: usize, callback: usize) {
    unsafe {
        __subscribe(driver_num, cmdnum, callback);
    }
}

fn wait() {
    unsafe {
        __wait(0, 0, 0);
    }
}

mod console {
    use core::str::StrExt;
    use super::{command, subscribe};

    pub fn putc(c: char) {
        command(0, 0, c as usize);
    }

    pub fn puts(string: &str) {
        for c in string.chars() {
            putc(c);
        }
    }

    pub fn subscribe_read(f: fn(char)) {
        subscribe(0, 0, f as usize);
    }
}

pub mod spi_test {
    use super::wait;
    use super::console::*;
    use core::str;
    use core::prelude::*;

    pub fn _start() {
        init();
        loop {
            wait();
        }
    }

    fn init() {
        puts("It works!");
    }
}
