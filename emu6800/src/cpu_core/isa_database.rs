use super::{AddrModeEnum, Instruction, Isa, Mnemonic, OpcodeData};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct InstructionInfo<'a> {
    pub mnemonic: Mnemonic,
    pub addr_mode: AddrModeEnum,
    pub opcode_data: &'a OpcodeData,
    pub instruction: &'a Instruction,
}

impl<'a> InstructionInfo<'a> {
    pub fn get_mnemonic_text(&self) -> String {
        format!("{:?}", self.get_mnemonic()).to_lowercase()
    }
    pub fn get_mnemonic(&self) -> Mnemonic {
        self.mnemonic
    }
}

pub struct IsaDatabase {
    op_code_to_data: HashMap<usize, (Mnemonic, AddrModeEnum, OpcodeData)>,
    m_to_addr_modes: HashMap<Mnemonic, Instruction>,
}

impl IsaDatabase {
    pub fn new(_isa: &Isa) -> Self {
        let mut op_code_to_data = HashMap::new();

        for (m, a_modes) in _isa.instructions.iter() {
            for (amode, data) in a_modes.addr_modes.iter() {
                let v = (*m, *amode, data.clone());
                op_code_to_data.insert(data.opcode, v);
            }
        }

        Self {
            m_to_addr_modes: _isa.instructions.clone(),
            op_code_to_data,
        }
    }

    fn get_instruction_address_modes(&self, _m: Mnemonic) -> &Instruction {
        self.m_to_addr_modes.get(&_m).unwrap()
    }

    pub fn get_instruction_info_from_opcode(&self, _op_code: usize) -> Option<InstructionInfo> {
        self.op_code_to_data
            .get(&_op_code)
            .map(|(mnemonic, addr_mode, data)| {
                let instruction = self.m_to_addr_modes.get(mnemonic).unwrap();
                InstructionInfo {
                    mnemonic: *mnemonic,
                    addr_mode: *addr_mode,
                    opcode_data: data,
                    instruction,
                }
            })
    }
}
