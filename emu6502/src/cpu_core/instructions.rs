use super::{from_text, AddrModeEnum, Mnemonic, InstructionKind};
use serde::{de::Error, Deserialize, Deserializer};

use emucore::flagmods::FlagMods;
use std::collections::HashMap;

/// All of the information for all of the address modes
/// of this instruction
#[derive(Default, Deserialize, Debug, Clone)]
pub struct Instruction {
    #[serde(deserialize_with = "from_text")]
    pub flags: FlagMods,
    #[serde(default)]
    pub kind: InstructionKind,
    pub addr_modes: HashMap<AddrModeEnum, OpcodeData>,
}

/// Information about one particular instruction
#[derive(Default, Deserialize, Debug, Clone, Copy)]
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

/// All of the data about all of the instructions
#[derive(Default, Deserialize, Debug, Clone)]
pub struct Instructions {
    pub flag_order: String,
    pub instructions: HashMap<Mnemonic, Instruction>,
}

#[derive(Debug, Copy, Clone)]
pub struct OpcodeInfo {
    pub addr_mode : AddrModeEnum,
    pub opcode_data: OpcodeData,
    pub menmonic: Mnemonic,
}

impl Instructions {
    pub fn get_opcode_info(&self, m: Mnemonic, a: AddrModeEnum) -> Option<OpcodeInfo> {
        let i = self.instructions.get(&m)?;
        let od = i.addr_modes.get(&a)?;
        Some(OpcodeInfo {
            addr_mode: a,
            opcode_data: *od,
            menmonic: m,
        })

    }

    pub fn get_opcode_info_from_opcode(&self, opcode: u8) -> Option<OpcodeInfo> {
        for (m, i) in self.instructions.iter() {
            for (a, od) in i.addr_modes.iter() {
            if od.opcode == opcode {
                return Some(OpcodeInfo {
                    addr_mode: *a,
                    opcode_data: *od,
                    menmonic: *m,
                });
            }
            }
        }
        None
    }
}
