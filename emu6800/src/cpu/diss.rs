use emucore::mem::MemoryIO;

use super::{InstructionInfo, IsaDatabase};

/// Returns a string disassmbly string and the next instruction address
pub fn diss<M: MemoryIO>(mem: &M, addr: usize, isa: &IsaDatabase) -> (usize, String) {
    let op_code = mem.inspect_byte(addr).unwrap();
    let ins = isa
        .get_instruction_info_from_opcode(op_code as usize)
        .unwrap();
    let mn = ins.get_mnemonic_text();

    let operand = diss_operand(mem, addr + 1, &ins);
    let diss = format!("{mn} {operand}");

    (addr + ins.opcode_data.size, diss)
}

/// Returns operand + next ins PC
pub fn diss_operand<M: MemoryIO>(mem: &M, addr: usize, ins: &InstructionInfo) -> String {
    use super::AddrModeEnum::*;
    let text = match ins.addr_mode {
        AccA => "A".to_owned(),
        AccB => "B".to_owned(),
        Immediate => {
            let b = mem.inspect_byte(addr).unwrap();
            format!("#0x{b:02x}")
        }

        Immediate16 => {
            let w = mem.inspect_word(addr).unwrap();
            format!("#0x{w:04x}")
        }
        Direct => {
            let b = mem.inspect_byte(addr).unwrap();
            format!("0x{b:02x}")
        },
        Extended => {
            let w = mem.inspect_word(addr).unwrap();
            format!("0x{w:04x}")
        }

        Indexed => {
            let b = mem.inspect_byte(addr).unwrap();
            format!("0x{b:02x},x")
        },

        Inherent => "".to_owned(),

        Relative => {
            let b = mem.inspect_byte(addr).unwrap();
            let bs = b as i8;
            format!("{bs:02x}")
        },

        Illegal => "????".to_owned(),
    };

    text
}
