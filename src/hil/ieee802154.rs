//! APIs for IEEE 802.15.4 communication



use core::option::Option;
use core::clone::Clone;

/// Frame types
pub enum FrameType {
    Beacon,
    Data,
    Acknowledge,
    MACCommand,
}

impl FrameType {
    /// Converts this frame type into a 3-bit value
    fn as_byte(&self) -> u8 {
        match *self {
            FrameType::Beacon => 0,
            FrameType::Data => 1,
            FrameType::Acknowledge => 2,
            FrameType::MACCommand => 3,
        }
    }
}

/// Forms of addresses
#[derive(Copy,Clone)]
pub enum Address {
    /// Short 16-bit address
    Short(u16),
    /// Long 64-bit address
    Long(u64),
}

impl Address {
    /// Returns the addressing mode of this address as a 2-bit value
    fn address_mode(&self) -> u8 {
        match *self {
            Address::Short(_) => 0b10,
            Address::Long(_) => 0b11,
        }
    }
}

/// The maximum length in bytes of a MAC protocol data unit
const MAX_MPDU_LENGTH: usize = 127;
/// The maximum length in bytes of the payload of a MAC protocol data unit
/// A payload can have this size when the source and destination addresses are excluded
const MAX_PAYLOAD_LENGTH: usize = MAX_MPDU_LENGTH - 5;

/// Represents an IEEE 802.15.4 MAC Protocol Data Unit
pub struct Frame {
    /// The type of this frame
    pub frame_type: FrameType,
    /// If security processing is enabled for this frame
    pub security_enabled: bool,
    /// If the transmitter has more frames to send in the near future
    pub frame_pending: bool,
    /// If the node receiving the frame should send an acknowledgment
    pub acknowledgment_request: bool,
    /// The number of this frame in a sequence
    /// Used to detect duplicate or dropped frames
    pub sequence_number: u8,
    /// Source PAN ID
    pub source_pan_id: Option<u16>,
    /// Source address
    pub source_address: Option<Address>,
    /// Destination PAN ID
    pub destination_pan_id: Option<u16>,
    /// Destination address
    pub destination_address: Option<Address>,

    /// The payload
    /// The array contains MAX_PAYLOAD_LENGTH elements, which may be greater than the actual
    /// size of the payload
    payload: [u8; MAX_PAYLOAD_LENGTH],
    /// The actual length of the payload
    /// Must be less than or equal to MAX_PAYLOAD_LENGTH
    payload_length: usize,
}

pub trait Reader {
    /// Called when a frame has been received
    fn frame_received(&mut self, frame: Frame);
    /// Called when a frame has been successfully sent
    fn send_done(&mut self);
}

pub struct Params {
    client: &'static mut Reader,
}

/// A trait for transceivers that can send and receive 802.15.4 frames
// Might need a better name
pub trait Transceiver {
    fn init(&mut self, params: Params);

    /// Enables receiving of frames. When enabled, the client's `frame_received()` method
    /// will be called when a frame is received.
    fn enable_rx(&mut self);
    /// Disables receiving of frames
    fn disable_rx(&mut self);

    /// Sends the provided frame.
    /// Executes asynchronously and calls the client's `send_done()` method when done.
    fn send(&mut self, frame: Frame);

}
