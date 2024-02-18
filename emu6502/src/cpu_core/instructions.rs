use super::{  AddrModeEnum, from_text };
use serde::{ Deserialize, Deserializer, de::Error };

use emucore::flagmods::FlagMod;

use std::collections::HashMap;

/// All of the information for all of the address modes
/// of this instruction
#[derive(Default,  Deserialize, Debug, Clone)]
pub struct Instruction {
    #[serde(deserialize_with = "from_text")]
    pub flags: [FlagMod; 8],
    pub addr_modes: HashMap<AddrModeEnum, OpcodeData>,
}

#[derive(Default, Deserialize, Debug, Clone)]
pub struct OpcodeData {
    #[serde(deserialize_with = "from_hex")]
    pub opcode: u8,
    pub cycles: u8,
    pub size: u8,
}

fn from_hex<'de, D>(de: D) -> Result<u8, D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(de)?;
    // do better hex decoding than this
    u8::from_str_radix(s, 16).map_err(D::Error::custom)
}

#[derive(Default, Deserialize, Debug, Clone)]
pub struct Instructions {
    pub flag_order: String,
    pub instructions: HashMap<String, Instruction>
}

