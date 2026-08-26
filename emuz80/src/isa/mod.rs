//! Z80 instruction set database.
//!
//! The table lives in `resources/opcodesZ80.json` (generated from Deep
//! Toaster's MIT-licensed opcode table by `tools/gen_opcodes_z80.py`) and
//! is loaded here into a [`Dbase`]. The gazm assembler backend resolves
//! instructions by `(action, template)` — e.g. `("LD", "r1,r2")` — where
//! `template` is the canonical operand shape; a future emulator can decode
//! through `opcode_to_ins`.
//!
//! Note on `opcode`: register/bit/vector fields are zeroed. `LD r1,r2` has
//! base `0x40`; the backend ORs in the register bits (`0x40 | r1<<3 | r2`).

mod isa_reader;
pub use isa_reader::*;

use serde::Deserialize;

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
