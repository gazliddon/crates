use super::{AddrModeEnum, Instruction, Isa, Mnemonic, OpcodeData};
use std::collections::HashMap;

use strum::{ EnumIter,IntoEnumIterator };

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
    opcode_to_mnemonic: HashMap<String,Mnemonic>
}

impl IsaDatabase {
    pub fn new(_isa: &Isa) -> Self {
        let mut op_code_to_data = HashMap::new();
        let mut opcode_to_mnemonic = HashMap::new();

        for (m, a_modes) in _isa.instructions.iter() {
            for (amode, data) in a_modes.addr_modes.iter() {
                let v = (*m, *amode, data.clone());
                op_code_to_data.insert(data.opcode, v);
            }
        }

        for _m in Mnemonic::iter() {
            let text = format!("{_m:?}").to_lowercase();
            opcode_to_mnemonic.insert(text, _m);
        }

        Self {
            m_to_addr_modes: _isa.instructions.clone(),
            op_code_to_data,
            opcode_to_mnemonic
        }
    }

    fn get_instruction_address_modes(&self, _m: Mnemonic) -> &Instruction {
        self.m_to_addr_modes.get(&_m).unwrap()
    }

    pub fn get_opcode(&self, _name: &str) -> Option<&Instruction> {
        self.opcode_to_mnemonic.get(_name).map(|m| self.get_instruction_address_modes(*m))
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
