use super::{mem, CpuErr, IndexModes, IndexedFlags, InstructionDecoder, Regs};
use mem::MemoryIO;

use crate::isa::AddrModeEnum;

use log::warn;

pub trait AddressLines {
    // fn diss<M: MemoryIO>(_mem: &M, _ins: &mut InstructionDecoder) -> String {
    //     panic!("NOT IMP {:?}", Self::name())
    // }

    fn get_addr_mode() -> AddrModeEnum;

    fn name() -> String {
        Self::get_addr_mode().to_string()
    }

    fn diss(_mem: &dyn MemoryIO, _ins: &mut InstructionDecoder) -> String;

    fn ea(
        _mem: &mut dyn MemoryIO,
        _regs: &mut Regs,
        _ins: &mut InstructionDecoder,
    ) -> Result<u16, CpuErr> {
        warn!("EA for {}", Self::name());
        Err(CpuErr::IllegalAddressingMode)
    }

    fn store_byte(
        _mem: &mut dyn MemoryIO,
        _regs: &mut Regs,
        _ins: &mut InstructionDecoder,
        _val: u8,
    ) -> Result<u16, CpuErr> {
        Err(CpuErr::IllegalAddressingMode)
    }

    fn store_word(
        _mem: &mut dyn MemoryIO,
        _regs: &mut Regs,
        _ins: &mut InstructionDecoder,
        _val: u16,
    ) -> Result<u16, CpuErr> {
        Err(CpuErr::IllegalAddressingMode)
    }

    fn fetch_byte(
        _mem: &mut dyn MemoryIO,
        _regs: &mut Regs,
        _ins: &mut InstructionDecoder,
    ) -> Result<u8, CpuErr> {
        Err(CpuErr::IllegalAddressingMode)
    }

    fn fetch_word(
        _mem: &mut dyn MemoryIO,
        _regs: &mut Regs,
        _ins: &mut InstructionDecoder,
    ) -> Result<u16, CpuErr> {
        Err(CpuErr::IllegalAddressingMode)
    }

    fn fetch_byte_as_i16(
        mem: &mut dyn MemoryIO,
        regs: &mut Regs,
        ins: &mut InstructionDecoder,
    ) -> Result<i16, CpuErr> {
        let byte = Self::fetch_byte(mem, regs, ins)? as i8;
        Ok(i16::from(byte))
    }
}

////////////////////////////////////////////////////////////////////////////////
pub struct Direct {}

impl AddressLines for Direct {
    fn get_addr_mode() -> AddrModeEnum {
        AddrModeEnum::Direct
    }

    fn ea(
        mem: &mut dyn MemoryIO,
        regs: &mut Regs,
        ins: &mut InstructionDecoder,
    ) -> Result<u16, CpuErr> {
        let index = u16::from(ins.fetch_byte(mem));
        Ok(regs.get_dp_ptr().wrapping_add(index))
    }

    fn fetch_byte(
        mem: &mut dyn MemoryIO,
        regs: &mut Regs,
        ins: &mut InstructionDecoder,
    ) -> Result<u8, CpuErr> {
        let ea = Self::ea(mem, regs, ins)?;
        let b = mem.load_byte(ea.into())?;
        Ok(b)
    }

    fn fetch_word(
        mem: &mut dyn MemoryIO,
        regs: &mut Regs,
        ins: &mut InstructionDecoder,
    ) -> Result<u16, CpuErr> {
        let ea = Self::ea(mem, regs, ins)?;
        let w = mem.load_word(ea.into())?;
        Ok(w)
    }

    fn store_byte(
        mem: &mut dyn MemoryIO,
        regs: &mut Regs,
        ins: &mut InstructionDecoder,
        val: u8,
    ) -> Result<u16, CpuErr> {
        let ea = Self::ea(mem, regs, ins)?;
        mem.store_byte(ea.into(), val)?;
        Ok(ea)
    }

    fn store_word(
        mem: &mut dyn MemoryIO,
        regs: &mut Regs,
        ins: &mut InstructionDecoder,
        val: u16,
    ) -> Result<u16, CpuErr> {
        let ea = Self::ea(mem, regs, ins)?;
        mem.store_word(ea.into(), val)?;
        Ok(ea)
    }

    fn diss(mem: &dyn MemoryIO, ins: &mut InstructionDecoder) -> String {
        let val = ins.fetch_inspecte_byte(mem).unwrap();
        format!("<${val:02x}")
    }
}

////////////////////////////////////////////////////////////////////////////////
pub struct Extended {}

impl AddressLines for Extended {
    fn get_addr_mode() -> AddrModeEnum {
        AddrModeEnum::Extended
    }
    fn ea(
        mem: &mut dyn MemoryIO,
        _regs: &mut Regs,
        ins: &mut InstructionDecoder,
    ) -> Result<u16, CpuErr> {
        ins.fetch_word(mem)
    }

