use emucore::mem::MemResult;
use emucore::mem::MemoryIO;
use serde::Deserialize;
use serde::Serialize;

use super::Machine;
use super::{RegisterFileTrait, StatusRegTrait};

use crate::cpu_core::AddrModeEnum;
use crate::cpu_core::calc_rel;

pub fn u8_sign_extend(byte: u8) -> u16 {
    if (byte & 0x80) == 0x80 {
        byte as u16
    } else {
        (byte as u16) | 0xff00
    }
}

pub trait Bus {
    fn fetch_effective_address<M: MemoryIO, R: RegisterFileTrait + StatusRegTrait>(
        _m: &mut Machine<M, R>,
    ) -> MemResult<u16> {
        panic!(
            "Not implemented fetch_effective_address for {}",
            Self::get_name()
        )
    }

    fn get_name() -> String;

    fn fetch_operand<M: MemoryIO, R: RegisterFileTrait + StatusRegTrait>(
        _m: &mut Machine<M, R>,
    ) -> MemResult<u8> {
        panic!("Not implemented fetch_operand for {}", Self::get_name())
    }

    fn fetch_operand_16<M: MemoryIO, R: RegisterFileTrait + StatusRegTrait>(
        _m: &mut Machine<M, R>,
    ) -> MemResult<u16> {
        panic!("Not implemented fetch_operand_16 for {}", Self::get_name())
    }

    fn store_byte<M: MemoryIO, R: RegisterFileTrait + StatusRegTrait>(
        _m: &mut Machine<M, R>,
        _val: u8,
    ) -> MemResult<()> {
        panic!("Not implemented store_byte for {}", Self::get_name())
    }

    fn store_word<M: MemoryIO, R: RegisterFileTrait + StatusRegTrait>(
        _m: &mut Machine<M, R>,
        _val: u16,
    ) -> MemResult<()> {
        panic!("Not implemented store_word for {}", Self::get_name())
    }

    fn read_mod_write<M: MemoryIO, R: RegisterFileTrait + StatusRegTrait, F>(
        _m: &mut Machine<M, R>,
        _f: F
    ) -> MemResult<(u8,u8)> 
        where
            F: Fn(u8) -> u8

    {
        panic!("Not implemented read_mod_write for {}", Self::get_name())
    }
}

pub struct AccA;
pub struct AccB;
pub struct Immediate;
pub type Immediate16 = Immediate;
pub struct Direct;
pub struct Extended;
pub struct Indexed;
pub struct Inherent;
pub struct Relative;
pub struct Illegal;

impl Bus for AccA {
    fn get_name() -> String {
        "AccA".to_owned()
    }

    fn read_mod_write<M: MemoryIO, R: RegisterFileTrait + StatusRegTrait, F>(
        m: &mut Machine<M, R>,
        f: F
    ) -> MemResult<(u8,u8)> 
        where
            F: Fn(u8) -> u8

    {
        let old = Self::fetch_operand(m)?;
        let new = f(old);
        Self::store_byte(m, new)?;
        Ok((old,new))
    }

    fn fetch_operand<M: MemoryIO, R: RegisterFileTrait + StatusRegTrait>(
        m: &mut Machine<M, R>,
    ) -> MemResult<u8> {
        Ok(m.regs.a())
    }

    fn store_byte<M: MemoryIO, R: RegisterFileTrait + StatusRegTrait>(
        m: &mut Machine<M, R>,
        val: u8,
    ) -> MemResult<()> {
        m.regs.set_a(val);
        Ok(())
    }
}

impl Bus for AccB {
    fn get_name() -> String {
        "AccB".to_owned()
    }

    fn read_mod_write<M: MemoryIO, R: RegisterFileTrait + StatusRegTrait, F>(
        m: &mut Machine<M, R>,
        f: F
    ) -> MemResult<(u8,u8)> 
        where
            F: Fn(u8) -> u8

    {
        let old = Self::fetch_operand(m)?;
        let new = f(old);
        Self::store_byte(m,new)?;
        Ok((old,new))
    }


    fn fetch_operand<M: MemoryIO, R: RegisterFileTrait + StatusRegTrait>(
        _m: &mut Machine<M, R>,
    ) -> MemResult<u8> {
        Ok(_m.regs.b())
    }

    fn store_byte<M: MemoryIO, R: RegisterFileTrait + StatusRegTrait>(
        m: &mut Machine<M, R>,
        val: u8,
    ) -> MemResult<()> {
        m.regs.set_b(val);
        Ok(())
    }
}

impl Bus for Immediate {
    fn get_name() -> String {
        "Immediate".to_owned()
    }
    fn fetch_operand_16<M: MemoryIO, R: RegisterFileTrait + StatusRegTrait>(
        _m: &mut Machine<M, R>,
    ) -> MemResult<u16> {
        let pc = _m.regs.pc();
        let ret = _m.mem_mut().load_word(pc as usize)?;
        _m.regs.set_pc(pc.wrapping_add(2));
        Ok(ret)
    }

    fn fetch_operand<M: MemoryIO, R: RegisterFileTrait + StatusRegTrait>(
        _m: &mut Machine<M, R>,
    ) -> MemResult<u8> {
        let pc = _m.regs.pc();
        let ret = _m.mem_mut().load_byte(pc as usize)?;
        _m.regs.set_pc(pc.wrapping_add(1));
        Ok(ret)
    }
}

