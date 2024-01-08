use super::registers::*;
use super::statusreg::*;
use std::collections::HashSet;

pub struct DebugRegisterFile<'a> {
    flags_that_will_alter: StatusReg,
    regs_read: HashSet<RegEnum>,
    regs_write: HashSet<RegEnum>,
    regs: &'a mut RegisterFile,
    flags_altered: StatusReg,
}

pub enum DebugRegsErrorKind {
    FlagsNotAltered(StatusReg),
    RegisterRead(RegEnum),
    RegisterWrite(RegEnum),
}

pub struct DebugRegsError {
    kind: DebugRegsErrorKind,
    regs: RegisterFile,
}

impl<'a> std::fmt::Display for DebugRegisterFile<'a> {
    // TODO file this in 
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(_f,"{}", self.regs)?;
        write!(_f,"Fill this in dummy")
    }
}

impl<'a> DebugRegisterFile<'a> {
    pub fn new(regs: &'a mut RegisterFile, altered_flags: StatusReg) -> Self {
        Self {
            regs,
            regs_read: Default::default(),
            regs_write: Default::default(),
            flags_that_will_alter: altered_flags,
            flags_altered: Default::default(),
        }
    }

    #[inline]
    fn check_reg_read(&self, r: RegEnum) -> bool {
        self.regs_read.contains(&r)
    }

    #[inline]
    fn check_reg_write(&self, r: RegEnum) -> bool {
        self.regs_write.contains(&r)
    }

    #[inline]
    fn set_write_status_reg(&mut self, f: StatusReg) -> &mut Self{
        self.flags_altered.set(f, true);
        self
    }

    #[inline]
    fn set_write_register(&mut self, _r: RegEnum) -> &mut Self{
        self
    }

    #[inline]
    fn not_altered(&self) -> StatusReg {
        self.flags_that_will_alter - self.flags_altered
    }

    pub fn get_flags_error(&self) -> Result<(), DebugRegsError> {
        let not_altered = self.not_altered();

        if not_altered.is_empty() {
            Ok(())
        } else {
            Err(DebugRegsError {
                kind: DebugRegsErrorKind::FlagsNotAltered(not_altered),
                regs: self.regs.clone(),
            })
        }
    }
}

impl<'a> RegisterFileTrait for DebugRegisterFile<'a> {
    #[inline]
    fn set_reg_8(&mut self, r: RegEnum, val: u8) -> &mut Self {
        self.regs.set_reg_8(r, val);
        self
    }

    #[inline]
    fn set_reg_16(&mut self, r: RegEnum, val: u16) -> &mut Self {
        self.regs.set_reg_16(r, val);
        self
    }

    #[inline]
    fn get_reg_8(&self, r: RegEnum) -> u8 {
        self.regs.get_reg_8(r)
    }

    #[inline]
    fn get_reg_16(&self, r: RegEnum) -> u16 {
        self.regs.get_reg_16(r)
    }

}

impl<'a> StatusRegTrait for DebugRegisterFile<'a> {
    #[inline]
    fn set_n(&mut self, val: bool) -> &mut Self {
        self.regs.set_n(val);
        self.set_write_status_reg(StatusReg::N)
    }

    #[inline]
    fn set_v(&mut self, val: bool) -> &mut Self {
        self.regs.set_v(val);
        self.set_write_status_reg(StatusReg::V)
    }

    #[inline]
    fn set_c(&mut self, val: bool) -> &mut Self {
        self.regs.set_c(val);
        self.set_write_status_reg(StatusReg::C)
    }

    #[inline]
    fn set_h(&mut self, val: bool) -> &mut Self {
        self.regs.set_h(val);
        self.set_write_status_reg(StatusReg::H)
    }

    #[inline]
    fn set_i(&mut self, val: bool) -> &mut Self {
        self.regs.set_i(val);
        self.set_write_status_reg(StatusReg::I)
    }

    #[inline]
    fn set_z(&mut self, val: bool) -> &mut Self {
        self.regs.set_z(val);
        self.set_write_status_reg(StatusReg::Z)
    }

    #[inline]
    fn n(&self) -> bool {
        self.regs.n()
    }

    #[inline]
    fn v(&self) -> bool {
        self.regs.v()
    }

    #[inline]
    fn c(&self) -> bool {
        self.regs.c()
    }

    #[inline]
    fn h(&self) -> bool {
        self.regs.h()
    }

    #[inline]
    fn i(&self) -> bool {
        self.regs.i()
    }

    #[inline]
    fn z(&self) -> bool {
        self.regs.z()
    }
}
