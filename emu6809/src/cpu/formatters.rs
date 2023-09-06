use super::{Flags, Regs};

use std::fmt;

impl Regs {
    pub fn get_hdr(&self) -> String {
        "PC   D    A  B  X    Y    U    S    DP : flags".to_string()
    }
    pub fn get_text(&self) -> String {
        format!(
            "{:04x} {:04x} {:02x} {:02x} {:04x} {:04x} {:04x} {:04x} {:02x} : {}",
            self.pc,
            self.get_d(),
            self.a,
            self.b,
            self.x,
            self.y,
            self.u,
            self.s,
            self.dp,
            self.flags
        )
    }
}

impl fmt::Display for Regs {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}\n{}", self.get_hdr(), self.get_text(),)
    }
}

impl fmt::Display for Flags {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:08b} : {:?}", self.bits(), *self)
    }
}
