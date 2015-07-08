
use core::ops::Index;
use core::ops::IndexMut;

/// Represents a frame received from the RF230
/// A frame can contain up to 127 bytes of data.
pub struct Frame {
    /// 127 bytes of data
    /// Actual frames can be shorter than 127 bytes, but this length ensures that all frames
    /// can fit. The actual length is stored in the length field.
    data: [u8; FRAME_MAX_LENGTH as usize],
    /// The number of data bytes in the frame
    length: u8,
}
