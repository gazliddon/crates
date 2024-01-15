use serde::{Deserialize, Serialize};
use super::Mnemonic;

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

#[derive(Debug, Copy, Clone, PartialEq, Hash, Eq, Deserialize, Serialize)]
pub enum AddrModeEnum {
    AccA,
    AccB,
    Immediate,
    Immediate16,
    Direct,
    Extended,
    Indexed,
    Inherent,
    Relative,
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

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct Isa {
    pub instructions: HashMap<Mnemonic, Instruction>,
}
