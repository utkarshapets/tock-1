
use core::ops::Index;
use core::ops::IndexMut;

/// The maximum number of bytes in a frame
const FRAME_MAX_LENGTH: u8 = 127;
/// Represents a frame received from the RF230
/// A frame can contain up to 127 bytes of data.
pub struct Frame {
    /// 127 bytes of data
    /// Actual frames can be shorter than 127 bytes, but this length ensures that all frames
    /// can fit. The actual length is stored in the lenght field.
    data: [u8; FRAME_MAX_LENGTH as usize],
    /// The number of data bytes in the frame
    pub len: u8,
    /// The link qualtiy index, a value from 0 to 255 representing the quality of the connection.
    /// The LQI is measured over several frames, not just this frame.
    pub lqi: u8,
}
impl Frame {
    /// Creates a new frame with the specified length, with all data bytes set to zero
    /// and LQI set to zero.
    /// If the requested length is greater than 127, returns a frame with a length of 127 bytes.
    pub fn new(mut length: u8) -> Frame {
        if length > FRAME_MAX_LENGTH {
            length = FRAME_MAX_LENGTH;
        }
        Frame{ data: [0; FRAME_MAX_LENGTH as usize], len: length, lqi: 0 }
    }

    /// Returns the maximum number of data bytes a frame can contain
    pub fn max_length() -> usize {
        FRAME_MAX_LENGTH as usize
    }
}
impl Index<usize> for Frame {
    type Output = u8;
    /// Provides access to a byte of data in a frame. Panics if index > self.len - 1.
    fn index<'a>(&'a self, index: usize) -> &'a u8 {
        // Bounds check based on actual frame length
        if index > ((self.len as usize) - 1) {
            static MESSAGE: (&'static str, &'static str, u32) = ("Frame index out of bounds", file!(), line!());
            ::core::panicking::panic(&MESSAGE);
        }
        &self.data[index]
    }
}
impl IndexMut<usize> for Frame {
    /// Provides access to a byte of data in a frame. Panics if index > self.len - 1.
    fn index_mut<'a>(&'a mut self, index: usize) -> &'a mut u8 {
        if index > ((self.len as usize) - 1) {
            static MESSAGE: (&'static str, &'static str, u32) = ("Frame index out of bounds", file!(), line!());
            ::core::panicking::panic(&MESSAGE);
        }
        &mut self.data[index]
    }
}
