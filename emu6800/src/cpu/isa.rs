use super::{AddrModeEnum, Flags, RegEnum};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
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
/// All of the information for all of the address modes
/// of this instruction
pub struct Instruction {
    #[serde(default)]
    flags_modified: Flags,
    addr_modes: HashMap<AddrModeEnum, OpcodeData>,
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
/// Data for an individual opcode
pub struct OpcodeData {
    #[serde(default)]
    pub regs_read: HashSet<RegEnum>,
    #[serde(default)]
    pub regs_written: HashSet<RegEnum>,
    pub opcode: usize,
    pub cycles: usize,
    pub size: usize,
}

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct Isa {
    instructions: HashMap<Mnemonic, Instruction>,
}

struct InstructionInfo<'a> {
    mnemonic: Mnemonic,
    addr_mode: AddrModeEnum,
    instruction: &'a OpcodeData,
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
