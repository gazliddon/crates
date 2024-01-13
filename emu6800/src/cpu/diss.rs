use std::usize;

use emucore::mem::{MemResult, MemoryIO, MemErrorTypes};
use itertools::MergeJoinBy;

use crate::cpu_core::{InstructionInfo, IsaDatabase};

pub struct Disassmbly<'a> {
    pub text: String,
    pub mem_data: Vec<u8>,
    pub mem_string: String,
    pub ins: InstructionInfo<'a>,
    pub pc: usize,
    pub next_pc: usize,
}

use thiserror::Error;

#[derive(Error,Debug)]
pub enum DisError {
    #[error(transparent)]
    Mem(#[from] MemErrorTypes),
    #[error("Illegal instruction {0}")]
    IllegalInstruction(u8),
}

pub type DisResult<T> = Result<T,DisError>;

pub fn diss<'a, M: MemoryIO>(
    mem: &M,
    pc: usize,
    isa: &'a IsaDatabase,
) -> DisResult<Disassmbly<'a>> {
    let addr_u16 = (pc & 0xffff) as u16;

    let op_code = mem.inspect_byte(pc)?;
    let ins = isa
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

fn calc_rel(addr: u16, rel_byte: u8) -> u16 {
    let addr = ( addr as isize ) + ((rel_byte as i8) as isize);
    (addr & 0xffff) as u16
}

/// Returns operand + next ins PC
pub fn diss_operand<M: MemoryIO>(mem: &M, addr: u16, ins: &InstructionInfo) -> DisResult<String> {
    let addr_usize = addr as usize;

    use crate::cpu_core::AddrModeEnum::*;
    let text = match ins.addr_mode {
        AccA => "a".to_owned(),

        AccB => "b".to_owned(),

        Immediate => {
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
