use std::u8;

use emucore::mem::MemResult;
use emucore::mem::MemoryIO;
use serde::Deserialize;
use serde::Serialize;

use super::Machine;
use super::RegisterFileTrait;

#[derive(Debug, Copy, Clone, PartialEq, Hash, Eq, Deserialize, Serialize)]
pub enum AddrModeEnum {
    AccA,
    AccB,
    Immediate,
    Immediate16,
    Direct,
    Extended,
    Indexed,
    Inherent,
    Relative,
    Illegal,
}

pub fn u8_sign_extend(byte: u8) -> u16 {
    if (byte & 0x80) == 0x80 {
        byte as u16
    } else {
        (byte as u16) | 0xff00
    }
}

pub trait Bus {
    fn fetch_rel_addr<M: MemoryIO, R: RegisterFileTrait>(&mut self, _m: &mut Machine<M,R>) -> MemResult<u16> {
        panic!()
    }

    fn fetch_operand<M: MemoryIO,R: RegisterFileTrait>(&mut self, _m: &mut Machine<M,R>) -> MemResult<u8> {
        panic!()
    }

    fn fetch_operand_16<M: MemoryIO,R: RegisterFileTrait>(&mut self, _m: &mut Machine<M,R>) -> MemResult<u16> {
        panic!()
    }

    fn store_byte<M: MemoryIO,R: RegisterFileTrait>(
        &self,
        _m: &mut Machine<M,R>,
        _val: u8,
    ) -> MemResult<()>{
        panic!()
    }

    fn store_word<M: MemoryIO,R: RegisterFileTrait>(
        &self,
        _m: &mut Machine<M,R>,
        _val: u16,
    ) -> MemResult<()> {
        panic!()
    }
}

pub struct AccA;
pub struct AccB;
pub struct Immediate;
pub struct Direct;
pub struct Extended;
pub struct Indexed;
pub struct Inherent;
pub struct Relative;
pub struct Illegal;

impl Bus for AccA {
    fn fetch_operand<M: MemoryIO,R: RegisterFileTrait>(&mut self, _m: &mut Machine<M,R>) -> MemResult<u8> {
        Ok(_m.regs.a())
    }

    fn store_byte<M: MemoryIO,R: RegisterFileTrait>(
            &self,
            m: &mut Machine<M,R>,
            val: u8,
        ) -> MemResult<()> {
        m.regs.set_a(val);
        Ok(())
    }
}

impl Bus for AccB {
    fn fetch_operand<M: MemoryIO,R: RegisterFileTrait>(&mut self, _m: &mut Machine<M,R>) -> MemResult<u8> {
        Ok(_m.regs.b())
    }

    fn store_byte<M: MemoryIO,R: RegisterFileTrait>(
            &self,
            m: &mut Machine<M,R>,
            val: u8,
        ) -> MemResult<()> {
        m.regs.set_b(val);
        Ok(())
    }
}

impl Bus for Immediate {
    fn fetch_operand<M: MemoryIO,R: RegisterFileTrait>(&mut self, _m: &mut Machine<M,R>) -> MemResult<u8> {
        let pc = _m.regs.pc();
        let ret = _m.mem_mut().load_byte(pc as usize)?;
        _m.regs.set_pc(pc.wrapping_add(1));
        Ok(ret)
    }
}

impl Bus for Direct {}
impl Bus for Extended {
    fn fetch_operand<M: MemoryIO,R: RegisterFileTrait>(&mut self, _m: &mut Machine<M,R>) -> MemResult<u8> {
        let pc = _m.regs.pc();
        let ret = _m.mem_mut().load_word(pc as usize)?;
        _m.regs.set_pc(pc.wrapping_add(2));
        _m.mem_mut().load_byte(ret as usize)
    }

    fn store_byte<M: MemoryIO,R: RegisterFileTrait>(
            &self,
            _m: &mut Machine<M,R>,
            _val: u8,
        ) -> MemResult<()> {
        let pc = _m.regs.pc();
        let ret = _m.mem_mut().load_word(pc as usize)?;
        _m.regs.set_pc(pc.wrapping_add(2));
        _m.mem_mut().store_byte(ret as usize, _val)
    }
}

impl Bus for Indexed {}
impl Bus for Inherent {}
impl Bus for Relative {}
impl Bus for Illegal {}

