use emucore::mem::MemResult;
use emucore::mem::MemoryIO;
use serde::Deserialize;
use serde::Serialize;

use super::Machine;
use super::RegisterFileTrait;

use crate::cpu_core::AddrModeEnum;

pub fn u8_sign_extend(byte: u8) -> u16 {
    if (byte & 0x80) == 0x80 {
        byte as u16
    } else {
        (byte as u16) | 0xff00
    }
}

pub trait Bus {
    fn fetch_effective_address<M: MemoryIO, R: RegisterFileTrait>(&mut self, _m: &mut Machine<M,R>) -> MemResult<u16> { 
        panic!("Not implemented fetch_effective_address for {}", Self::get_name())
    }

    fn get_name() -> String;

    fn fetch_rel_addr<M: MemoryIO, R: RegisterFileTrait>(&mut self, _m: &mut Machine<M,R>) -> MemResult<u16> {
        panic!("Not implemented fetch_rel_add for {}", Self::get_name())
    }

    fn fetch_operand<M: MemoryIO,R: RegisterFileTrait>(&mut self, _m: &mut Machine<M,R>) -> MemResult<u8> {
        panic!("Not implemented fetch_operand for {}", Self::get_name())
    }

    fn fetch_operand_16<M: MemoryIO,R: RegisterFileTrait>(&mut self, _m: &mut Machine<M,R>) -> MemResult<u16> {
        panic!("Not implemented fetch_operand_16 for {}", Self::get_name())
    }

    fn store_byte<M: MemoryIO,R: RegisterFileTrait>(
        &self,
        _m: &mut Machine<M,R>,
        _val: u8,
    ) -> MemResult<()>{
        panic!("Not implemented store_byte for {}", Self::get_name())
    }

    fn store_word<M: MemoryIO,R: RegisterFileTrait>(
        &self,
        _m: &mut Machine<M,R>,
        _val: u16,
    ) -> MemResult<()> {
        panic!("Not implemented store_word for {}", Self::get_name())
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
    fn get_name() -> String {"AccA".to_owned()}

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
    fn get_name() -> String {"AccB".to_owned()}
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
    fn get_name() -> String {"Immediate".to_owned()}
    fn fetch_operand_16<M: MemoryIO,R: RegisterFileTrait>(&mut self, _m: &mut Machine<M,R>) -> MemResult<u16> {
        let pc = _m.regs.pc();
        let ret = _m.mem_mut().load_word(pc as usize)?;
        _m.regs.set_pc(pc.wrapping_add(2));
        _m.mem_mut().load_word(ret as usize)
    }

    fn fetch_operand<M: MemoryIO,R: RegisterFileTrait>(&mut self, _m: &mut Machine<M,R>) -> MemResult<u8> {
        let pc = _m.regs.pc();
        let ret = _m.mem_mut().load_byte(pc as usize)?;
        _m.regs.set_pc(pc.wrapping_add(1));
        Ok(ret)
    }
}


impl Bus for Extended {
    fn get_name() -> String {"Extended".to_owned()}

    fn fetch_effective_address<M: MemoryIO, R: RegisterFileTrait>(&mut self, m: &mut Machine<M,R>) -> MemResult<u16> {
        let operand = self.fetch_operand_16(m)?;
        m.mem_mut().load_word(operand as usize)
    }

    fn fetch_operand_16<M: MemoryIO,R: RegisterFileTrait>(&mut self, _m: &mut Machine<M,R>) -> MemResult<u16> {
        let pc = _m.regs.pc();
        let ret = _m.mem_mut().load_word(pc as usize)?;
        _m.regs.set_pc(pc.wrapping_add(2));
        _m.mem_mut().load_word(ret as usize)
    }

    fn fetch_operand<M: MemoryIO,R: RegisterFileTrait>(&mut self, _m: &mut Machine<M,R>) -> MemResult<u8> {
        let pc = _m.regs.pc();
        let ret = _m.mem_mut().load_word(pc as usize)?;
        _m.regs.set_pc(pc.wrapping_add(1));
        _m.mem_mut().load_byte(ret as usize)
    }

}

impl Bus for Direct {
    fn get_name() -> String {"Direct".to_owned()}
}

impl Bus for Indexed {
    fn get_name() -> String {"Indexed".to_owned()}

}
impl Bus for Inherent {
    fn get_name() -> String {"Inherent".to_owned()}

}
impl Bus for Relative {
    fn get_name() -> String {"Relative".to_owned()}
    
}
impl Bus for Illegal {
    fn get_name() -> String {"Illegal".to_owned()}

}