    fn fetch_byte(
        mem: &mut dyn MemoryIO,
        regs: &mut Regs,
        ins: &mut InstructionDecoder,
    ) -> Result<u8, CpuErr> {
        let addr = Self::ea(mem, regs, ins)?;
        let b = mem.load_byte(addr.into())?;
        Ok(b)
    }

    fn fetch_word(
        mem: &mut dyn MemoryIO,
        regs: &mut Regs,
        ins: &mut InstructionDecoder,
    ) -> Result<u16, CpuErr> {
        let addr = Self::ea(mem, regs, ins)?;
        let b = mem.load_word(addr.into())?;
        Ok(b)
    }

    fn store_byte(
        mem: &mut dyn MemoryIO,
        regs: &mut Regs,
        ins: &mut InstructionDecoder,
        val: u8,
    ) -> Result<u16, CpuErr> {
        let addr = Self::ea(mem, regs, ins)?;
        mem.store_byte(addr.into(), val)?;
        Ok(addr)
    }

    fn store_word(
        mem: &mut dyn MemoryIO,
        regs: &mut Regs,
        ins: &mut InstructionDecoder,
        val: u16,
    ) -> Result<u16, CpuErr> {
        let addr = Self::ea(mem, regs, ins)?;
        mem.store_word(addr.into(), val)?;
        Ok(addr)
    }

    fn diss(mem: &dyn MemoryIO, ins: &mut InstructionDecoder) -> String {
        let val = ins.fetch_inspect_word(mem).unwrap();
        format!(">${val:02x}")
    }
}

////////////////////////////////////////////////////////////////////////////////

pub struct Immediate8 {}

impl AddressLines for Immediate8 {
    fn get_addr_mode() -> AddrModeEnum {
        AddrModeEnum::Immediate8
    }

    fn fetch_byte(
        mem: &mut dyn MemoryIO,
        _regs: &mut Regs,
        ins: &mut InstructionDecoder,
    ) -> Result<u8, CpuErr> {
        Ok(ins.fetch_byte(mem))
    }

    fn diss(mem: &dyn MemoryIO, ins: &mut InstructionDecoder) -> String {
        let val = ins.fetch_inspecte_byte(mem).unwrap();
        format!("#${val:02x}")
    }
}

pub struct Immediate16 {}

impl AddressLines for Immediate16 {
    fn get_addr_mode() -> AddrModeEnum {
        AddrModeEnum::Immediate16
    }

    fn fetch_word(
        mem: &mut dyn MemoryIO,
        _regs: &mut Regs,
        ins: &mut InstructionDecoder,
    ) -> Result<u16, CpuErr> {
        ins.fetch_word(mem)
    }

    fn diss(mem: &dyn MemoryIO, ins: &mut InstructionDecoder) -> String {
        let val = ins.fetch_inspect_word(mem).unwrap();
        format!("#${val:04x}")
    }
}

////////////////////////////////////////////////////////////////////////////////
pub struct Inherent {}

impl AddressLines for Inherent {
    fn get_addr_mode() -> AddrModeEnum {
        AddrModeEnum::Inherent
    }

    fn fetch_byte(
        mem: &mut dyn MemoryIO,
        _regs: &mut Regs,
        ins: &mut InstructionDecoder,
    ) -> Result<u8, CpuErr> {
        Ok(ins.fetch_byte(mem))
    }

    fn diss(_mem: &dyn MemoryIO, _ins: &mut InstructionDecoder) -> String {
        "".to_string()
    }
}

pub struct RegisterSet {}

impl AddressLines for RegisterSet {
    fn get_addr_mode() -> AddrModeEnum {
        AddrModeEnum::RegisterSet
    }

    fn fetch_byte(
        mem: &mut dyn MemoryIO,
        _regs: &mut Regs,
        ins: &mut InstructionDecoder,
    ) -> Result<u8, CpuErr> {
        Ok(ins.fetch_byte(mem))
    }

    fn diss(_mem: &dyn MemoryIO, _ins: &mut InstructionDecoder) -> String {
        "".to_string()
    }
}

pub struct RegisterPair {}

impl AddressLines for RegisterPair {
    fn get_addr_mode() -> AddrModeEnum {
        AddrModeEnum::RegisterPair
    }

    fn fetch_byte(
        mem: &mut dyn MemoryIO,
        _regs: &mut Regs,
        ins: &mut InstructionDecoder,
    ) -> Result<u8, CpuErr> {
        Ok(ins.fetch_byte(mem))
    }

    fn diss(_mem: &dyn MemoryIO, _ins: &mut InstructionDecoder) -> String {
        "".to_string()
    }
}

