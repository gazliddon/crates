use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::AddrModeEnum;
use strum::EnumString;

#[derive(
    Clone, Copy, Deserialize, Serialize, Debug, EnumString, Hash, PartialEq, PartialOrd, Eq,
)]

pub enum Mnemonic {
    Des,
    Bmi,
    Bra,
    Bsr,
    Rti,
    Ins,
    Clc,
    Pul,
    Eor,
    And,
    Sbc,
    Cmp,
    Beq,
    Bcc,
    Clr,
    Jsr,
    Asl,
    Ldx,
    Tap,
    Tpa,
    Aba,
    Nop,
    Sei,
    Blt,
    Stx,
    Neg,
    Psh,
    Tba,
    Tsx,
    Rts,
    Sts,
    Sba,
    Clv,
    Lds,
    Dec,
    Add,
    Sev,
    Bgt,
    Adc,
    Sec,
    Bit,
    Lsr,
    Daa,
    Bvc,
    Rol,
    Bvs,
    Cba,
    Sta,
    Swi,
    Asr,
    Ror,
    Bhi,
    Ble,
    Bls,
    Txs,
    Jmp,
    Tab,
    Bge,
    Sub,
    Bpl,
    Inx,
    Inc,
    Tst,
    Ora,
    Com,
    Cpx,
    Dex,
    Bne,
    Bcs,
    Cli,
    Lda,
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct AddrModes {
    addr_modes: HashMap<AddrModeEnum, InstructionData>,
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct InstructionData {
    pub opcode: usize,
    pub cycles: usize,
    pub size: usize,
}

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct Isa {
    instructions: HashMap<Mnemonic, AddrModes>,
}

struct InstructionInfo<'a> {
    mnemonic: Mnemonic,
    addr_mode: AddrModeEnum,
    instruction: &'a InstructionData,
}

pub struct IsaDatabase {
    op_code_to_data: HashMap<usize, (Mnemonic, AddrModeEnum, InstructionData)>,
    m_to_addr_modes: HashMap<Mnemonic, AddrModes>,
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

    fn get_instruction_address_modes(&self, _m: Mnemonic) -> &AddrModes {
        self.m_to_addr_modes.get(&_m).unwrap()
    }

    fn get_instruction_info_from_opcode(&self, _op_code: usize) -> Option<InstructionInfo> {
        self.op_code_to_data
            .get(&_op_code)
            .map(|(mnemonic, addr_mode, instruction)| InstructionInfo {
                mnemonic: *mnemonic,
                addr_mode: *addr_mode,
                instruction,
            })
    }
}
