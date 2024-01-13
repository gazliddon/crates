use super::{CpuResult, RegisterFileTrait, StatusRegTrait};
use crate::cpu::Ins;

use emucore::{
    mem::{MemResult, MemoryIO},
    traits::FlagsTrait,
};

pub struct Machine<M, R>
where
    M: MemoryIO,
    R: RegisterFileTrait + StatusRegTrait,
{
    pub regs: R,
    pub mem: M,
    pub cycle: usize,
}

fn u8_sign_extend(byte: u8) -> u16 {
    if (byte & 0x80) == 0x80 {
        byte as u16
    } else {
        (byte as u16) | 0xff00
    }
}

impl<M, R> Machine<M, R>
where
    M: MemoryIO,
    R: RegisterFileTrait + StatusRegTrait,
{
    pub fn step(&mut self) -> CpuResult<()> {
        use super::addrmodes::*;

        let addr = self.regs.pc();
        let op_code = self.mem_mut().load_byte(addr as usize)?;
        self.regs.inc_pc();

        macro_rules! handle_op {
            ($action:ident, $addr:ident, $cycles:expr, $size:expr) => {{
                let mut ins = Ins {
                    bus: $addr {},
                    m: self,
                };
                ins.$action()?;
            }};
        }

        let _ = op_table!(op_code, { panic!("NOT IMP") });

        Ok(())
    }

    pub fn reset(&mut self) -> MemResult<()> {
        let v = self.mem_mut().load_word(0xfffe)?;
        self.regs.set_pc(v);
        self.regs.sei();
        Ok(())
    }

    pub fn new(mem: M, regs: R) -> Self {
        Self {
            mem,
            regs,
            cycle: 0,
        }
    }

    pub fn mem_mut(&mut self) -> &mut M {
        &mut self.mem
    }

    #[inline]
    pub fn inc_pc(&mut self) {
        self.regs.inc_pc();
    }

    #[inline]
    pub fn inc_inc_pc(&mut self) {
        self.regs.inc_pc();
        self.regs.inc_pc();
    }

    #[inline]
    pub fn fetch_rel_addr(&mut self) -> MemResult<u16> {
        let pc = self.regs.pc();
        let byte = self.mem.load_byte(pc as usize)?;
        let res = pc.wrapping_add(u8_sign_extend(byte));
        Ok(res)
    }

    pub fn fetch_byte(&mut self) -> MemResult<u8> {
        let pc = self.regs.pc();
        let byte = self.mem.load_byte(pc as usize)?;
        self.inc_pc();
        Ok(byte)
    }

    // pub fn fetch_word(&mut self) -> MemResult<u16> {
    //     let pc = self.regs.pc();
    //     let b = self.mem.load_word(pc as usize)?;
    //     self.inc_inc_pc();
    //     Ok(b)
    // }

    pub fn mem(&self) -> &M {
        &self.mem
    }

    // [[SP]] ← [val(LO)],
    // [[SP] - 1] ← [val(HI)],
    // [SP] ← [SP] - 2,
    pub fn push_word(&mut self, val: u16) -> MemResult<()> {
        let sp = self.regs.sp().wrapping_sub(1);
        self.mem.store_word(sp as usize, val)?;
        self.regs.set_sp(sp.wrapping_sub(1));
        Ok(())
    }

    // [[SP]] ← [A], [SP] ← [SP] - 1
    pub fn push_byte(&mut self, val: u8) -> MemResult<()> {
        let sp = self.regs.sp();
        self.mem.store_byte(sp as usize, val)?;
        self.regs.set_sp(sp.wrapping_sub(1));
        Ok(())
    }

    // [res(HI)] ← [[SP] + 1],
    // [res(LO)] ← [[SP] + 2],
    // [SP] ← [SP] + 2
    pub fn pop_word(&mut self) -> MemResult<u16> {
        let sp = self.regs.sp().wrapping_add(1);
        let word = self.mem.load_word(sp as usize)?;
        self.regs.set_sp(sp.wrapping_add(1));
        Ok(word)
    }

    //[SP] ← [SP] + 1, [A] ← [[SP]]
    pub fn pop_byte(&mut self) -> MemResult<u8> {
        let sp = self.regs.sp().wrapping_add(1);
        let byte = self.mem.load_byte(sp as usize)?;
        self.regs.set_sp(sp);
        Ok(byte)
    }
}
