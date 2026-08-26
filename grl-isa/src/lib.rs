//! Generic JSON instruction-set database reader.
//!
//! The emulator crates (emu6809, emuz80, ...) ship an instruction table as
//! JSON and wrap [`Dbase`] over their own [`AddrModeEnum`]-style mode type.
//! The assembler backend in gazm resolves instructions by
//! `(action, template)` — e.g. `("LD", "r1,r2")` — while the emulator
//! decodes through `by_opcode`/lookup.
//!
//! JSON row schema (all fields except `addr_mode`/`cycles`/`opcode`/
//! `action`/`size` are optional):
//! ```json
//! { "addr_mode": "Direct", "cycles": 6, "opcode": "00", "action": "NEG",
//!   "size": 2, "template": "r1,r2", "bit_fields": {"r1": 3, "r2": 0},
//!   "operand_offset": 2, "subroutine": false, "info": "...",
//!   "preferred": false }
//! ```
//! `opcode` is a hex string; prefix+opcode fit in one value (`LDIR` ->
//! 0xEDB0). Actions are stored lowercased with `/` replaced by `_` (the
//! 6809's `CALL/SUB` alias convention); mnemonic lookups are
//! case-insensitive. `preferred` (default true) marks rows the assembler
//! should emit: the table also lists undocumented alias opcodes for the
//! emulator's decoder, which opt out so the mode map keeps the canonical
//! encoding.

use serde::{Deserialize, Deserializer};
use std::collections::HashMap;
use std::fmt;

/// Stable identity for one instruction row: its index in the loaded table.
/// Syntax trees store this and resolve through [`Dbase::get_by_id`].
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Default)]
pub struct InstructionId(pub usize);

fn hex_str_to_num<'de, D>(deserializer: D) -> Result<usize, D::Error>
where
    D: Deserializer<'de>,
{
    let hex_string = String::deserialize(deserializer)?;
    usize::from_str_radix(&hex_string, 16).map_err(serde::de::Error::custom)
}

fn fixup_action<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let action = String::deserialize(deserializer)?;
    Ok(action.to_lowercase().replace('/', "_"))
}

/// `preferred` defaults to true: only undocumented alias rows opt out.
fn default_true() -> bool {
    true
}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Instruction<M> {
    /// Row index in the loaded table; assigned by [`Dbase::from_text`].
    #[serde(skip)]
    pub id: InstructionId,
    pub addr_mode: M,
    pub cycles: usize,
    #[serde(deserialize_with = "fixup_action")]
    pub action: String,
    /// Prefix + opcode as a single value with register/bit/vector fields
    /// zeroed (`LD r1,r2` -> 0x40; `LDIR` -> 0xEDB0). The backend ORs in
    /// the operand-dependent bits.
    #[serde(deserialize_with = "hex_str_to_num")]
    pub opcode: usize,
    /// Total instruction size in bytes (prefixes + opcode + operands).
    pub size: usize,
    /// Bytes that follow the opcode (n=1, nn=2, d=1).
    #[serde(default)]
    pub operand_size: usize,
    /// Canonical operand shape, e.g. `"A,(IX+d)"`, `"r1,r2"`, `"d"`, `""`.
    /// The assembler parser canonicalizes operand text to these strings.
    #[serde(default)]
    pub template: String,
    /// Position of the first operand byte within the instruction. Only the
    /// DD/FD CB d forms interleave (`DD CB <d> <op>`); everywhere else this
    /// equals `size - operand_size`.
    #[serde(default)]
    pub operand_offset: usize,
    /// Where each symbolic operand sits in the low opcode byte:
    /// `{var: shift}`, e.g. LD r1,r2 is `{"r1": 3, "r2": 0}`. Vars:
    /// r, r1, r2, dd, b, p. The backend computes `byte |= value << shift`.
    #[serde(default)]
    pub bit_fields: HashMap<String, u8>,
    #[serde(default)]
    pub subroutine: Option<bool>,
    #[serde(default)]
    pub info: Option<String>,
    /// Preferred for assembly. The table lists every opcode the CPU
    /// executes (the emulator decodes all of them); the assembler emits
    /// only the preferred subset. Undocumented alias rows — e.g. the 6809's
    /// `NEG` direct 0x01, aliasing the documented 0x00 — set this to
    /// `false` so the mode map keeps the canonical encoding.
    #[serde(default = "default_true")]
    pub preferred: bool,
}

impl<M> Instruction<M> {
    pub fn id(&self) -> InstructionId {
        self.id
    }
}

/// Instructions for one mnemonic, indexed by addressing mode and (for
/// template-driven backends) by operand template.
#[derive(Debug, Clone)]
pub struct InstructionInfo<M> {
    pub mnemonic: String,
    pub ops: Vec<Instruction<M>>,
    /// Mode -> instruction. Among rows with equal [`Instruction::preferred`]
    /// status the last row wins (legacy overwrite behavior); a later
    /// non-preferred alias row never displaces the preferred encoding.
    pub addressing_modes: HashMap<M, Instruction<M>>,
    /// Template -> candidate rows. Several rows can share a template (the
    /// DD/FD prefixed variants of register forms and alternate encodings);
    /// the assembler backend picks by register/prefix preference.
    pub templates: HashMap<String, Vec<Instruction<M>>>,
}

