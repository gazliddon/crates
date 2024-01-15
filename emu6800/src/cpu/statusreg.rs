use serde::{Deserialize, Serialize};

use super::opcodes::UnsignedVal;
use crate::cpu_core::StatusReg;

pub trait StatusRegTrait {
    fn set_n(&mut self, val: bool) -> &mut Self;
    fn set_v(&mut self, val: bool) -> &mut Self;
    fn set_c(&mut self, val: bool) -> &mut Self;
    fn set_h(&mut self, val: bool) -> &mut Self;
    fn set_i(&mut self, val: bool) -> &mut Self;
    fn set_z(&mut self, val: bool) -> &mut Self;

    fn n(&self) -> bool;
    fn v(&self) -> bool;
    fn c(&self) -> bool;
    fn h(&self) -> bool;
    fn i(&self) -> bool;
    fn z(&self) -> bool;

    fn neg(&self) -> bool {
        self.n()
    }

    fn eq(&self) -> bool {
        self.z()
    }

    fn ls(&self) -> bool {
        self.z() || self.c()
    }

    /// less than or equal to zero
    fn le(&self) -> bool {
        self.z() || (self.n() != self.v())
    }

    fn lt(&self) -> bool {
        self.n() != self.v()
    }

    fn plus(&self) -> bool {
        !self.neg()
    }

    fn ne(&self) -> bool {
        !self.eq()
    }

    fn hi(&self) -> bool {
        !self.c() && !self.z()
    }

    fn gt(&self) -> bool {
        !self.z() && (self.n() == self.v())
    }

    fn ge(&self) -> bool {
        self.n() == self.v()
    }

    // setters
    fn cln(&mut self) -> &mut Self {
        self.set_n(false)
    }

    fn sen(&mut self) -> &mut Self {
        self.set_n(true)
    }

    fn clv(&mut self) -> &mut Self {
        self.set_v(false)
    }

    fn sev(&mut self) -> &mut Self {
        self.set_v(true)
    }

    fn clc(&mut self) -> &mut Self {
        self.set_c(false)
    }
    fn sec(&mut self) -> &mut Self {
        self.set_c(true)
    }

    fn set_nz_from_u8(&mut self, val: u8) -> &mut Self {
        let n = val.bit(7);
        let z = val == 0;
        self.set_n(n).set_z(z)
    }

    fn set_nz_from_u16(&mut self, val: u16) -> &mut Self {
        let n = val & 0x8000 == 0x8000;
        let z = val == 0x0000;
        self.set_n(n).set_z(z)
    }

    fn clh(&mut self) -> &mut Self {
        self.set_h(false)
    }

    fn seh(&mut self) -> &mut Self {
        self.set_h(true)
    }

    fn cli(&mut self) -> &mut Self {
        self.set_i(false)
    }
    fn sei(&mut self) -> &mut Self {
        self.set_i(true)
    }

    fn clz(&mut self) -> &mut Self {
        self.set_z(false)
    }
    fn sez(&mut self) -> &mut Self {
        self.set_z(true)
    }
}

impl StatusRegTrait for StatusReg {
    fn set_n(&mut self, val: bool) -> &mut Self {
        self.set(StatusReg::N, val);
        self
    }

    fn set_v(&mut self, val: bool) -> &mut Self {
        self.set(StatusReg::V, val);
        self
    }
    fn set_c(&mut self, val: bool) -> &mut Self {
        self.set(StatusReg::C, val);
        self
    }
    fn set_h(&mut self, val: bool) -> &mut Self {
        self.set(StatusReg::H, val);
        self
    }
    fn set_i(&mut self, val: bool) -> &mut Self {
        self.set(StatusReg::I, val);
        self
    }
    fn set_z(&mut self, val: bool) -> &mut Self {
        self.set(StatusReg::Z, val);
        self
    }

    fn n(&self) -> bool {
        self.contains(StatusReg::N)
    }

    fn v(&self) -> bool {
        self.contains(StatusReg::V)
    }
    fn c(&self) -> bool {
        self.contains(StatusReg::C)
    }
    fn h(&self) -> bool {
        self.contains(StatusReg::H)
    }
    fn i(&self) -> bool {
        self.contains(StatusReg::I)
    }
    fn z(&self) -> bool {
        self.contains(StatusReg::Z)
    }
}
