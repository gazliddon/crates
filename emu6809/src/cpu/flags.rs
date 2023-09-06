use serde::{Deserialize, Serialize};

bitflags::bitflags! {

#[derive( Default, Serialize, Deserialize)]
    pub struct Flags: u8 {

        const E  = 1 << 7;
        const F  = 1 << 6;
        const H  = 1 << 5;
        const I  = 1 << 4;
        const N  = 1 << 3;
        const Z  = 1 << 2;
        const V  = 1 << 1;
        const C  = 1 ;

        const NZVCH =
            Self::N.bits |
            Self::Z.bits |
            Self::V.bits |
            Self::C.bits |
            Self::H.bits ;

        const NZVC =
            Self::N.bits |
            Self::Z.bits |
            Self::V.bits |
            Self::C.bits ;

        const NZC = Self::N.bits | Self::Z.bits | Self::C.bits ;
        const NZ  = Self::N.bits | Self::Z.bits ;
        const NZV = Self::N.bits | Self::Z.bits | Self::V.bits ;
    }
}

impl Flags {
    pub fn new(val: u8) -> Flags {
        Flags { bits: val }
    }

    #[inline]
    pub fn set_flags(&mut self, val: u8) {
        self.bits = val
    }

    pub fn write_with_mask(&mut self, mask: u8, val: u8) {
        self.bits = (self.bits & !mask) | (val & mask)
    }

    pub fn le(self) -> bool {
        let v = self.contains(Flags::V);
        let n = self.contains(Flags::N);
        let z = self.contains(Flags::Z);
        z | (v ^ n)
    }

    pub fn gt(self) -> bool {
        !self.le()
    }

    pub fn lt(self) -> bool {
        !self.ge()
    }

    pub fn ge(self) -> bool {
        let v = self.contains(Flags::V);
        let n = self.contains(Flags::N);
        !(v ^ n)
    }

    pub fn hi(self) -> bool {
        let c = self.contains(Flags::C);
        let z = self.contains(Flags::Z);
        !(c | z)
    }

    pub fn ls(self) -> bool {
        !self.hi()
    }
}