impl<M: Clone + Eq + std::hash::Hash> InstructionInfo<M> {
    pub fn new(ins: Instruction<M>) -> Self {
        let mut ret = Self {
            mnemonic: ins.action.clone(),
            ops: vec![],
            addressing_modes: Default::default(),
            templates: Default::default(),
        };
        ret.add(&ins);
        ret
    }

    pub fn add(&mut self, ins: &Instruction<M>) {
        // Keep the preferred (canonical) row for a mode when a later
        // non-preferred alias row would overwrite it; among equal
        // preference the last row wins, preserving legacy behavior for
        // tables that don't use the flag.
        match self.addressing_modes.get(&ins.addr_mode) {
            Some(existing) if existing.preferred && !ins.preferred => {}
            _ => {
                self.addressing_modes
                    .insert(ins.addr_mode.clone(), (*ins).clone());
            }
        }
        self.templates
            .entry(ins.template.clone())
            .or_default()
            .push((*ins).clone());
        self.ops.push((*ins).clone());
    }

    pub fn get_instruction(&self, amode: &M) -> Option<&Instruction<M>>
    where
        M: Eq + std::hash::Hash,
    {
        self.addressing_modes.get(amode)
    }

    pub fn get_instruction_id(&self, amode: M) -> Option<InstructionId>
    where
        M: Eq + std::hash::Hash,
    {
        self.addressing_modes.get(&amode).map(|i| i.id)
    }

    pub fn supports_addr_mode(&self, amode: M) -> bool
    where
        M: Eq + std::hash::Hash,
    {
        self.addressing_modes.contains_key(&amode)
    }

    pub fn get(&self, template: &str) -> Option<&Vec<Instruction<M>>> {
        self.templates.get(template)
    }
}

#[derive(Debug, Clone)]
pub struct Dbase<M> {
    unknown: Instruction<M>,
    instructions: Vec<Instruction<M>>,
    infos: Vec<InstructionInfo<M>>,
    name_to_ins: HashMap<String, usize>,
    opcode_to_ins: HashMap<usize, Vec<Instruction<M>>>,
    /// Row -> index of its info block (for alias actions like `CALL/SUB`
    /// the last alias name's info, mirroring the legacy per-opcode map).
    id_to_info: Vec<usize>,
}

/// The JSON's top level: `unknown` plus the flat `instructions` array.
#[derive(Deserialize)]
struct Raw<M> {
    unknown: Instruction<M>,
    instructions: Vec<Instruction<M>>,
}

/// Split a `/`-aliased action (`CALL/SUB`) into its mnemonic names.
fn split_opcodes(action: &str) -> Vec<String> {
    action.split('_').map(String::from).collect()
}