////////////////////////////////////////////////////////////////////////////////
pub struct Indexed {}

impl Indexed {
    fn get_cycles(itype: &IndexModes) -> u32 {
        use IndexModes::*;

        match itype {
            RPlus(_) => 0,
            RPlusPlus(_) => 0,
            RSub(_) => 0,
            RSubSub(_) => 0,
            RZero(_) => 0,
            RAddB(_) => 0,
            RAddA(_) => 0,
            RAddi8(_) => 0,
            RAddi16(_) => 0,
            RAddD(_) => 0,
            PCAddi8 => 0,
            PCAddi16 => 0,
            Illegal => 0,
            Ea => 0,
            ROff(_, _) => 0,
        }
    }

    fn get_index_mode(
        mem: &mut dyn MemoryIO,
        regs: &mut Regs,
        ins: &mut InstructionDecoder,
    ) -> Result<(u16, IndexedFlags), CpuErr> {
        let index_mode_id = ins.fetch_byte(mem);

        let index_mode = IndexedFlags::new(index_mode_id);

        let itype = index_mode.get_index_type();

        match itype {
            IndexModes::RPlus(r) => {
                // format!("{:?}+",r)
                let addr = regs.get(&r);
                regs.inc(&r);
                Ok((addr, index_mode))
            }

            IndexModes::RPlusPlus(r) => {
                // format!("{:?}++",r)
                let addr = regs.get(&r);
                regs.incinc(&r);
                Ok((addr, index_mode))
            }

            IndexModes::RSub(r) => {
                // format!("{:?}-",r)
                Ok((regs.dec(&r), index_mode))
            }

            IndexModes::RSubSub(r) => {
                // format!("{:?}--",r)
                Ok((regs.decdec(&r), index_mode))
            }

            IndexModes::RZero(r) => {
                // format!("{:?}",r)
                Ok((regs.get(&r), index_mode))
            }

            IndexModes::RAddB(r) => {
                // format!("B,{:?}", r)
                let add_r = u16::from(regs.b);
                Ok((regs.get(&r).wrapping_add(add_r), index_mode))
            }

            IndexModes::RAddA(r) => {
                // format!("A,{:?}", r)
                let add_r = u16::from(regs.a);
                Ok((regs.get(&r).wrapping_add(add_r), index_mode))
            }

            IndexModes::RAddi8(r) => {
                // format!("{},{:?}",self.fetch_byte_as_i16(mem) as i8, r)
                let v = ins.fetch_byte_as_i16(mem) as u16;
                Ok((regs.get(&r).wrapping_add(v), index_mode))
            }

            IndexModes::RAddi16(r) => {
                // format!("{},{:?}",diss.fetch_word(mem) as i16, r)
                let v = ins.fetch_word(mem)?;
                Ok((regs.get(&r).wrapping_add(v), index_mode))
            }

            IndexModes::RAddD(r) => {
                // format!("D,{:?}", r)
                let add_r = regs.get_d();
                Ok((regs.get(&r).wrapping_add(add_r), index_mode))
            }

            IndexModes::PCAddi8 => {
                // format!("PC,{:?}",diss.fetch_byte(mem) as i8)
                let offset = ins.fetch_byte_as_i16(mem) as u16;
                Ok((regs.pc.wrapping_add(offset), index_mode))
            }

            IndexModes::PCAddi16 => {
                // format!("PC,{:?}",diss.fetch_word(mem) as i16)
                let offset = ins.fetch_word(mem)?;
                Ok((regs.pc.wrapping_add(offset), index_mode))
            }

            IndexModes::Illegal => Err(CpuErr::IllegalAddressingMode),

            IndexModes::Ea => {
                let idx = ins.fetch_word(mem)?;
                // format!("0x{:04X}", diss.fetch_word(mem))
                Ok((idx, index_mode))
            }

            IndexModes::ROff(r, offset) => {
                // format!("{}, {:?}", offset, r)
                Ok((regs.get(&r).wrapping_add(offset), index_mode))
            }
        }
    }
}

impl AddressLines for Indexed {
    fn get_addr_mode() -> AddrModeEnum {
        AddrModeEnum::Indexed
    }

