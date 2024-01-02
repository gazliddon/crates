/// Registers, Flags and regiter store
use std::{
    fmt::Debug,
    hash::Hash,
    str::FromStr,
};

use emucore::traits::*;

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Hash, Ord, Eq, PartialEq, PartialOrd, Default)]
pub enum RegEnum {
    #[default]
    A,
    B,
    X,
    PC,
    SP,
    SR,
}

impl RegEnum {}

impl FromStr for RegEnum {
    type Err = ();
    fn from_str(txt: &str) -> Result<Self, Self::Err> {
        let x = txt.to_ascii_lowercase();

        match x.as_str() {
            "a" => Ok(RegEnum::A),
            "b" => Ok(RegEnum::B),
            "x" => Ok(RegEnum::X),
            "*" | "pc" => Ok(RegEnum::PC),
            "sp" => Ok(RegEnum::SP),
            "SR" => Ok(RegEnum::SR),
            _ => Err(()),
        }
    }
}

impl std::fmt::Display for RegEnum {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(_f, "{self:?}")
    }
}

impl RegEnumTrait for RegEnum {
    fn get_size_bytes(&self) -> usize {
        match self {
            RegEnum::A => 1,
            RegEnum::B => 1,
            RegEnum::X => 2,
            RegEnum::PC => 2,
            RegEnum::SP => 2,
            RegEnum::SR => 1,
        }
    }
}

pub struct Flags(u8);

bitflags::bitflags! { 
    impl Flags: u8 {
        const H  = 1 << 5;
        const I  = 1 << 4;
        const N  = 1 << 3;
        const Z  = 1 << 2;
        const V  = 1 << 1;
        const C = 1 << 0;
    }
}

pub struct Regs {
    pub a: u8,
    pub b: u8,
    pub x: u16,
    pub pc: u16,
    pub sp: u16,
    pub flags: Flags
}

impl RegistersTrait<RegEnum> for Regs {
    fn get(&self, r: &RegEnum) -> u64 {
        use RegEnum::*;

        match r {
            A => self.a as u64,
            B => self.b as u64,
            X => self.x as u64,
            PC => self.pc as u64,
            SP => self.sp as u64,
            SR => u64::from(self.flags.bits())
        }
    }

    fn set(&mut self, r: &RegEnum, v: u64) {
        use RegEnum::*;
        let v8 = (v & 0xff) as u8;
        let v16 = (v & 0xffff) as u16;

        match r {
            A => self.a = v8,
            B => self.b = v8,
            X => self.x = v16,
            PC => self.pc = v16,
            SP => self.sp = v16,
            SR => self.flags = Flags(v8)
        }
    }
}


