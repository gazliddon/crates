use super::{AddrModeEnum, RegEnum, StatusReg};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use strum::EnumString;

#[derive(
    Clone, Copy, Deserialize, Serialize, Debug, EnumString, Hash, PartialEq, PartialOrd, Eq, Default
)]

pub enum Mnemonic {
    Aba,
    AdcA,
    AdcB,
    AddA,
    AddB,
    AndA,
    AndB,
    Asl,
    Asr,
    Bcc,
    Bcs,
    Beq,
    Bge,
    Bgt,
    Bhi,
    BitA,
    BitB,
    Ble,
    Bls,
    Blt,
    Bmi,
    Bne,
    Bpl,
    Bra,
    Bsr,
    Bvc,
    Bvs,
    Cba,
    Clc,
    Cli,
    Clr,
    Clv,
    CmpA,
    CmpB,
    Com,
    Cpx,
    Daa,
    Dec,
    Des,
    Dex,
    EorA,
    EorB,
    Inc,
    Ins,
    Inx,
    Jmp,
    Jsr,
    LdaA,
    LdaB,
    Lds,
    Ldx,
    Lsr,
    Neg,
    Nop,
    OraA,
    OraB,
    Psh,
    Pul,
    Rol,
    Ror,
    Rti,
    Rts,
    Sba,
    SbcA,
    SbcB,
    Sec,
    Sei,
    Sev,
    StaA,
    StaB,
    Sts,
    Stx,
    SubA,
    SubB,
    Swi,
    Tab,
    Tap,
    Tba,
    Tpa,
    Tst,
    Tsx,
    Txs,
    Wai,

    #[default]
    Illegal,
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
/// All of the information for all of the address modes
/// of this instruction
pub struct Instruction {
    #[serde(default)]
    flags_read: StatusReg,
    flags_written: StatusReg,
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


#[derive( Debug, Clone)]
pub struct InstructionInfo<'a> {
    pub mnemonic: Mnemonic,
    pub addr_mode: AddrModeEnum,
    pub opcode_data: &'a OpcodeData,
    pub instruction: &'a Instruction,
}

impl<'a> InstructionInfo<'a> {
    pub fn get_mnemonic_text(&self) -> String {
        format!("{:?}", self.get_mnemonic())
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
            } })
    }
}
