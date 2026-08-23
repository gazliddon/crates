use std::usize;

use emucore::mem::{MemErrorTypes, MemResult, MemoryIO};
use itertools::MergeJoinBy;

use crate::cpu_core::{calc_rel, InstructionInfo, IsaDatabase};

pub struct Disassmbly<'a> {
    pub text: String,
    pub mem_data: Vec<u8>,
    pub mem_string: String,
    pub ins: InstructionInfo<'a>,
    pub pc: usize,
    pub next_pc: usize,
}

impl<'a> std::fmt::Display for Disassmbly<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let cycles = self.ins.opcode_data.cycles;
        let pc = self.pc;
        write!(
            f,
            "{pc:04x} {:19} [ {cycles} ]    {}",
            self.mem_string, self.text
        )
    }
}

use thiserror::Error;

use super::ISA_DBASE;

#[derive(Error, Debug)]
pub enum DisError {
    #[error(transparent)]
    Mem(#[from] MemErrorTypes),
    #[error("Illegal instruction {0}")]
    IllegalInstruction(u8),
}

pub type DisResult<T> = Result<T, DisError>;

pub fn diss<'a, M: MemoryIO>(mem: &M, pc: usize) -> DisResult<Disassmbly<'a>> {
    let addr_u16 = (pc & 0xffff) as u16;

    let op_code = mem.inspect_byte(pc)?;
    let ins = ISA_DBASE
        .get_instruction_info_from_opcode(op_code as usize)
        .ok_or(DisError::IllegalInstruction(op_code))?;

    let mn = ins.get_mnemonic_text();
    let operand = diss_operand(mem, addr_u16.wrapping_add(1), &ins)?;
    let text = format!("{mn} {operand}");
    let next_pc = addr_u16.wrapping_add(ins.opcode_data.size as u16) as usize;

    let r = &(pc..next_pc);

    let mem_data = mem.get_mem(r);
    let mem_string = mem.get_mem_as_str(r, " ");

    let ret = Disassmbly {
        text,
        pc,
        next_pc,
        mem_data,
        mem_string,
        ins,
    };

    Ok(ret)
}

/// Returns operand + next ins PC
pub fn diss_operand<M: MemoryIO>(mem: &M, addr: u16, ins: &InstructionInfo) -> DisResult<String> {
    let addr_usize = addr as usize;

    use crate::cpu_core::AddrModeEnum::*;
    let text = match ins.addr_mode {
        Immediate8 => {
            let b = mem.inspect_byte(addr_usize)?;
            format!("#0x{b:02x}")
        }

        Immediate16 => {
            let w = mem.inspect_word(addr_usize)?;
            format!("#0x{w:04x}")
        }

        Direct => {
            let b = mem.inspect_byte(addr_usize)?;
            format!("<0x{b:02x}")
        }

        Extended => {
            let w = mem.inspect_word(addr_usize)?;
            format!("0x{w:04x}")
        }

        Indexed => {
            let b = mem.inspect_byte(addr_usize)?;
            format!("0x{b:02x},x")
        }

        Inherent => "".to_owned(),

        Relative => {
            let b = mem.inspect_byte(addr_usize)?;
            let x = calc_rel(addr.wrapping_add(1), b);
            format!("{x:04x}")
        }

        Illegal => "????".to_owned(),
    };
    Ok(text)
}

#[cfg(test)]
mod tests {
    use super::diss;
    use crate::cpu::ISA_DBASE;
    use emucore::byteorder::BigEndian;
    use emucore::mem::{MemBlock, MemoryIO};

    #[test]
    fn disassembles_sound_rom_entry() {
        let mut mem: MemBlock<BigEndian> = MemBlock::new("test", false, &(0..0x10000));
        mem.store_byte(0xf801, 0x0f).unwrap();

        let instruction = diss(&mem, 0xf801).unwrap();

        assert_eq!(instruction.text.trim(), "sei");
        assert_eq!(instruction.next_pc, 0xf802);
        assert_eq!(instruction.mem_data, vec![0x0f]);
    }

    #[test]
    fn disassembles_relative_branch_target() {
        let mut mem: MemBlock<BigEndian> = MemBlock::new("test", false, &(0..0x10000));
        // BRA -2 at $f828 branches back to itself.
        mem.store_byte(0xf828, 0x20).unwrap();
        mem.store_byte(0xf829, 0xfe).unwrap();

        let instruction = diss(&mem, 0xf828).unwrap();

        assert_eq!(instruction.text.trim(), "bra f828");
        assert_eq!(instruction.next_pc, 0xf82a);
    }

    #[test]
    fn opcode_database_contains_all_256_byte_values_that_are_defined() {
        assert!(ISA_DBASE.get_instruction_info_from_opcode(0x0f).is_some());
        assert!(ISA_DBASE.get_instruction_info_from_opcode(0xff).is_some());
    }
}
