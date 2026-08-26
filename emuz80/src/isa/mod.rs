#![allow(dead_code)]
use serde::Deserialize;

use grl_isa::Dbase as GenericDbase;

pub use grl_isa::InstructionId;

/// Form-level Z80 addressing modes. Coarse by design: multiple
/// `(action, template)` rows share a mode (`LD A,(nn)` and `LD HL,(nn)`
/// are both `AbsoluteIndirect`); the template string disambiguates.
#[derive(Debug, Copy, Clone, Deserialize, PartialEq, Hash, Eq)]
pub enum AddrModeEnum {
    Inherent,
    Immediate8,
    Immediate16,
    ImmediateMode,      // IM 0/1/2
    Register,           // single 8-bit register operand (INC r, ADC A,r, ...)
    RegisterPair,       // 16-bit pair (INC dd, ADD HL,dd, POP BC, EX DE,HL, ...)
    RegisterRegister,   // LD r1,r2
    Indirect,           // (HL): LD r,(HL), LD (HL),n, INC (HL), ...
    Indexed,            // (IX+d)/(IY+d)/(IX)/(IY): LD r,(IX+d), JP (IX), ...
    BitRegister,        // BIT/SET/RES b,r
    BitIndirect,        // BIT/SET/RES b,(HL)
    BitIndexed,         // BIT/SET/RES b,(IX+d) (and the `,r` destination forms)
    AbsoluteIndirect,   // LD A,(nn), LD (nn),A
    AbsoluteIndirect16, // LD HL,(nn), LD (nn),HL, LD BC,(nn), ...
    RegisterIndirect,   // LD A,(BC)/(DE), LD (BC)/(DE),A
    StackRegister,      // LD SP,HL/IX/IY
    StackIndirect,      // EX (SP),HL/IX/IY
    SpecialRegister,    // LD A,I/R, LD I/R,A
    ExchangeAF,         // EX AF,AF'
    Port,               // IN A,(n), OUT (n),A, IN0 r,(n)
    PortRegister,       // IN r,(C), OUT (C),r
    Restart,            // RST p
    Relative,           // JR e, DJNZ e
    Condition,          // RET cc
    ConditionRelative,  // JR cc,e
    ConditionImmediate, // JP cc,nn, CALL cc,nn
}

/// `Instruction`/`InstructionInfo` bound to the Z80 mode type, so
/// consumers can write `isa::Instruction` without the generic parameter.
pub type Instruction = grl_isa::Instruction<AddrModeEnum>;
pub type InstructionInfo = grl_isa::InstructionInfo<AddrModeEnum>;

/// Z80 instruction database: the generic reader. The assembler resolves
/// by `(action, template)`; a future emulator decodes via `by_opcode`.
pub struct Dbase(GenericDbase<AddrModeEnum>);

impl Dbase {
    pub fn new() -> Self {
        Self(GenericDbase::from_text(include_str!(
            "../../resources/opcodesZ80.json"
        )))
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
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_generated_table() {
        let db = Dbase::new();
        assert_eq!(db.instructions().len(), 300);
        assert!(db.mnemonics().any(|m| m == "ldir"));
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
    fn alternate_encodings_are_preserved() {
        let db = Dbase::new();
        let rows = db.get("LD", "(nn),HL").unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].opcode, 0x22);
        assert_eq!(rows[1].opcode, 0xED63);
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
}
