#![allow(dead_code)]
use super::AddrModeEnum;
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;

////////////////////////////////////////////////////////////////////////////////

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

#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Instruction {
    /// Row index in the loaded table; assigned by [`Dbase::from_text`].
    #[serde(skip)]
    pub id: InstructionId,
    /// Canonical operand shape, e.g. `"A,(IX+d)"`, `"r1,r2"`, `"d"`, `""`.
    /// The assembler parser canonicalizes operand text to these strings.
    #[serde(default)]
    pub template: String,
    pub addr_mode: AddrModeEnum,
    pub cycles: usize,
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
    /// Position of the first operand byte within the instruction. Only the
    /// DD/FD CB d forms interleave (DD CB <d> <op>); everywhere else this
    /// equals `size - operand_size`.
    #[serde(default)]
    pub operand_offset: usize,
    /// Where each symbolic operand sits in the low opcode byte:
    /// `{var: shift}`, e.g. LD r1,r2 is `{"r1": 3, "r2": 0}`. Vars:
    /// r, r1, r2, dd, b, p. The backend computes `byte |= value << shift`.
    #[serde(default)]
    pub bit_fields: HashMap<String, u8>,
}

impl Instruction {
    pub fn id(&self) -> InstructionId {
        self.id
    }
}

/// Instructions for one mnemonic, indexed by operand template.
#[derive(Debug, Clone)]
pub struct InstructionInfo {
    pub mnemomic: String,
    pub ops: Vec<Instruction>,
    /// Template -> candidate rows. Several rows can share a template: the
    /// DD/FD prefixed variants of register forms (`LD r1,r2` has plain,
    /// DD, and FD rows) and alternate encodings (`LD (nn),HL` 0x22 vs
    /// 0xED63). The assembler backend picks by register/prefix preference.
    pub templates: HashMap<String, Vec<Instruction>>,
}

impl InstructionInfo {
    pub fn new(ins: Instruction) -> Self {
        let mut ret = Self {
            mnemomic: ins.action.clone(),
            ops: vec![],
            templates: Default::default(),
        };
        ret.add(&ins);
        ret
    }

    pub fn get(&self, template: &str) -> Option<&Vec<Instruction>> {
        self.templates.get(template)
    }

