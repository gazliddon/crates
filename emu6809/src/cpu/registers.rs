#![deny(unused_imports)]
use std::{fmt::Display, str::FromStr};

use super::Flags;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy, PartialOrd, Ord)]
pub enum RegEnum {
    A,
    B,
    D,
    X,
    Y,
    U,
    S,
    DP,
    CC,
    PC,
}

impl Display for RegEnum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use RegEnum::*;
        let s = match self {
            A => "A",
            B => "B",
            D => "D",
            X => "X",
            Y => "Y",
            U => "U",
            S => "S",
            DP => "DP",
            CC => "CC",
            PC => "PC",
        };
        write!(f, "{s}")
    }
}

impl FromStr for RegEnum {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let txt = s.to_lowercase();
        match txt.as_str() {
            "a" => Ok( Self::A ),
            "b" => Ok( Self::B ),
            "d" => Ok( Self::D ),
            "x" => Ok( Self::X ),
            "y" => Ok( Self::Y ),
            "u" => Ok( Self::U ),
            "s" => Ok( Self::S ),
            "dp" => Ok( Self::DP ),
            "cc" => Ok( Self::CC ),
            "pc" => Ok( Self::PC),
            _ => Err(())

        }
    }
}

impl RegEnum {
    /// Is this register okay to use as an index?
    pub fn valid_for_index(&self) -> bool {
        use RegEnum::*;
        matches!(self, X | Y | S | U)
    }
    pub fn valid_abd(&self) -> bool {
        use RegEnum::*;
        matches!(self, A | B | D )
    }
}

use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct Regs {
    pub a: u8,
    pub b: u8,
    pub x: u16,
    pub y: u16,
    pub u: u16,
    pub s: u16,
    pub pc: u16,
    pub dp: u8,
    pub flags: Flags,
}

impl Regs {
    pub fn set(&mut self, r: &RegEnum, val: u16) {
        use RegEnum::*;
        match *r {
            A => self.a = val as u8,
            B => self.b = val as u8,
            D => self.set_d(val),
            X => self.x = val,
            Y => self.y = val,
            U => self.u = val,
            S => self.s = val,
            DP => self.dp = val as u8,
            CC => self.flags = Flags::new(val as u8),
            PC => self.pc = val,
        }
    }

    pub fn get(&self, r: &RegEnum) -> u16 {
        use RegEnum::*;
        match *r {
            A => u16::from(self.a),
            B => u16::from(self.b),
            D => self.get_d(),
            X => self.x,
            Y => self.y,
            U => self.u,
            S => self.s,
            DP => u16::from(self.dp),
            CC => u16::from(self.flags.bits()),
            PC => self.pc,
        }
    }

    pub fn wrapping_add_and_set(&mut self, r: &RegEnum, v: u16) -> u16 {
        let mut rv = self.get(r);
        rv = rv.wrapping_add(v);
        self.set(r, rv);
        rv
    }

    pub fn inc(&mut self, r: &RegEnum) -> u16 {
        self.wrapping_add_and_set(r, 1)
    }

    pub fn incinc(&mut self, r: &RegEnum) -> u16 {
        self.wrapping_add_and_set(r, 2)
    }

    pub fn dec(&mut self, r: &RegEnum) -> u16 {
        self.wrapping_add_and_set(r, 0xffff)
    }

    pub fn decdec(&mut self, r: &RegEnum) -> u16 {
        self.wrapping_add_and_set(r, 0xfffe)
    }

    pub fn get_dp_ptr(&self) -> u16 {
        (u16::from(self.dp)) << 8
    }

    pub fn get_d(&self) -> u16 {
        (u16::from(self.a) << 8) | u16::from(self.b)
    }

    pub fn set_d(&mut self, d: u16) {
        self.a = (d >> 8) as u8;
        self.b = d as u8;
    }

    pub fn new() -> Regs {
        Regs {
            a: 0,
            b: 0,
            x: 0,
            y: 0,
            u: 0,
            s: 0,
            pc: 0,
            dp: 0,
            flags: Flags::new(0),
        }
    }
}
