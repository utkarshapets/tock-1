

use core::option::Option;
use core::clone::Clone;

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

impl Addresses {
    /// Returns true if this set of addresses is for a transmission within a PAN that exludes
    /// the source PAN ID. Otherwise returns false.
    fn is_within_pan(&self) -> bool {
        match *self {
            Addresses::Local{ref source_address, ref destination} => true,
            _ => false,
        }
    }

    /// Returns the addressing mode of the source address as a 2-bit value
    fn source_mode(&self) -> u8 {
        match *self {
            Addresses::Local{ref source_address, ref destination} => {
                source_address.address_mode()
            },
            Addresses::Full{ref source, ref destination} => {
                match source {
                    &Option::Some(ref source_address) => source_address.address.address_mode(),
                    &Option::None => 0b0,
                }
            }
        }
    }
    /// Returns the addressing mode of the destination address as a 2-bit value
    fn destination_mode(&self) -> u8 {
        match *self {
            Addresses::Local{ref source_address, ref destination} => {
                match destination.address {
                    Address::Short(_) => 0b10,
                    Address::Long(_) => 0b11,
                }
            },
            Addresses::Full{ref source, ref destination} => {
                match destination {
                    &Option::Some(ref destination_address) => destination_address.address.address_mode(),
                    &Option::None => 0b0,
                }
            }
        }
    }
    /// Returns the source PAN ID if this address block contains one
    fn source_pan_id(&self) -> Option<u16> {
        match *self {
            Addresses::Local{ref source_address, ref destination} => {
                Option::None
            },
            Addresses::Full{ref source, ref destination} => {
                match source {
                    &Option::Some(ref source_address) => Option::Some(source_address.pan_id),
                    &Option::None => Option::None,
                }
            }
        }
    }
    /// Returns the source address if this address block contains one
    fn source_address(&self) -> Option<Address> {
        match *self {
            Addresses::Local{ref source_address, ref destination} => {
                Option::Some(source_address.clone())
            },
            Addresses::Full{ref source, ref destination} => {
                match source {
                    &Option::Some(ref source_address) => Option::Some(source_address.address),
                    &Option::None => Option::None,
                }
            }
        }
    }
    /// Returns the destination PAN ID if this address block contains one
    fn destination_pan_id(&self) -> Option<u16> {
        match *self {
            Addresses::Local{ref source_address, ref destination} => {
                Option::Some(destination.pan_id)
            },
            Addresses::Full{ref source, ref destination} => {
                match destination {
                    &Option::Some(ref destination_address) => Option::Some(destination_address.pan_id),
                    &Option::None => Option::None,
                }
            }
        }
    }
    /// Returns the destination address if this address block contains one
    fn destination_address(&self) -> Option<Address> {
        match *self {
            Addresses::Local{ref source_address, ref destination} => {
                Option::Some(destination.address)
            },
            Addresses::Full{ref source, ref destination} => {
                match destination {
                    &Option::Some(ref destination_address) => Option::Some(destination_address.address),
                    &Option::None => Option::None,
                }
            }
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
    /// to the IEEE 802.15.4 protocol, but excluding the CRC-16 checksum.
    ///
    /// Byte 0 in the returned array should be sent first.
    ///
    /// Returns an array of bytes and the length of the protocol data unit.
    ///
    fn as_mpdu_bytes(&self) -> ([u8; MAX_MPDU_LENGTH], usize) {
        let mut bytes: [u8; MAX_MPDU_LENGTH] = [0; MAX_MPDU_LENGTH];
        let intra_pan = self.addresses.is_within_pan();

        bytes[0] = self.frame_type.as_byte()
            | ((self.security_enabled as u8) << 3)
            | ((self.frame_pending as u8) << 4)
            | ((self.acknowledgment_request as u8) << 5)
            | ((intra_pan as u8) << 6);
        bytes[1] = (self.addresses.destination_mode() << 2)
            | (self.addresses.source_mode() << 6);

        // Stores the index in bytes where the next value will be put
        let mut index: usize = 2;
        Frame::append_address(&mut bytes, &mut index, &self.addresses.destination_address());
        Frame::append_pan_id(&mut bytes, &mut index, &self.addresses.destination_pan_id());
        Frame::append_address(&mut bytes, &mut index, &self.addresses.source_address());
        Frame::append_pan_id(&mut bytes, &mut index, &self.addresses.source_pan_id());

        // Payload
        for payload_index in (0..self.payload_length) {
            bytes[index] = self.payload[payload_index];
            index = index + 1;
        }
        
        (bytes, index)
    }

    /// Appends an address to an array of bytes
    /// The least significant byte is placed at the initial value of index.
    /// The index parameter is incremented to be one greater than the index of the most
    /// significant bit of the address.
    fn append_address(bytes: &mut [u8], index: &mut usize, address_option: &Option<Address>) {
        match *address_option {
            Option::Some(address) => {
                match address {
                    Address::Short(n) => {
                        bytes[*index] = (n & 0xFF) as u8;
                        bytes[*index + 1] = ((n >> 8) & 0xFF) as u8;
                        *index += 2;
                    },
                    Address::Long(n) => {
                        bytes[*index] = (n & 0xFF) as u8;
                        bytes[*index + 1] = ((n >> 8) & 0xFF) as u8;
                        bytes[*index + 2] = ((n >> 16) & 0xFF) as u8;
                        bytes[*index + 3] = ((n >> 24) & 0xFF) as u8;
                        bytes[*index + 4] = ((n >> 32) & 0xFF) as u8;
                        bytes[*index + 5] = ((n >> 40) & 0xFF) as u8;
                        bytes[*index + 6] = ((n >> 48) & 0xFF) as u8;
                        bytes[*index + 7] = ((n >> 56) & 0xFF) as u8;
                        *index += 8;
                    },
                }
            },
            Option::None => {},
        }
    }
    /// Appends a PAN ID to an array of bytes
    /// The least significant byte is placed at the initial value of index.
    /// The index parameter is incremented to be one greater than the index of the most
    /// significant bit of the PAN ID.
    fn append_pan_id(bytes: &mut [u8], index: &mut usize, pan_id_option: &Option<u16>) {
        match *pan_id_option {
            Option::Some(pan_id) => {
                bytes[*index] = (pan_id & 0xFF) as u8;
                bytes[*index + 1] = ((pan_id >> 8) & 0xFF) as u8;
                *index += 2;
            },
            Option::None => {},
        }
    }
}
