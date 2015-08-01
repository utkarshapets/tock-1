

/// The registers used to control interrupts
#[repr(C, packed)]
struct EICRegisters {
    ier: u32, // 0x0
    idr: u32, // 0x4
    imr: u32, // 0x8
    isr: u32, // 0xC
    icr: u32, // 0x10
    mode: u32, // 0x14
    edge: u32, // 0x18
    level: u32, // 0x1C
    filter: u32, // 0x20
    test: u32, // 0x24
    async: u32, // 0x28
    reserved0: u32, // 0x2C
    // Note: en, dis, and ctrl seem to be very similar to ier, idr, and imr. It is not clear
    // how they are different. The current implementation sets both when enabling or disabling.
    en: u32, // 0x30
    dis: u32, // 0x34
    ctrl: u32, // 0x38
    // Version register omitted
}

const BASE_ADDRESS: u32 = 0x400F1000;
static mut regs: *mut EICRegisters = BASE_ADDRESS as *mut EICRegisters;

/// Defines the available external interrupts
#[derive(Copy,Clone)]
pub enum Interrupt {
    /// The non-maskable interrupt
    NonMaskable = 0,
    /// Interrupt 1
    Interrupt1 = 1,
    /// Interrupt 2
    Interrupt2 = 2,
    /// Interrupt 3
    Interrupt3 = 3,
    /// Interrupt 4
    Interrupt4 = 4,
    /// Interrupt 5
    Interrupt5 = 5,
    /// Interrupt 6
    Interrupt6 = 6,
    /// Interrupt 7
    Interrupt7 = 7,
    /// Interrupt 8
    Interrupt8 = 8,
}

/// Conditions for interrupts to be triggered
pub enum Trigger {
    /// Trigger on rising edge
    RisingEdge,
    /// Trigger on falling edge
    FallingEdge,
    /// Trigger on high
    High,
    /// Trigger on low
    Low,
}

/// Sets whether an interrupt should be enabled
pub fn set_enabled(int: Interrupt, enabled: bool) {
    if enabled {
        unsafe { volatile!((*regs).ier = 1 << (int as u8)) };
        unsafe { volatile!((*regs).en = 1 << (int as u8)) };
    }
    else {
        unsafe { volatile!((*regs).idr = 1 << (int as u8)) };
        unsafe { volatile!((*regs).dis = 1 << (int as u8)) };
    }
}

/// Returns true if the specified interrupt is enabled, otherwise false
pub fn is_enabled(int: Interrupt) -> bool {
    unsafe { ((volatile!((*regs).imr) >> (int as u8)) & 1) == 1 }
}

/// Returns true if the specified interrupt has been triggered since the last call to
/// clear() with the same interrupt
///
/// This value is reset by calling clear().
pub fn has_occurred(int: Interrupt) -> bool {
    unsafe { ((volatile!((*regs).isr) >> (int as u8)) & 1) == 1 }
}

/// Clears the specified interrupt
pub fn clear(int: Interrupt) {
    unsafe { volatile!((*regs).icr = 1 << int as u8) };
}
/// Sets the trigger for an interrupt
pub fn set_trigger(int: Interrupt, trigger: Trigger) {
    match trigger {
        Trigger::RisingEdge => unsafe {
            volatile!((*regs).mode |= 1 << (int as u8)); // Mode bit 1
            volatile!((*regs).edge |= 1 << (int as u8)); // Edge bit 1
        },
        Trigger::FallingEdge => unsafe {
            volatile!((*regs).mode |= 1 << (int as u8)); // Mode bit 1
            volatile!((*regs).edge ^= 1 << (int as u8)); // Edge bit 0
        },
        Trigger::High => unsafe {
            volatile!((*regs).mode ^= 1 << (int as u8)); // Mode bit 0
            volatile!((*regs).level |= 1 << (int as u8)); // Level bit 1
        },
        Trigger::Low => unsafe {
            volatile!((*regs).mode ^= 1 << (int as u8)); // Mode bit 0
            volatile!((*regs).level ^= 1 << (int as u8)); // Level bit 0
        }
    }
}

/// Enables or disables filtering for an interrupt
/// Filtering slows propagation but may reduce spurious interrupts.
pub fn set_filter_enabled(int: Interrupt, enabled: bool) {
    if enabled {
        unsafe { volatile!((*regs).filter |= 1 << (int as u8)) };
    }
    else {
        unsafe { volatile!((*regs).filter ^= 1 << (int as u8)) };
    }
}
