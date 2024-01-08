use std::usize;

use emucore::mem::{MemResult, MemoryIO};

use super::{InstructionInfo, IsaDatabase};

/// Returns a string disassmbly string and the next instruction address
pub fn diss<M: MemoryIO>(mem: &M, addr: usize, isa: &IsaDatabase) -> MemResult<(usize, String)> {
    let addr = ( addr & 0xffff ) as u16;
    let addr_u = addr as usize;

    let op_code = mem.inspect_byte(addr_u).unwrap();
    let ins = isa
        .get_instruction_info_from_opcode(op_code as usize)
        .unwrap();
    let mn = ins.get_mnemonic_text();
    let operand = diss_operand(mem, addr.wrapping_add(1), &ins)?;
    let diss = format!("{mn} {operand}");
    let next = addr.wrapping_add(ins.opcode_data.size as u16);
    Ok((next as usize, diss))
}

/// Returns operand + next ins PC
pub fn diss_operand<M: MemoryIO>(mem: &M, addr: u16, ins: &InstructionInfo) -> MemResult<String> {
    let addr_u = addr as usize;

    use super::AddrModeEnum::*;
    let text = match ins.addr_mode {
        AccA => "A".to_owned(),
        AccB => "B".to_owned(),
        Immediate => {
            let b = mem.inspect_byte(addr_u)?;
            format!("#0x{b:02x}")
        }

        Immediate16 => {
            let w = mem.inspect_word(addr_u)?;
            format!("#0x{w:04x}")
        }
        Direct => {
            let b = mem.inspect_byte(addr_u)?;
            format!("0x{b:02x}")
        }
        Extended => {
            let w = mem.inspect_word(addr_u)?;
            format!("0x{w:04x}")
        }

        Indexed => {
            let b = mem.inspect_byte(addr_u)?;
            format!("0x{b:02x},x")
        }

        Inherent => "".to_owned(),

        Relative => {
            let b = mem.inspect_byte(addr_u)?;
            let bs = b as i8;
            format!("{bs:02x}")
        }

        Illegal => "????".to_owned(),
    };
    Ok(text)
}
