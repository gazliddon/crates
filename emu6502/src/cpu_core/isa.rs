use super::{ Instructions, Mnemonic,  Instruction, OpcodeInfo};

pub struct Isa {
    pub instructions: Instructions,
    pub opcode_to_mnemonic: [Mnemonic; 256],
    pub opcode_to_instruction_info: [Option<OpcodeInfo>; 256],
}

impl Isa {
    pub fn get_instruction_info(&self, opcode: u8) -> Option<&OpcodeInfo> {
        self.opcode_to_instruction_info[opcode as usize].as_ref()
    }

    pub fn get_instruction(&self, m : Mnemonic) -> Option<&Instruction> {
        self.instructions.instructions.get(&m)
    }

    pub fn new(instructions: Instructions) -> Self {
        let mut opcodes = [Mnemonic::Illegal; 256];
        let mut opcode_to_instruction_info = [None; 256];

        for (mn, ins) in instructions.instructions.iter() {
            for (amode, opdata) in ins.addr_modes.iter() {
                let op_code = opdata.opcode as usize;
                let ins = instructions.get_opcode_info(*mn, *amode);
                opcode_to_instruction_info[op_code] = ins;
                opcodes[op_code] = *mn;
            }
        }

        Isa {
            opcode_to_instruction_info,
            instructions,
            opcode_to_mnemonic: opcodes,
        }
    }
}
