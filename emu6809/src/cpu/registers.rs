use std::fmt::Display;

use super::Flags;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum RegEnum {
    A,
    B,
    X,
    Y,
    U,
    S,
    D,
    DP,
    CC,
    PC,
}

impl Display for RegEnum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::A => "A",
            Self::B => "B",
            Self::X => "X",
            Self::Y => "Y",
            Self::U => "U",
            Self::S => "S",
            Self::D => "D",
            Self::DP => "DP",
            Self::CC => "CC",
            Self::PC => "PC",
        };
        write!(f, "{s}")
    }
}

impl RegEnum {
    /// Is this register okay to use as an index?
    pub fn is_valid_for_index(&self) -> bool {
        use RegEnum::*;
        matches!(self, X | Y | S | U)
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
        match *r {
            RegEnum::A => self.a = val as u8,
            RegEnum::B => self.b = val as u8,
            RegEnum::X => self.x = val,
            RegEnum::Y => self.y = val,
            RegEnum::U => self.u = val,
            RegEnum::S => self.s = val,
            RegEnum::D => self.set_d(val),
            RegEnum::DP => self.dp = val as u8,
            RegEnum::CC => self.flags = Flags::new(val as u8),
            RegEnum::PC => self.pc = val,
        }
    }

    pub fn get(&self, r: &RegEnum) -> u16 {
        match *r {
            RegEnum::A => u16::from(self.a),
            RegEnum::B => u16::from(self.b),
            RegEnum::X => self.x,
            RegEnum::Y => self.y,
            RegEnum::U => self.u,
            RegEnum::S => self.s,
            RegEnum::D => self.get_d(),
            RegEnum::DP => u16::from(self.dp),
            RegEnum::CC => u16::from(self.flags.bits()),
            RegEnum::PC => self.pc,
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
