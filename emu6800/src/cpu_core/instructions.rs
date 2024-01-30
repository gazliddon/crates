use serde::{Deserialize, Serialize};
use super::Mnemonic;
use std::str::FromStr;

use std::collections::{ HashMap, HashSet };
#[derive(
    Copy, Debug, Clone, Hash, Ord, Eq, PartialEq, PartialOrd, Default, Serialize, Deserialize,
)]

pub enum RegEnum {
    #[default]
    A,
    B,
    X,
    PC,
    SP,
    SR,
}

bitflags::bitflags! {
    #[derive(Clone, Copy, Debug,Deserialize, Serialize, PartialEq, Default)]
    #[serde(transparent)]
    pub struct StatusReg : u8
        {
            const H  = 1 << 5;
            const I  = 1 << 4;
            const N  = 1 << 3;
            const Z  = 1 << 2;
            const V  = 1 << 1;
            const C = 1 << 0;
        }
}

#[derive(Debug, Copy, Clone, PartialEq, Hash, Eq, Deserialize, Serialize, Default)]
pub enum AddrModeEnum {
    Immediate8,
    Immediate16,
    Direct,
    Extended,
    Indexed,
    Inherent,
    Relative,
    #[default]
    Illegal,
}

/// All of the information for all of the address modes
/// of this instruction
#[derive(Default, Serialize,Deserialize,Debug, Clone)]
pub struct Instruction {
    #[serde(default)]
    pub flags_read: StatusReg,
    pub flags_written: StatusReg,
    pub addr_modes: HashMap<AddrModeEnum, OpcodeData>,
}

impl Instruction {
    pub fn get_opcode_data(&self, _amode: AddrModeEnum) -> Option<&OpcodeData> {
        self.addr_modes.get(&_amode)
    }

    pub fn supports(&self, amode: AddrModeEnum) -> bool {
        self.get_opcode_data(amode).is_some()
    }
}

#[derive(Default, Serialize, Deserialize, Debug, Clone, PartialEq)]
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

impl FromStr for RegEnum {
    type Err = ();
    fn from_str(txt: &str) -> Result<Self, Self::Err> {
        let x = txt.to_ascii_lowercase();

        match x.as_str() {
            "a" => Ok(RegEnum::A),
            "b" => Ok(RegEnum::B),
            "x" => Ok(RegEnum::X),
            "pc" => Ok(RegEnum::PC),
            "sp" => Ok(RegEnum::SP),
            "sr" => Ok(RegEnum::SR),
            _ => Err(()),
        }
    }
}

impl std::fmt::Display for RegEnum {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(_f, "{self:?}")
    }
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct Isa {
    pub instructions: HashMap<Mnemonic, Instruction>,
}
