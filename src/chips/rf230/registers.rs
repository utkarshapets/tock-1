

/// Stores the address and reset value of a register
#[derive(Copy,Clone)]
pub struct Register {
    /// The address
    pub address: u8,
    /// The reset value
    /// (When a register is written, the reserved bits must be set to their reset values)
    pub reset_value: u8,
    /// A value with the reserved bits set to 1 and other bits set to 0, to indicate the bits
    /// that are reserved
    pub reserved_mask: u8,
}

impl Register {
    /// Processes a value to be written. Ensures that the value to be written does not change
    /// any reserved bits, and all reserved bits are set to their default values.
    /// Returns the cleaned value.
    pub fn clean_for_write(&self, value: u8) -> u8 {
        // Zero all the bits of the value that are reserved
        // Note: The ! operator on a non-boolean value is bitwise not.
        let changed_bits = value & !self.reserved_mask;
        // Zero the non-reserved bits of the reset value, leaving the reserved bits
        // equal to the reset value
        let reserved_reset_bits = self.reset_value & self.reserved_mask;
        // Return the unreserved changed bits and the reserved reset bits
        changed_bits | reserved_reset_bits
    }
}

// Register definitions

pub const TRX_STATUS:   Register = Register{ address: 0x01, reset_value: 0x0,  reserved_mask: 0b00100000 };
pub const TRX_STATE:    Register = Register{ address: 0x02, reset_value: 0x0,  reserved_mask: 0x0 };
pub const TRX_CTRL_0:   Register = Register{ address: 0x03, reset_value: 0x19, reserved_mask: 0x0 };
pub const PHY_TX_PWR:   Register = Register{ address: 0x05, reset_value: 0x0,  reserved_mask: 0b01110000 };
pub const PHY_RSSI:     Register = Register{ address: 0x06, reset_value: 0x0,  reserved_mask: 0b01100000 };
pub const PHY_ED_LEVEL: Register = Register{ address: 0x07, reset_value: 0x0,  reserved_mask: 0x0 };
pub const PHY_CC_CCA:   Register = Register{ address: 0x08, reset_value: 0x2B, reserved_mask: 0x0 };
pub const CCA_THRES:    Register = Register{ address: 0x09, reset_value: 0xC7, reserved_mask: 0b11110000 };
pub const IRQ_MASK:     Register = Register{ address: 0x0E, reset_value: 0xFF, reserved_mask: 0b00110000 };
pub const IRQ_STATUS:   Register = Register{ address: 0x0F, reset_value: 0x0,  reserved_mask: 0b00110000 };
pub const VREG_CTRL:    Register = Register{ address: 0x10, reset_value: 0x0,  reserved_mask: 0b00110011 };
pub const BATMON:       Register = Register{ address: 0x11, reset_value: 0x02, reserved_mask: 0b11000000 };
pub const XOSC_CTRL:    Register = Register{ address: 0x12, reset_value: 0xF0, reserved_mask: 0x0 };
pub const PLL_CF:       Register = Register{ address: 0x1A, reset_value: 0x5F, reserved_mask: 0b01111111 };
pub const PLL_DCU:      Register = Register{ address: 0x1B, reset_value: 0x20, reserved_mask: 0b01111111 };
pub const PART_NUM:     Register = Register{ address: 0x1C, reset_value: 0x02, reserved_mask: 0x0 };
pub const VERSION_NUM:  Register = Register{ address: 0x1D, reset_value: 0x02, reserved_mask: 0x0 };
pub const MAN_ID_0:     Register = Register{ address: 0x1E, reset_value: 0x1F, reserved_mask: 0x0 };
pub const MAN_ID_1:     Register = Register{ address: 0x1F, reset_value: 0x0,  reserved_mask: 0x0 };
pub const SHORT_ADDR_0: Register = Register{ address: 0x20, reset_value: 0x0,  reserved_mask: 0x0 };
pub const SHORT_ADDR_1: Register = Register{ address: 0x21, reset_value: 0x0,  reserved_mask: 0x0 };
pub const PAN_ID_0:     Register = Register{ address: 0x22, reset_value: 0x0,  reserved_mask: 0x0 };
pub const PAN_ID_1:     Register = Register{ address: 0x23, reset_value: 0x0,  reserved_mask: 0x0 };
pub const IEEE_ADDR_0:  Register = Register{ address: 0x24, reset_value: 0x0,  reserved_mask: 0x0 };
pub const IEEE_ADDR_1:  Register = Register{ address: 0x25, reset_value: 0x0,  reserved_mask: 0x0 };
pub const IEEE_ADDR_2:  Register = Register{ address: 0x26, reset_value: 0x0,  reserved_mask: 0x0 };
pub const IEEE_ADDR_3:  Register = Register{ address: 0x27, reset_value: 0x0,  reserved_mask: 0x0 };
pub const IEEE_ADDR_4:  Register = Register{ address: 0x28, reset_value: 0x0,  reserved_mask: 0x0 };
pub const IEEE_ADDR_5:  Register = Register{ address: 0x29, reset_value: 0x0,  reserved_mask: 0x0 };
pub const IEEE_ADDR_6:  Register = Register{ address: 0x2A, reset_value: 0x0,  reserved_mask: 0x0 };
pub const IEEE_ADDR_7:  Register = Register{ address: 0x2B, reset_value: 0x0,  reserved_mask: 0x0 };
pub const XAH_CTRL:     Register = Register{ address: 0x2C, reset_value: 0x38, reserved_mask: 0b00000001 };
pub const CSMA_SEED_0:  Register = Register{ address: 0x2D, reset_value: 0xEA, reserved_mask: 0x0 };
pub const CSMA_SEED_1:  Register = Register{ address: 0x2E, reset_value: 0xC2, reserved_mask: 0b00010000 };
