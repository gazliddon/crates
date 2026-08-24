use super::Mnemonic;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use std::collections::{HashMap, HashSet};

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

use strum::Display;

#[derive(Debug, Copy, Clone, PartialEq, Hash, Eq, Deserialize, Serialize, Default, Display)]
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
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct Instruction {
    #[serde(default)]
    pub flags_read: StatusReg,
    pub flags_written: StatusReg,
    /// 6-char HINZVC effects ('-' unaffected, '*' set by result, '0'
    /// cleared, '1' set) from the v2 table.
    #[serde(default)]
    pub flags_hnzvc: String,
    #[serde(default)]
    pub flags_read_hnzvc: String,
    #[serde(default)]
    pub undocumented: bool,
    pub addr_modes: HashMap<AddrModeEnum, OpcodeData>,
}

/// One MAME-verified undocumented opcode from the v2 table.
#[derive(Default, Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct UndocumentedOp {
    pub mnemonic: Mnemonic,
    pub opcode: usize,
    pub cycles: usize,
    pub size: usize,
    #[serde(default)]
    pub flags_hnzvc: String,
    #[serde(default)]
    pub behavior: String,
}

/// Convert a 6-char HINZVC string to the StatusReg bits it writes
/// ('*'/'0'/'1' all set the bit — the result is a written flag).
pub(crate) fn flags_from_hnzvc(s: &str) -> StatusReg {
    let mut flags = StatusReg::empty();
    let bits = [
        StatusReg::H,
        StatusReg::I,
        StatusReg::N,
        StatusReg::Z,
        StatusReg::V,
        StatusReg::C,
    ];
    for (i, ch) in s.chars().take(6).enumerate() {
        if ch != '-' {
            flags |= bits[i];
        }
    }
    flags
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
    /// Whether the instruction's operand performs a data-memory read/write.
    /// Instruction fetches and stack accesses are intentionally excluded.
    #[serde(default)]
    pub memory_read: bool,
    #[serde(default)]
    pub memory_write: bool,
    pub opcode: usize,
    pub cycles: usize,
    pub size: usize,
}

/// Compact identity for an opcode in the ISA database.
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct OpcodeId(pub usize);

impl OpcodeData {
    pub fn id(&self) -> OpcodeId {
        OpcodeId(self.opcode)
    }
}

impl From<&OpcodeData> for OpcodeId {
    fn from(data: &OpcodeData) -> Self {
        data.id()
    }
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
    /// MAME-verified undocumented opcodes (the v2 table's top-level
    /// `undocumented` array).
    #[serde(default)]
    pub undocumented: Vec<UndocumentedOp>,
}
