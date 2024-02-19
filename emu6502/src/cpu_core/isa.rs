use super::{ Instructions, Mnemonic, AddrModeEnum, Instruction};

pub struct Isa {
    pub instructions: Instructions,
    pub opcode_to_mnemonic: [Mnemonic; 256],
    pub opcode_to_instruction_info: [Option<InstructionInfo>; 256],
}

#[derive(Debug, Clone, Copy)]
pub struct InstructionInfo {
    pub opcode: u8,
    pub mnemonic: Mnemonic,
    pub addr_mode: AddrModeEnum,
    pub cycles: u8,
    pub size: u8,
}

impl Isa {
    pub fn get_instruction_info<'a>(&'a self, opcode: u8) -> Option<&'a InstructionInfo> {
        self.opcode_to_instruction_info[opcode as usize].as_ref()
    }

    pub fn get_instruction(&self, m : Mnemonic) -> Option<&Instruction> {
        self.instructions.instructions.get(&m)
    }

    pub fn new(ins: Instructions) -> Self {
        let mut opcodes = [Mnemonic::Illegal; 256];
        let mut opcode_to_instruction_info = [None; 256];

        for (mn, ins) in ins.instructions.iter() {
            for (amode, opdata) in ins.addr_modes.iter() {
                let op_code = opdata.opcode as usize;
                opcodes[op_code] = *mn;
                let ins = InstructionInfo {
                    opcode: opdata.opcode,
                    mnemonic: *mn,
                    addr_mode: *amode,
                    cycles: opdata.cycles,
                    size: opdata.size,
                };
                opcode_to_instruction_info[op_code] = Some(ins);
            }
        }

        Isa {
            opcode_to_instruction_info,
            instructions: ins,
            opcode_to_mnemonic: opcodes,
        }
    }
}
