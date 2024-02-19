use super::{Instruction, Instructions, Mnemonic, OpcodeInfo};

#[derive(Debug, Clone)]
pub struct Isa {
    pub instructions: Instructions,
    pub opcode_to_mnemonic: [Mnemonic; 256],
    pub opcode_to_instruction_info: [Option<OpcodeInfo>; 256],
}

impl Isa {
    pub fn get_opcode_info(&self, opcode: u8) -> Option<&OpcodeInfo> {
        self.opcode_to_instruction_info[opcode as usize].as_ref()
    }

    pub fn get_instruction_info(&self, m: Mnemonic) -> Option<&Instruction> {
        self.instructions.instructions.get(&m)
    }

    pub fn new(instructions: Instructions) -> Self {
        let mut opcodes = [Mnemonic::Illegal; 256];
        let mut opcode_to_instruction_info = [None; 256];

        for i in 0..=255 {
            let ins = instructions.get_opcode_info_from_opcode(i);
            opcode_to_instruction_info[i as usize] = ins;
            opcodes[i as usize] = ins.map(|i| i.menmonic).unwrap_or_default();
        }

        Isa {
            opcode_to_instruction_info,
            instructions,
            opcode_to_mnemonic: opcodes,
        }
    }
}