    pub fn add(&mut self, ins: &Instruction) {
        self.templates
            .entry(ins.template.clone())
            .or_default()
            .push(ins.clone());
        self.ops.push(ins.clone());
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Dbase {
    unknown: Instruction,
    instructions: Vec<Instruction>,
    #[serde(skip)]
    name_to_ins: HashMap<String, InstructionInfo>,
    #[serde(skip)]
    opcode_to_ins: HashMap<usize, Vec<Instruction>>,
}

impl Dbase {
    pub fn from_text(json_str: &str) -> Self {
        let Dbase {
            unknown,
            mut instructions,
            ..
        } = serde_json::from_str(json_str).unwrap();
        let mut name_to_ins: HashMap<String, InstructionInfo> = HashMap::new();
        let mut opcode_to_ins: HashMap<usize, Vec<Instruction>> = HashMap::new();

        for (i, ins) in instructions.iter_mut().enumerate() {
            ins.id = InstructionId(i);
            name_to_ins
                .entry(ins.action.clone())
                .or_insert_with(|| InstructionInfo::new(ins.clone()))
                .add(ins);
            opcode_to_ins
                .entry(ins.opcode)
                .or_default()
                .push(ins.clone());
        }

        Self {
            unknown,
            instructions,
            name_to_ins,
            opcode_to_ins,
        }
    }

    pub fn new() -> Self {
        let json_str = include_str!("../../resources/opcodesZ80.json");
        Self::from_text(json_str)
    }

    /// All instruction rows.
    pub fn instructions(&self) -> &[Instruction] {
        &self.instructions
    }

    pub fn unknown(&self) -> &Instruction {
        &self.unknown
    }

    /// The candidate rows for `(action, template)`, e.g. `("LD", "r1,r2")`.
    pub fn get(&self, action: &str, template: &str) -> Option<&Vec<Instruction>> {
        self.name_to_ins
            .get(action)
            .and_then(|info| info.get(template))
    }

    /// The instruction info block for a mnemonic (all its templates).
    pub fn get_info(&self, action: &str) -> Option<&InstructionInfo> {
        self.name_to_ins.get(action)
    }

    /// Resolve a stored [`InstructionId`] back to its row.
    pub fn get_by_id(&self, id: InstructionId) -> Option<&Instruction> {
        self.instructions.get(id.0)
    }

    /// Rows sharing a (zeroed) opcode value — the future decoder's entry point.
    pub fn by_opcode(&self, opcode: usize) -> Option<&Vec<Instruction>> {
        self.opcode_to_ins.get(&opcode)
    }

    pub fn mnemonics(&self) -> impl Iterator<Item = &String> {
        self.name_to_ins.keys()
    }
}

impl Default for Dbase {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_generated_table() {
        let db = Dbase::new();
        assert_eq!(db.instructions().len(), 300);
        assert!(db.mnemonics().any(|m| m == "LDIR"));
    }

    #[test]
    fn resolves_documented_opcodes() {
        let db = Dbase::new();
        assert_eq!(db.get("NOP", "").unwrap()[0].opcode, 0x00);
        assert_eq!(db.get("LDIR", "").unwrap()[0].opcode, 0xEDB0);
        assert_eq!(db.get("LD", "A,(nn)").unwrap()[0].opcode, 0x3A);
        assert_eq!(db.get("JR", "d").unwrap()[0].opcode, 0x18);
        assert_eq!(db.get("RST", "p").unwrap()[0].opcode, 0xC7);
    }

    #[test]
    fn register_forms_keep_base_and_variants() {
        let db = Dbase::new();
        let ld_rr = db.get("LD", "r1,r2").unwrap();
        assert_eq!(ld_rr.len(), 3); // plain + DD + FD
        assert_eq!(ld_rr[0].opcode, 0x40);
        assert_eq!(ld_rr[0].size, 1);
        assert_eq!(ld_rr[1].opcode, 0xDD40);
        assert_eq!(ld_rr[2].opcode, 0xFD40);

        let adc = db.get("ADC", "A,r").unwrap();
        assert_eq!(adc[0].opcode, 0x88);
        assert_eq!(adc[0].size, 1);
    }

    #[test]
    fn bit_indexed_keeps_only_documented_base() {
        let db = Dbase::new();
        let rows = db.get("BIT", "b,(IX+d)").unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].opcode, 0xDDCB46);
    }

    #[test]
    fn relative_and_indexed_operand_sizes() {
        let db = Dbase::new();
        let jr = db.get("JR", "d").unwrap()[0].clone();
        assert_eq!((jr.size, jr.operand_size), (2, 1));
        let ld_ix = db.get("LD", "r,(IX+d)").unwrap()[0].clone();
        assert_eq!((ld_ix.size, ld_ix.operand_size), (3, 1));
        let ld_nn = db.get("LD", "dd,nn").unwrap()[0].clone();
        assert_eq!((ld_nn.size, ld_nn.operand_size), (3, 2));
    }

    #[test]
    fn ids_round_trip_through_the_table() {
        let db = Dbase::new();
        let nop = db.get("NOP", "").unwrap()[0].clone();
        let again = db.get_by_id(nop.id()).unwrap();
        assert_eq!(again.opcode, 0x00);
        assert!(db.get_info("LDIR").is_some());
        assert!(db.get_info("NOTAMNEMONIC").is_none());
    }

    #[test]
    fn alternate_encodings_are_preserved() {
        let db = Dbase::new();
        let rows = db.get("LD", "(nn),HL").unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].opcode, 0x22);
        assert_eq!(rows[1].opcode, 0xED63);
    }
}