    fn diss(mem: &dyn MemoryIO, ins: &mut InstructionDecoder) -> String {
        // FIXIT
        // Change type sign return to Result(String, CpuError)
        let index_mode_id = ins.fetch_inspecte_byte(mem).unwrap();

        let index_mode = IndexedFlags::new(index_mode_id);
        let itype = index_mode.get_index_type();

        match itype {
            IndexModes::RPlus(r) => {
                format!("{r:?}+")
            }
            IndexModes::RPlusPlus(r) => {
                format!("{r:?}++")
            }
            IndexModes::RSub(r) => {
                format!("{r:?}-")
            }
            IndexModes::RSubSub(r) => {
                format!("{r:?}--")
            }
            IndexModes::RZero(r) => {
                format!("{r:?}")
            }
            IndexModes::RAddB(r) => {
                format!("B,{r:?}")
            }
            IndexModes::RAddA(r) => {
                format!("A,{r:?}")
            }
            IndexModes::RAddi8(r) => {
                format!("{},{r:?}", ins.fetch_inspecte_byte(mem).unwrap() as i8)
            }
            IndexModes::RAddi16(r) => {
                format!("{},{r:?}", ins.fetch_inspecte_byte(mem).unwrap() as i8)
            }
            IndexModes::RAddD(r) => {
                format!("D,{r:?}")
            }
            IndexModes::PCAddi8 => {
                format!("PC,{:?}", ins.fetch_inspecte_byte(mem).unwrap() as i8)
            }
            IndexModes::PCAddi16 => {
                format!("PC,{:?}", ins.fetch_inspecte_byte(mem).unwrap())
            }
            IndexModes::Illegal => format!("ILLEGAL INDEX MODE {itype:?}"),
            IndexModes::Ea => {
                format!("0x{:04X}", ins.fetch_inspect_word(mem).unwrap())
            }
            IndexModes::ROff(r, offset) => {
                format!("{offset}, {r:?}")
            }
        }
    }

    fn ea(
        mem: &mut dyn MemoryIO,
        regs: &mut Regs,
        ins: &mut InstructionDecoder,
    ) -> Result<u16, CpuErr> {
        let (ea, index_mode) = Indexed::get_index_mode(mem, regs, ins)?;

        let ea = if index_mode.is_indirect() {
            mem.load_word(ea.into())?
        } else {
            ea
        };

        Ok(ea)
    }

    fn fetch_byte(
        mem: &mut dyn MemoryIO,
        regs: &mut Regs,
        ins: &mut InstructionDecoder,
    ) -> Result<u8, CpuErr> {
        let ea = Self::ea(mem, regs, ins)?;
        let b = mem.load_byte(ea.into())?;
        Ok(b)
    }

    fn fetch_word(
        mem: &mut dyn MemoryIO,
        regs: &mut Regs,
        ins: &mut InstructionDecoder,
    ) -> Result<u16, CpuErr> {
        let ea = Self::ea(mem, regs, ins)?;
        let w = mem.load_word(ea.into())?;
        Ok(w)
    }

    fn store_byte(
        mem: &mut dyn MemoryIO,
        regs: &mut Regs,
        ins: &mut InstructionDecoder,
        val: u8,
    ) -> Result<u16, CpuErr> {
        let ea = Self::ea(mem, regs, ins)?;
        mem.store_byte(ea.into(), val)?;
        Ok(ea)
    }

    fn store_word(
        mem: &mut dyn MemoryIO,
        regs: &mut Regs,
        ins: &mut InstructionDecoder,
        val: u16,
    ) -> Result<u16, CpuErr> {
        let ea = Self::ea(mem, regs, ins)?;
        mem.store_word(ea.into(), val)?;
        Ok(ea)
    }
}

////////////////////////////////////////////////////////////////////////////////
pub struct Relative {}

impl AddressLines for Relative {
    fn get_addr_mode() -> AddrModeEnum {
        AddrModeEnum::Relative
    }

    fn fetch_byte(
        mem: &mut dyn MemoryIO,
        _regs: &mut Regs,
        ins: &mut InstructionDecoder,
    ) -> Result<u8, CpuErr> {
        Ok(ins.fetch_byte(mem))
    }

    fn diss(mem: &dyn MemoryIO, ins: &mut InstructionDecoder) -> String {
        let val = ins.fetch_inspecte_byte(mem).unwrap() as i8;
        format!(" ${:04x} + {}", ins.addr, val)
    }
}

pub struct Relative16 {}

impl AddressLines for Relative16 {
    fn get_addr_mode() -> AddrModeEnum {
        AddrModeEnum::Relative16
    }

    fn fetch_byte(
        mem: &mut dyn MemoryIO,
        _regs: &mut Regs,
        ins: &mut InstructionDecoder,
    ) -> Result<u8, CpuErr> {
        Ok(ins.fetch_byte(mem))
    }

    fn fetch_word(
        mem: &mut dyn MemoryIO,
        _regs: &mut Regs,
        ins: &mut InstructionDecoder,
    ) -> Result<u16, CpuErr> {
        ins.fetch_word(mem)
    }

    fn diss(mem: &dyn MemoryIO, ins: &mut InstructionDecoder) -> String {
        let val = ins.fetch_inspect_word(mem).unwrap() as i16;
        format!(" ${:04x} + {}", ins.addr, val)
    }
}
