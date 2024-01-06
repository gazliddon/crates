use super::registers::*;
use std::collections::HashSet;

struct DebugRegisterFile<'a> {
    writable_flags: Flags,
    readable_flags: Flags,
    regs_read: HashSet<RegEnum>,
    regs_write: HashSet<RegEnum>,
    regs: &'a mut Regs,
}

enum DebugRegsError {
    FlagRead(Flags),
    FlagWrite(Flags),
    RegisterRead(RegEnum),
    RegisterWrite(RegEnum),
}

impl<'a> DebugRegisterFile<'a> {
    pub fn new(regs: &'a mut Regs) -> Self {
        Self {
            regs,
            regs_read: Default::default(),
            regs_write: Default::default(),
            writable_flags: Default::default(),
            readable_flags: Default::default(),
        }
    }

    #[inline]
    pub fn check_reg_read(&self, r: RegEnum) -> bool {
        self.regs_read.contains(&r)
    }

    #[inline]
    pub fn check_reg_write(&self, r: RegEnum) -> bool {
        self.regs_write.contains(&r)
    }

    #[inline]
    pub fn check_flag_read(&self, f: Flags) -> bool {
        self.readable_flags.contains(f)
    }
    
    #[inline]
    pub fn check_flag_write(&self, f: Flags) -> bool {
        self.writable_flags.contains(f)
    }
}

impl<'a> RegisterFileTrait for DebugRegisterFile<'a> {
    #[inline]
    fn set_reg_8(&mut self,r : RegEnum, val: u8) -> &mut Self{
        self.regs.set_reg_8(r, val);
        self
    }

    #[inline]
    fn set_reg_16(&mut self,r : RegEnum, val: u16) -> &mut Self{
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

    #[inline]
    fn set_n(&mut self, val: bool) -> &mut Self{
        self.regs.set_n(val);
        self
    }

    #[inline]
    fn set_v(&mut self, val: bool) -> &mut Self{
        self.regs.set_v(val);
        self

    }

    #[inline]
    fn set_c(&mut self, val: bool) -> &mut Self{
        self.regs.set_c(val);
        self
    }

    #[inline]
    fn set_h(&mut self, val: bool) -> &mut Self{
        self.regs.set_h(val);
        self
    }

    #[inline]
    fn set_i(&mut self, val: bool) -> &mut Self{
        self.regs.set_i(val);
        self
    }

    #[inline]
    fn set_z(&mut self, val: bool) -> &mut Self{
        self.regs.set_z(val);
        self
    }

    #[inline]
    fn n(&mut self) -> bool {
        self.regs.n()
    }

    #[inline]
    fn v(&mut self) -> bool {
        self.regs.flags.contains(Flags::V)
    }

    #[inline]
    fn c(&mut self) -> bool {
        self.regs.c()
    }

    #[inline]
    fn h(&mut self) -> bool {
        self.regs.h()
    }

    #[inline]
    fn i(&mut self) -> bool {
        self.regs.i()
    }

    #[inline]
    fn z(&mut self) -> bool {
        self.regs.z()
    }
}
