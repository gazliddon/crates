/// Registers, Flags and regiter store
use std::{fmt::Debug, hash::Hash, str::FromStr};
use serde::{Deserialize, Serialize};
use crate::cpu_core::{ StatusReg, RegEnum };
use super::StatusRegTrait;


////////////////////////////////////////////////////////////////////////////////
pub trait RegisterFileTrait : std::fmt::Display {

    fn set_reg_8(&mut self, r: RegEnum, val: u8) -> &mut Self;
    fn set_reg_16(&mut self, r: RegEnum, val: u16) -> &mut Self;
    fn get_reg_8(&self, r: RegEnum) -> u8;
    fn get_reg_16(&self, r: RegEnum) -> u16;

    #[inline]
    fn set_a(&mut self, val: u8) -> &mut Self {
        self.set_reg_8(RegEnum::A, val)
    }

    #[inline]
    fn set_b(&mut self, val: u8) -> &mut Self {
        self.set_reg_8(RegEnum::B, val)
    }

    #[inline]
    fn set_x(&mut self, val: u16) -> &mut Self {
        self.set_reg_16(RegEnum::X, val)
    }

    #[inline]
    fn set_sp(&mut self, val: u16) -> &mut Self {
        self.set_reg_16(RegEnum::SP, val)
    }

    #[inline]
    fn set_sr(&mut self, val: u8) -> &mut Self {
        self.set_reg_8(RegEnum::SR, val)
    }

    #[inline]
    fn a(&mut self) -> u8 {
        self.get_reg_8(RegEnum::A)
    }

    #[inline]
    fn b(&mut self) -> u8 {
        self.get_reg_8(RegEnum::B)
    }

    #[inline]
    fn x(&mut self) -> u16 {
        self.get_reg_16(RegEnum::X)
    }

    #[inline]
    fn sr(&mut self) -> u8 {
        self.get_reg_8(RegEnum::SR)
    }

    #[inline]
    fn pc(&mut self) -> u16 {
        self.get_reg_16(RegEnum::PC)
    }

    #[inline]
    fn sp(&mut self) -> u16 {
        self.get_reg_16(RegEnum::SP)
    }

    #[inline]
    fn set_pc(&mut self, pc: u16) -> &mut Self {
        self.set_reg_16(RegEnum::PC, pc)
    }

    #[inline]
    fn inc_pc(&mut self) -> &mut Self {
        let pc = self.pc().wrapping_add(1);
        self.set_pc(pc)
    }
}

impl StatusRegTrait for RegisterFile {
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

impl RegisterFileTrait for RegisterFile {
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

#[derive(Clone,Debug,PartialEq, Default, Copy)]
pub struct RegisterFile {
    pub a: u8,
    pub b: u8,
    pub x: u16,
    pub pc: u16,
    pub sp: u16,
    pub flags: StatusReg,
}

impl std::fmt::Display for RegisterFile {
    // TODO file this in 
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(_f, "A  B  X    PC   SP     F")?;
        write!(_f, "{:02x} {:02x} {:04x} {:04x} {:04x} {:?}",self.a, self.b,self.x,self.pc,self.sp, self.flags )
    }
}

impl RegisterFile {
    #[inline]
    fn set_status_reg(&mut self, f: StatusReg, val: bool) -> &mut Self {
        self.flags.set(f, val);
        self
    }
}

