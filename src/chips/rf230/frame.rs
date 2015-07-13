
use core::option::Option;
use core::result::Result;

/// Frame types
pub enum FrameType {
    Beacon,
    Data,
    Acknowledge,
    MACCommand,
    /// Indicates an unexpected value
    Other(u8),
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
    /// Converts this frame into a slice of bytes representing a MAC protocol data unit according
    /// to the IEEE 802.15.4 protocol


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
        0
    }
}
