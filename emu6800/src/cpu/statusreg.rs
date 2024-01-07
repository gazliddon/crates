use serde::{Deserialize, Serialize};

bitflags::bitflags! {
    #[derive(Clone, Copy, Debug,Deserialize, Serialize, PartialEq)]
    #[serde(transparent)]
    pub struct StatusReg : u8
        {
            const H  = 1 << 5;
            const I  = 1 << 4;
            const N  = 1 << 3;
            const Z  = 1 << 2;
            const V  = 1 << 1;
            const C = 1 << 0;
        }
}

impl Default for StatusReg {
    fn default() -> Self {
        Self(Default::default())
    }
}

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

    fn hi(&self) -> bool {
        panic!()
    }

    fn gt(&self) -> bool {
        panic!()
    }

    fn le(&self) -> bool {
        panic!()
    }

    fn ls(&self) -> bool {
        panic!()
    }

    fn ge(&self) -> bool {
        panic!()
    }

    fn cln(&mut self) -> &mut Self {
        self.set_n(false);
        self
    }
    fn sen(&mut self) -> &mut Self {
        self.set_n(true);
        self
    }

    fn clv(&mut self) -> &mut Self {
        self.set_v(false);
        self
    }
    fn sev(&mut self) -> &mut Self {
        self.set_v(true);
        self
    }

    fn clc(&mut self) -> &mut Self {
        self.set_c(false);
        self
    }
    fn sec(&mut self) -> &mut Self {
        self.set_c(true);
        self
    }

    fn set_nz_from_u8(&mut self, val: u8) -> &mut Self {
        let n = val & 0x80 == 0x80;
        let z = val == 0x0000;
        self.set_n(n).set_z(z)
    }

    fn set_nz_from_u16(&mut self, val: u16) -> &mut Self {
        let n = val & 0x8000 == 0x8000;
        let z = val == 0x0000;
        self.set_n(n).set_z(z)
    }

    fn clh(&mut self) -> &mut Self {
        self.set_h(false);
        self
    }
    fn seh(&mut self) -> &mut Self {
        self.set_h(true);
        self
    }

    fn cli(&mut self) -> &mut Self {
        self.set_i(false);
        self
    }
    fn sei(&mut self) -> &mut Self {
        self.set_i(true);
        self
    }

    fn clz(&mut self) -> &mut Self {
        self.set_z(false);
        self
    }
    fn sez(&mut self) -> &mut Self {
        self.set_z(true);
        self
    }

    fn lt(&mut self) -> bool {
        panic!()
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

