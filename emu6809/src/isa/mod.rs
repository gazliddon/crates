#![allow(dead_code)]
use serde::Deserialize;
use std::fmt;

use grl_isa::Dbase as GenericDbase;

pub use grl_isa::InstructionId;

#[derive(Debug, Copy, Clone, Deserialize, PartialEq, Hash, Eq)]
pub enum AddrModeEnum {
    Indexed,
    Direct,
    Extended,
    Relative,
    Relative16,
    Inherent,
    Immediate8,
    Immediate16,
    RegisterSet,
    RegisterPair,
}
impl fmt::Display for AddrModeEnum {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

/// `Instruction`/`InstructionInfo` bound to the 6809 mode type, so
/// consumers can write `isa::Instruction` without the generic parameter.
pub type Instruction = grl_isa::Instruction<AddrModeEnum>;
pub type InstructionInfo = grl_isa::InstructionInfo<AddrModeEnum>;

/// 6809 instruction database: the generic reader plus the emulator's
/// opcode-indexed lookup table and the `op_table!` macro generation.
pub struct Dbase {
    db: GenericDbase<AddrModeEnum>,
    /// opcode -> row, `unknown` filling the gaps (the decoder's fast path).
    lookup: Vec<Instruction>,
}

impl Dbase {
    pub fn from_text(json_str: &str) -> Self {
        let db = GenericDbase::from_text(json_str);
        let max = db
            .instructions()
            .iter()
            .map(|i| i.opcode)
            .max()
            .unwrap_or(0);
        let unknown = db.unknown().clone();
        let mut lookup = vec![unknown; max + 1];
        for ins in db.instructions() {
            lookup[ins.opcode] = ins.clone();
        }
        Self { db, lookup }
    }

    pub fn new() -> Self {
        Self::from_text(include_str!("../../resources/opcodes6809.json"))
    }

    /// Decode: the row for an opcode value, or the table's `unknown` row.
    pub fn get(&self, opcode: u16) -> &Instruction {
        self.lookup
            .get(opcode as usize)
            .unwrap_or(&self.db.unknown())
    }

    pub fn get_by_id(&self, id: InstructionId) -> &Instruction {
        self.db.get_by_id(id).expect("stale instruction id")
    }

    pub fn get_opcode(&self, input: &str) -> Option<&InstructionInfo> {
        self.db.get_info(&input.to_lowercase())
    }

    pub fn is_opcode(&self, input: &str) -> bool {
        self.get_opcode(input).is_some()
    }
}

impl Default for Dbase {
    fn default() -> Self {
        Self::new()
    }
}

impl std::ops::Deref for Dbase {
    type Target = GenericDbase<AddrModeEnum>;
    fn deref(&self) -> &Self::Target {
        &self.db
    }
}

/// The `op_table!` macro source consumed by the emulator's executor
/// (generated at build time into `isa_macros_6809.rs`).
impl fmt::Display for Dbase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let header = r#"#[macro_export]
macro_rules! op_table {
    ($op:expr, $fail:block) => {
        match $op {"#;

        let footer = r#"
            _ => $fail
        }
    }
}"#;

        writeln!(f, "{header}")?;

        for i in self.db.instructions() {
            writeln!(
                f,
                "\t\t0x{:04x} => handle_op!({:?}, {}, 0x{:04x}, {}, {}),",
                i.opcode, i.addr_mode, i.action, i.opcode, i.cycles, i.size
            )?;
        }
        writeln!(f, "{footer}")
    }
}
