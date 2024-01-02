use serde::Deserialize;
use std::collections::HashMap;

use super::AddrModeEnum;

#[derive(Clone,Deserialize)]
pub struct Instruction {
    opcode: usize,
    cycles: usize,
    size: usize,
    addr_mode : AddrModeEnum,
    text: String,
}

// Instruction default is a nop
impl Default for Instruction {
    fn default() -> Self {
        Self {
            opcode : 1,
            cycles: 2,
            size: 1,
            addr_mode: AddrModeEnum::Inherent,
            text: "nop".to_owned(),
        }
    }
}

#[derive(Clone,Deserialize, Default)]
pub struct DataBase {
    pub default: Instruction,
    pub instructions: Vec<Instruction>
}

impl DataBase {
}

pub struct InstructionInfo {
    text: String,
    instructions: HashMap<AddrModeEnum, Instruction>,
}