impl Bus for Extended {
    fn get_name() -> String {
        "Extended".to_owned()
    }

    fn fetch_effective_address<M: MemoryIO, R: RegisterFileTrait + StatusRegTrait>(
        m: &mut Machine<M, R>,
    ) -> MemResult<u16> {
        Immediate::fetch_operand_16(m)
    }


    fn read_mod_write<M: MemoryIO, R: RegisterFileTrait + StatusRegTrait, F>(
        m: &mut Machine<M, R>,
        f: F
    ) -> MemResult<(u8,u8)> 
        where
            F: Fn(u8) -> u8

    {
        let addr = Self::fetch_effective_address(m)?;
        let mem = m.mem_mut();
        let old = mem.load_byte(addr as usize)?;
        let new = f(old);
        mem.store_byte(addr.into(), new)?;
        Ok((old,new))
    }

    fn fetch_operand_16<M: MemoryIO, R: RegisterFileTrait + StatusRegTrait>(
        _m: &mut Machine<M, R>,
    ) -> MemResult<u16> {
        let addr = Self::fetch_effective_address(_m)?;
        _m.mem_mut().load_word(addr as usize)
    }

    fn fetch_operand<M: MemoryIO, R: RegisterFileTrait + StatusRegTrait>(
        _m: &mut Machine<M, R>,
    ) -> MemResult<u8> {
        let addr = Self::fetch_effective_address(_m)?;
        _m.mem_mut().load_byte(addr as usize)
    }

    fn store_byte<M: MemoryIO, R: RegisterFileTrait + StatusRegTrait>(
            m: &mut Machine<M, R>,
            val: u8,
        ) -> MemResult<()> {
        let addr = Self::fetch_effective_address(m)?;
        m.mem_mut().store_byte(addr.into(), val)
    }
}

impl Bus for Direct {
    fn get_name() -> String {
        "Direct".to_owned()
    }

    fn fetch_effective_address<M: MemoryIO, R: RegisterFileTrait + StatusRegTrait>(
        m: &mut Machine<M, R>,
    ) -> MemResult<u16> {
        let offset = Immediate::fetch_operand(m)? as u16;
        Ok(offset as u16)
    }
    fn fetch_operand_16<M: MemoryIO, R: RegisterFileTrait + StatusRegTrait>(
        _m: &mut Machine<M, R>,
    ) -> MemResult<u16> {
        let addr = Self::fetch_effective_address(_m)?;
        _m.mem_mut().load_word(addr as usize)
    }

    fn fetch_operand<M: MemoryIO, R: RegisterFileTrait + StatusRegTrait>(
        _m: &mut Machine<M, R>,
    ) -> MemResult<u8> {
        let addr = Self::fetch_effective_address(_m)?;
        _m.mem_mut().load_byte(addr as usize)
    }
}

impl Bus for Indexed {
    fn get_name() -> String {
        "Indexed".to_owned()
    }

    fn fetch_effective_address<M: MemoryIO, R: RegisterFileTrait + StatusRegTrait>(
        m: &mut Machine<M, R>,
    ) -> MemResult<u16> {
        let offset = Immediate::fetch_operand(m)? as u16;
        let dst = m.regs.x().wrapping_add(offset);
        Ok(dst)
    }

    fn fetch_operand<M: MemoryIO, R: RegisterFileTrait + StatusRegTrait>(
            m: &mut Machine<M, R>,
        ) -> MemResult<u8> {
        let addr = Self::fetch_effective_address(m)?;
        m.mem_mut().load_byte(addr.into())
    }

    fn fetch_operand_16<M: MemoryIO, R: RegisterFileTrait + StatusRegTrait>(
            m: &mut Machine<M, R>,
        ) -> MemResult<u16> {
        let addr = Self::fetch_effective_address(m)?;
        m.mem_mut().load_word(addr.into())
    }


    fn store_byte<M: MemoryIO, R: RegisterFileTrait + StatusRegTrait>(
        m: &mut Machine<M, R>,
        v: u8,
    ) -> MemResult<()> {
        let addr = Self::fetch_effective_address(m)?;
        m.mem_mut().store_byte(addr.into(), v)
    }
}
impl Bus for Inherent {
    fn get_name() -> String {
        "Inherent".to_owned()
    }
}
impl Bus for Relative {
    fn get_name() -> String {
        "Relative".to_owned()
    }

    fn fetch_operand_16<M: MemoryIO, R: RegisterFileTrait + StatusRegTrait>(
            m: &mut Machine<M, R>,
        ) -> MemResult<u16> {
        Self::fetch_effective_address(m)
    }

    fn fetch_effective_address<M: MemoryIO, R: RegisterFileTrait + StatusRegTrait>(
            m: &mut Machine<M, R>,
        ) -> MemResult<u16> {
        let op = Immediate::fetch_operand(m)?;
        let pc = m.regs.pc();
        let dest = calc_rel(pc, op);
        Ok(dest)
    }
}
impl Bus for Illegal {
    fn get_name() -> String {
        "Illegal".to_owned()
    }
}
