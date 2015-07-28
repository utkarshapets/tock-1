//! Traits and parameters for SPI master communication

use core::ops::Fn;

/// Values for the ordering of bits
#[derive(Copy, Clone)]
pub enum DataOrder {
    /// The most significant bit is sent first
    MSBFirst,
    /// The least significant bit is sent first
    LSBFirst,
}

/// Values for the clock polarity (idle state or CPOL)
#[derive(Copy, Clone)]
pub enum ClockPolarity {
    /// The base value of the clock is one
    /// (CPOL = 1)
    IdleHigh,
    /// The base value of the clock is zero
    /// (CPOL = 0)
    IdleLow,
}
/// Values for the clock phase (CPHA), which defines when
/// values are sampled
#[derive(Copy, Clone)]
pub enum ClockPhase {
    /// Sample on the leading edge (CPHA = 0)
    SampleLeading,
    /// Sample on the trailing edge (CPHA = 1)
    SampleTrailing,
}


/// A trait for notification when a byte has been read
pub trait Reader {
    /// Called when a write has finished
    fn write_done(&mut self);
    /// Called when a read has finished
    fn read_done(&mut self);
    /// Called when a combined read/write operation has finished
    fn read_write_done(&mut self);
}

/// Parameters for SPI communication
#[derive(Copy, Clone)]
pub struct SPIParams {
    /// The number of bits per second to send and receive
    pub baud_rate: u32,
    /// The bit ordering
    pub data_order: DataOrder,
    /// The clock polarity
    pub clock_polarity: ClockPolarity,
    /// The clock phase
    pub clock_phase: ClockPhase,
    /// The client to be notified when a read or write completes
    pub client: &'static mut Reader,
}

/// A trait for types that allow SPI communication
pub trait SPI {
    /// Configures an object for communication as an SPI master
    fn init(&mut self, params: SPIParams);

    /// Simultaneously sends a byte and receives a byte.
    /// Returns the received byte.
    fn write_byte(&mut self, out_byte: u8) -> u8;
    /// Sends a zero byte while simultaneously receiving a byte,
    /// and returns the received byte.
    fn read_byte(&mut self) -> u8;

    /// Reads `buffer.len()` bytes and stores them in the provided buffer.
    /// Executes asynchronously and calls this object's client `read_done()` callback when done.
    fn read(&mut self, buffer: &mut [u8]);
    /// Writes `buffer.len()` bytes from the provided buffer.
    /// Executes asynchronously and calls this object's client `write_done()` callback when done.
    fn write(&mut self, buffer: &[u8]);

    /// Simultaneously reads and writes bytes.
    /// The number of bytes read is the smaller of `read_buffer.len()` and `write_buffer.len()`.
    /// If the read buffer is larger than the write buffer, the values
    /// in the read buffer at indices `write_buffer.len()` and greater are
    /// undefined.
    /// Executes asynchronously and calls this object's client `read_write_done()` callback when
    /// done.
    fn read_and_write(&mut self, read_buffer: &mut [u8], write_buffer: &[u8]);

    /// Enables receive functionality
    fn enable_rx(&mut self);
    /// Disables receive functionality
    fn disable_rx(&mut self);
    /// Enables transmit functionality
    fn enable_tx(&mut self);
    /// Disables transmit functionality
    fn disable_tx(&mut self);
}
