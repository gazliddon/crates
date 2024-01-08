/// Registers, Flags and regiter store
use std::{fmt::Debug, hash::Hash, str::FromStr};

use bitflags::Flags;
use emucore::traits::*;
use serde::{Deserialize, Serialize};

use super::{StatusReg, StatusRegTrait};

////////////////////////////////////////////////////////////////////////////////
pub trait RegisterFileTrait : std::fmt::Display{
    fn set_reg_8(&mut self, r: RegEnum, val: u8) -> &mut Self;
    fn set_reg_16(&mut self, r: RegEnum, val: u16) -> &mut Self;
    fn get_reg_8(&self, r: RegEnum) -> u8;
    fn get_reg_16(&self, r: RegEnum) -> u16;

    fn set_a(&mut self, val: u8) -> &mut Self {
        self.set_reg_8(RegEnum::A, val)
    }

    fn set_b(&mut self, val: u8) -> &mut Self {
        self.set_reg_8(RegEnum::B, val)
    }

    fn set_x(&mut self, val: u16) -> &mut Self {
        self.set_reg_16(RegEnum::X, val)
    }

    fn set_sp(&mut self, val: u16) -> &mut Self {
        self.set_reg_16(RegEnum::SP, val)
    }

    fn set_sr(&mut self, val: u8) -> &mut Self {
        self.set_reg_8(RegEnum::SR, val)
    }

    fn a(&mut self) -> u8 {
        self.get_reg_8(RegEnum::A)
    }

    fn b(&mut self) -> u8 {
        self.get_reg_8(RegEnum::B)
    }

    fn x(&mut self) -> u16 {
        self.get_reg_16(RegEnum::X)
    }

    fn sr(&mut self) -> u8 {
        self.get_reg_8(RegEnum::SR)
    }

    fn pc(&mut self) -> u16 {
        self.get_reg_16(RegEnum::PC)
    }

    fn sp(&mut self) -> u16 {
        self.get_reg_16(RegEnum::SP)
    }

    fn set_pc(&mut self, pc: u16) -> &mut Self {
        self.set_reg_16(RegEnum::PC, pc)
    }

    fn inc_pc(&mut self) -> &mut Self {
        let pc = self.pc().wrapping_add(1);
        self.set_pc(pc)
    }
}

impl StatusRegTrait for Regs {
    #[inline]
    fn set_n(&mut self, val: bool) -> &mut Self {
        self.set_status_reg(StatusReg::N, val)
    }

    #[inline]
    fn set_v(&mut self, val: bool) -> &mut Self {
        self.set_status_reg(StatusReg::V, val)
    }

    #[inline]
    fn set_c(&mut self, val: bool) -> &mut Self {
        self.set_status_reg(StatusReg::C, val)
    }

    #[inline]
    fn set_h(&mut self, val: bool) -> &mut Self {
        self.set_status_reg(StatusReg::H, val)
    }

    #[inline]
    fn set_i(&mut self, val: bool) -> &mut Self {
        self.set_status_reg(StatusReg::I, val)
    }

    #[inline]
    fn set_z(&mut self, val: bool) -> &mut Self {
        self.set_status_reg(StatusReg::Z, val)
    }

    #[inline]
    fn n(&self) -> bool {
        self.flags.contains(StatusReg::N)
    }

    #[inline]
    fn v(&self) -> bool {
        self.flags.contains(StatusReg::V)
    }

    #[inline]
    fn c(&self) -> bool {
        self.flags.contains(StatusReg::C)
    }

    #[inline]
    fn h(&self) -> bool {
        self.flags.contains(StatusReg::H)
    }

    #[inline]
    fn i(&self) -> bool {
        self.flags.contains(StatusReg::I)
    }

    #[inline]
    fn z(&self) -> bool {
        self.flags.contains(StatusReg::Z)
    }
}

impl RegisterFileTrait for Regs {
    #[inline]
    fn set_reg_8(&mut self, r: RegEnum, val: u8) -> &mut Self {
        use RegEnum::*;
        match r {
            A => self.a = val,
            B => self.b = val,
            SR => self.flags = StatusReg::from_bits(val).unwrap(),
            _ => panic!(),
        }
        self
    }

    #[inline]
    fn set_reg_16(&mut self, r: RegEnum, val: u16) -> &mut Self {
        use RegEnum::*;
        match r {
            X => self.x = val,
            PC => self.pc = val,
            SP => self.sp = val,
            _ => panic!(),
        }
        self
    }

    #[inline]
    fn get_reg_8(&self, r: RegEnum) -> u8 {
        use RegEnum::*;
        match r {
            A => self.a,
            B => self.b,
            SR => self.flags.bits(),
            _ => panic!(),
        }
    }

    #[inline]
    fn get_reg_16(&self, r: RegEnum) -> u16 {
        use RegEnum::*;
        match r {
            X => self.x,
            PC => self.pc,
            SP => self.sp,
            _ => panic!(),
        }
    }

}

#[derive(
    Copy, Debug, Clone, Hash, Ord, Eq, PartialEq, PartialOrd, Default, Serialize, Deserialize,
)]

pub enum RegEnum {
    #[default]
    A,
    B,
    X,
    PC,
    SP,
    SR,
}

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

#[derive(Clone,Debug,PartialEq, Default)]
pub struct Regs {
    pub a: u8,
    pub b: u8,
    pub x: u16,
    pub pc: u16,
    pub sp: u16,
    pub flags: StatusReg,
}

impl std::fmt::Display for Regs {
    // TODO file this in 
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Regs {
    fn set_status_reg(&mut self, f: StatusReg, val: bool) -> &mut Self {
        self.flags.set(f, val);
        self
    }
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
            SR => u64::from(self.flags.bits()),
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
            SR => self.flags = StatusReg::from_bits(v8).unwrap(),
        }
    }
}
