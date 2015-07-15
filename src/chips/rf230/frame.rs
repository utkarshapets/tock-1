
use core::option::Option;
use core::result::Result;

/// Frame types
pub enum FrameType {
    Beacon,
    Data,
    Acknowledge,
    MACCommand,
    /// Indicates an unrecognized type
    Other(u8),
}

impl FrameType {
    /// Converts this frame type into a 3-bit value
    fn as_byte(&self) -> u8 {
        match *self {
            FrameType::Beacon => 0,
            FrameType::Data => 1,
            FrameType::Acknowledge => 2,
            FrameType::MACCommand => 3,
            FrameType::Other(value) => value & 0b111,
        }
    }
}

/// Forms of addresses
pub enum Address {
    /// Short 16-bit address
    Short(u16),
    /// Long 64-bit address
    Long(u64),
}

/// Contains a local address and a PAN ID
pub struct FullAddress {
    /// Address
    pub address: Address,
    /// PAN ID
    pub pan_id: u16,
}

/// Valid address combinations
pub enum Addresses {
    // Within a PAN, both addresses included, origin PAN ID excluded
    Local {
        /// Origin address without PAN ID
        source_address: Address,
        /// Destination address and PAN ID
        destination: FullAddress,
    },
    // Between PANs, both addresses optional, a PAN ID is required for each address
    Full {
        source: Option<FullAddress>,
        destination: Option<FullAddress>,
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
    /// Source and/or destination addresses
    pub addresses: Addresses,

    /// The payload
    /// The array contains MAX_PAYLOAD_LENGTH elements, which may be greater than the actual
    /// size of the payload
    payload: [u8; MAX_PAYLOAD_LENGTH],
    /// The actual length of the payload
    /// Must be less than or equal to MAX_PAYLOAD_LENGTH
    payload_length: usize,
}

impl Frame {
    /// Converts this frame into an array of bytes representing a MAC protocol data unit according
    /// to the IEEE 802.15.4 protocol.
    ///
    /// Byte 0 in the returned array should be sent first.
    ///
    /// Returns an array of bytes and the length of the protocol data unit
    fn as_mpdu_bytes(&self) -> ([u8; MAX_MPDU_LENGTH], usize) {
        let mut bytes: [u8; MAX_MPDU_LENGTH] = [0; MAX_MPDU_LENGTH];
        let length = self.mpdu_length();

        let mut index = 0;

        bytes[0] = self.frame_type.as_byte()
            | ((self.security_enabled as u8) << 3)
            | ((self.frame_pending as u8) << 4)
            | ((self.acknoledgment_request as u8) << 5);
        // TODO: More


        (bytes, length)
    }


    /// Returns the length of this frame when formatted as a MAC protocol data unit,
    /// including the MAC header and MAC footer
    fn mpdu_length(&self) -> usize {
        let address_block_length = match self.addresses {
            Addresses::Local{ ref source_address, ref destination } => {
                let source_address_length = match *source_address {
                    Address::Short(_) => 2,
                    Address::Long(_) => 8,
                };
                // 2 bytes for PAN ID plus 2 or 8 bytes for address
                let destination_address_length = 2 + match destination.address {
                    Address::Short(_) => 2,
                    Address::Long(_) => 8,
                };
                source_address_length + destination_address_length
            },
            Addresses::Full{ ref source, ref destination } => {
                0
            }
        };
        2 + 1 + address_block_length + self.payload_length + 2
    }
}
