use super::{ RegisterFileTrait};
use emucore::mem::{MemResult, MemoryIO};

pub struct Machine<M, R >
where
    M: MemoryIO,
    R: RegisterFileTrait,
{
    pub regs: R,
    pub mem: M,
}

fn u8_sign_extend(byte: u8) -> u16 {
    if (byte & 0x80) == 0x80 {
        byte as u16
    } else {
        (byte as u16) | 0xff00
    }
}

impl<M,R> Machine<M,R>
where
    M: MemoryIO,
    R: RegisterFileTrait,
{
    pub fn mem_mut(&mut self) -> &mut M {
        &mut self.mem
    }

    #[inline]
    pub fn inc_pc(&mut self) {
        panic!()
    }
    #[inline]
    pub fn inc_inc_pc(&mut self) {
        panic!()
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

    pub fn fetch_word(&mut self) -> MemResult<u16> {
        let pc = self.regs.pc();
        let b = self.mem.load_word(pc as usize)?;
        self.inc_inc_pc();
        Ok(b)
    }

    pub fn mem(&self) -> &M {
        &self.mem
    }

    pub fn push_word(&mut self, val: u16) -> MemResult<()> {
        let sp = self.regs.sp();
        self.mem.store_word(sp as usize, val)?;
        self.regs.set_sp(sp.wrapping_sub(2));
        Ok(())
    }

    pub fn push_byte(&mut self, val: u8) -> MemResult<()> {
        let sp = self.regs.sp();
        self.mem.store_byte(sp as usize, val)?;
        self.regs.set_sp(sp.wrapping_sub(1));
        Ok(())
    }

    pub fn pop_word(&mut self) -> MemResult<u16> {
        let sp = self.regs.sp().wrapping_add(2);
        let word = self.mem.load_word(sp as usize)?;
        self.regs.set_sp(sp);
        Ok(word)
    }

    pub fn pop_byte(&mut self) -> MemResult<u8> {
        let sp = self.regs.sp().wrapping_add(1);
        let byte = self.mem.load_byte(sp as usize)?;
        self.regs.set_sp(sp);
        Ok(byte)
    }
}