impl<M> Dbase<M>
where
    M: serde::de::DeserializeOwned + Clone + Eq + std::hash::Hash + fmt::Debug,
{
    pub fn from_text(json_str: &str) -> Self {
        let Raw {
            unknown,
            mut instructions,
        } = serde_json::from_str(json_str).unwrap();

        let mut infos: Vec<InstructionInfo<M>> = Vec::new();
        let mut name_to_ins: HashMap<String, usize> = HashMap::new();
        let mut opcode_to_ins: HashMap<usize, Vec<Instruction<M>>> = HashMap::new();
        let mut id_to_info: Vec<usize> = Vec::with_capacity(instructions.len());

        for ins in instructions.iter_mut() {
            ins.id = InstructionId(id_to_info.len());
            let mut last: Option<usize> = None;
            for name in split_opcodes(&ins.action) {
                let idx = *name_to_ins.entry(name).or_insert_with(|| {
                    infos.push(InstructionInfo::new(ins.clone()));
                    infos.len() - 1
                });
                infos[idx].add(ins);
                last = Some(idx);
            }
            id_to_info.push(last.unwrap_or(0));
            opcode_to_ins
                .entry(ins.opcode)
                .or_default()
                .push(ins.clone());
        }

        Self {
            unknown,
            instructions,
            infos,
            name_to_ins,
            opcode_to_ins,
            id_to_info,
        }
    }

    pub fn from_filename(file_name: &str) -> Self {
        let json_str = std::fs::read_to_string(file_name).unwrap();
        Self::from_text(&json_str)
    }

    /// All instruction rows.
    pub fn instructions(&self) -> &[Instruction<M>] {
        &self.instructions
    }

    pub fn unknown(&self) -> &Instruction<M> {
        &self.unknown
    }

    /// The candidate rows for `(action, template)`, e.g. `("LD", "r1,r2")`.
    pub fn get(&self, action: &str, template: &str) -> Option<&Vec<Instruction<M>>> {
        self.get_info(action).and_then(|info| info.get(template))
    }

    /// The instruction info block for a mnemonic (all its modes/templates).
    pub fn get_info(&self, action: &str) -> Option<&InstructionInfo<M>> {
        self.name_to_ins
            .get(&action.to_lowercase())
            .and_then(|&idx| self.infos.get(idx))
    }

    /// Resolve a stored [`InstructionId`] back to its row.
    pub fn get_by_id(&self, id: InstructionId) -> Option<&Instruction<M>> {
        self.instructions.get(id.0)
    }

    /// The info block that owns a stored row (via its id). The preference
    /// rules keep the canonical encoding in each mode map, so the final
    /// info is the one the assembler's mode lookups should see.
    pub fn get_info_by_id(&self, id: InstructionId) -> Option<&InstructionInfo<M>> {
        self.id_to_info
            .get(id.0)
            .and_then(|&idx| self.infos.get(idx))
    }

    /// Rows sharing an (zeroed) opcode value — the emulator decoder's entry
    /// point.
    pub fn by_opcode(&self, opcode: usize) -> Option<&Vec<Instruction<M>>> {
        self.opcode_to_ins.get(&opcode)
    }

    pub fn mnemonics(&self) -> impl Iterator<Item = &String> {
        self.name_to_ins.keys()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const Z80_JSON: &str = include_str!("../../emuz80/resources/opcodesZ80.json");
    const M6809_JSON: &str = include_str!("../../emu6809/resources/opcodes6809.json");

    // Union of the 6809 and Z80 addressing-mode sets (the reader is
    // generic over whichever mode enum the table uses).
    #[derive(Debug, Deserialize, Clone, PartialEq, Eq, Hash)]
    enum Mode {
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
        ImmediateMode,
        Register,
        RegisterRegister,
        Indirect,
        BitRegister,
        BitIndirect,
        BitIndexed,
        AbsoluteIndirect,
        AbsoluteIndirect16,
        RegisterIndirect,
        StackRegister,
        StackIndirect,
        SpecialRegister,
        ExchangeAF,
        Port,
        PortRegister,
        Restart,
        Condition,
        ConditionRelative,
        ConditionImmediate,
    }

    #[test]
    fn loads_6809_style_table() {
        let db: Dbase<Mode> = Dbase::from_text(M6809_JSON);
        assert!(db.instructions().len() > 100);
        assert!(db.get_info("neg").is_some());
        assert!(db.get_info("NEG").is_some()); // case-insensitive
        let neg = db.get_info("neg").unwrap();
        assert!(neg.supports_addr_mode(Mode::Direct));
        let row = neg.get_instruction(&Mode::Direct).unwrap();
        // The real 6809 has an undocumented alias (0x01 NEG direct aliases
        // the documented 0x00); the mode map keeps the *preferred* row, so
        // the assembler emits the canonical encoding.
        assert_eq!(row.opcode, 0x00);
        assert_eq!(row.size, 2);
        assert_eq!(row.action, "neg");
        assert!(row.preferred);
        // ...while the emulator still decodes the alias opcode.
        assert_eq!(db.by_opcode(0x01).unwrap()[0].opcode, 0x01);
        assert!(!db.by_opcode(0x01).unwrap()[0].preferred);
    }

    #[test]
    fn preferred_row_wins_over_later_alias() {
        let db: Dbase<Mode> = Dbase::from_text(M6809_JSON);
        // The 0x01 alias row comes last in the JSON, but the mode map must
        // keep the preferred 0x00 — the 6809 sizer's direct-page
        // optimization resolves through this map.
        let ext_row = db
            .get_info("neg")
            .unwrap()
            .get_instruction(&Mode::Extended)
            .unwrap();
        let info = db.get_info_by_id(ext_row.id()).unwrap();
        assert_eq!(info.get_instruction(&Mode::Direct).unwrap().opcode, 0x00);
    }

    #[test]
    fn ids_round_trip_and_templates_work() {
        let db: Dbase<Mode> = Dbase::from_text(Z80_JSON);
        let ldir = db.get_info("ldir").unwrap();
        let row = ldir.ops.first().unwrap();
        assert_eq!(row.opcode, 0xEDB0);
        assert_eq!(db.get_by_id(row.id()).unwrap().opcode, 0xEDB0);
        assert!(db.get_info_by_id(row.id()).is_some());
        let ld_rr = db.get("LD", "r1,r2").unwrap();
        assert_eq!(ld_rr.len(), 3); // plain + DD + FD
        assert_eq!(ld_rr[0].opcode, 0x40);
        assert_eq!(ld_rr[0].bit_fields.get("r1"), Some(&3));
    }

    #[test]
    fn by_opcode_indexes_rows() {
        let db: Dbase<Mode> = Dbase::from_text(Z80_JSON);
        let rows = db.by_opcode(0xEDB0).unwrap();
        assert_eq!(rows[0].action, "ldir");
    }
}
