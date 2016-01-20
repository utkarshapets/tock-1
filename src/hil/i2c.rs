pub trait I2C {
    fn enable(&self);
    fn disable(&self);
    fn write_sync(&self, addr: u16, data: &[u8]);
    fn read_sync(&self, addr: u16, buffer: &mut [u8]);
    fn write_read_repeated_start(&self, addr: u16, reg: u8, data: &mut [u8]);
}
