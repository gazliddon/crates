//! JRISC condition flags (G_FLAGS: Z bit0, C bit1, N bit2; I bit3; REGPAGE bit14).
//!
//! Semantics follow MAME's jaguar core:
//! - N is the **29th bit** of the result (`(r >> 29) & 1`) — a hardware quirk
//!   of the JRISC ALU,
//! - C for add = `b > !a`, for sub = `b > a` (unsigned),
//! - conditions are a pure function of (cc, Z, C/N): bit0 requires Z=0,
//!   bit1 requires Z=1, bits 2-3 test C (or N when bit4 is set).

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Flags {
    pub z: bool,
    pub c: bool,
    pub n: bool,
    pub imask: bool,
    pub regpage: bool,
}

impl Flags {
    /// G_FLAGS register value (bits 0-3 + bit 14).
    pub fn bits(&self) -> u32 {
        (self.z as u32)
            | ((self.c as u32) << 1)
            | ((self.n as u32) << 2)
            | ((self.imask as u32) << 3)
            | ((self.regpage as u32) << 14)
    }

    /// Z = (r == 0), N = bit 29 of r (JRISC quirk).
    pub fn set_zn(&mut self, r: u32) {
        self.z = r == 0;
        self.n = (r >> 29) & 1 != 0;
    }

    pub fn set_znc_add(&mut self, a: u32, b: u32, r: u32) {
        self.set_zn(r);
        self.c = b > !a;
    }

    pub fn set_znc_sub(&mut self, a: u32, b: u32, r: u32) {
        self.set_zn(r);
        self.c = b > a;
    }

    /// Evaluate a 5-bit condition code against the current flags.
    pub fn condition(&self, cc: u8) -> bool {
        let cc = cc & 31;
        let v = if cc & 16 != 0 { self.n } else { self.c };
        if cc & 1 != 0 && self.z {
            return false;
        }
        if cc & 2 != 0 && !self.z {
            return false;
        }
        if cc & 4 != 0 && v {
            return false;
        }
        if cc & 8 != 0 && !v {
            return false;
        }
        true
    }
}
