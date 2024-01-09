use std::usize;

use emucore::mem::{MemResult, MemoryIO};

use super::{InstructionInfo, IsaDatabase};

pub struct Disassmbly<'a> {
    pub text: String,
    pub mem_data: Vec<u8>,
    pub mem_string: String,
    pub ins: InstructionInfo<'a>,
    pub pc: usize,
    pub next_pc: usize,
}

pub fn diss<'a, M: MemoryIO>(
    mem: &M,
    pc: usize,
    isa: &'a IsaDatabase,
) -> MemResult<Disassmbly<'a>> {
    let addr_u16 = (pc & 0xffff) as u16;

    let op_code = mem.inspect_byte(pc)?;
    let ins = isa
        .get_instruction_info_from_opcode(op_code as usize)
        .unwrap();

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
pub fn diss_operand<M: MemoryIO>(mem: &M, addr: u16, ins: &InstructionInfo) -> MemResult<String> {
    let addr_usize = addr as usize;

    use super::AddrModeEnum::*;
    let text = match ins.addr_mode {
        AccA => "A".to_owned(),
        AccB => "B".to_owned(),
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
            let bs = b as i8;
            format!("{bs:02x}")
        }

        Illegal => "????".to_owned(),
    };
    Ok(text)
}
