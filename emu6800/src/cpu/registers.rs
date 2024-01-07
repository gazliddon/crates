/// Registers, Flags and regiter store
use std::{fmt::Debug, hash::Hash, str::FromStr};

use bitflags::Flags;
use emucore::traits::*;
use serde::{Deserialize, Serialize};

use super::{StatusReg, StatusRegTrait};

////////////////////////////////////////////////////////////////////////////////
pub trait RegisterFileTrait {
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

    // fn set_n(&mut self, val: bool) -> &mut Self;
    // fn set_v(&mut self, val: bool) -> &mut Self;
    // fn set_c(&mut self, val: bool) -> &mut Self;
    // fn set_h(&mut self, val: bool) -> &mut Self;
    // fn set_i(&mut self, val: bool) -> &mut Self;
    // fn set_z(&mut self, val: bool) -> &mut Self;

    // fn n(&self) -> bool;
    // fn v(&self) -> bool;
    // fn c(&self) -> bool;
    // fn h(&self) -> bool;
    // fn i(&self) -> bool;
    // fn z(&self) -> bool;

    // fn hi(&self) -> bool {
    //     panic!()
    // }

    // fn gt(&self) -> bool {
    //     panic!()
    // }

    // fn le(&self) -> bool {
    //     panic!()
    // }

    // fn ls(&self) -> bool {
    //     panic!()
    // }

    // fn ge(&self) -> bool {
    //     panic!()
    // }

    // fn cln(&mut self) -> &mut Self {
    //     self.set_n(false);
    //     self
    // }
    // fn sen(&mut self) -> &mut Self {
    //     self.set_n(true);
    //     self
    // }

    // fn clv(&mut self) -> &mut Self {
    //     self.set_v(false);
    //     self
    // }
    // fn sev(&mut self) -> &mut Self {
    //     self.set_v(true);
    //     self
    // }

    // fn clc(&mut self) -> &mut Self {
    //     self.set_c(false);
    //     self
    // }
    // fn sec(&mut self) -> &mut Self {
    //     self.set_c(true);
    //     self
    // }

    // fn set_nz_from_u8(&mut self, val: u8) -> &mut Self {
    //     let n = val & 0x80 == 0x80;
    //     let z = val == 0x0000;
    //     self.set_n(n).set_z(z)
    // }

    // fn set_nz_from_u16(&mut self, val: u16) -> &mut Self {
    //     let n = val & 0x8000 == 0x8000;
    //     let z = val == 0x0000;
    //     self.set_n(n).set_z(z)
    // }

    // fn clh(&mut self) -> &mut Self {
    //     self.set_h(false);
    //     self
    // }
    // fn seh(&mut self) -> &mut Self {
    //     self.set_h(true);
    //     self
    // }

    // fn cli(&mut self) -> &mut Self {
    //     self.set_i(false);
    //     self
    // }
    // fn sei(&mut self) -> &mut Self {
    //     self.set_i(true);
    //     self
    // }

    // fn clz(&mut self) -> &mut Self {
    //     self.set_z(false);
    //     self
    // }
    // fn sez(&mut self) -> &mut Self {
    //     self.set_z(true);
    //     self
    // }

    // fn lt(&mut self) -> bool {
    //     panic!()
    // }
}

impl StatusRegTrait for Regs {
    #[inline]
    fn set_n(&mut self, val: bool) -> &mut Self {
        self.flags.set(StatusReg::N, val);
        self
    }

    #[inline]
    fn set_v(&mut self, val: bool) -> &mut Self {
        self.flags.set(StatusReg::V, val);
        self
    }

    #[inline]
    fn set_c(&mut self, val: bool) -> &mut Self {
        self.flags.set(StatusReg::C, val);
        self
    }

    #[inline]
    fn set_h(&mut self, val: bool) -> &mut Self {
        self.flags.set(StatusReg::H, val);
        self
    }

    #[inline]
    fn set_i(&mut self, val: bool) -> &mut Self {
        self.flags.set(StatusReg::I, val);
        self
    }

    #[inline]
    fn set_z(&mut self, val: bool) -> &mut Self {
        self.flags.set(StatusReg::Z, val);
        self
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

#[derive(Clone,Debug,PartialEq)]
pub struct Regs {
    pub a: u8,
    pub b: u8,
    pub x: u16,
    pub pc: u16,
    pub sp: u16,
    pub flags: StatusReg,
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
