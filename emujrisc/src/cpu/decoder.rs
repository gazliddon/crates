//! Instruction decoder.
//!
//! Encoding: `word = (opcode6 << 10) | (src5 << 5) | dst5`. `movei` (extra32)
//! adds two words holding a word-swapped 32-bit immediate (lo word first,
//! matching the `.dc.i` convention). Shared opcode numbers resolve per chip
//! via [`crate::isa::Dbase::lookup`].

use crate::cpu::Chip;
use crate::isa::{Dbase, Insn};
use emucore::mem::{MemReader, MemoryIO};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodeError {
    pub message: String,
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for DecodeError {}

/// A decoded instruction: a reference into the static ISA table (no
/// per-instruction copy of the ~90-byte entry).
#[derive(Debug, Clone)]
pub struct DecodedInsn {
    pub addr: usize,
    pub word: u16,
    pub insn: &'static Insn,
    /// raw src field (5 bits)
    pub src: u8,
    /// raw dst field (5 bits)
    pub dst: u8,
    /// word-swapped 32-bit immediate for extra32 instructions
    pub extra: Option<u32>,
    /// total size in bytes (2, or 6 for movei)
    pub size: usize,
}

impl DecodedInsn {
    /// the register index in the dst field (single-register ops, movei)
    pub fn dst_reg(&self) -> u8 {
        self.dst
    }
}

/// Split a 16-bit word into (opcode, src, dst).
pub fn decode_word(word: u16) -> (u8, u8, u8) {
    (
        ((word >> 10) & 0x3F) as u8,
        ((word >> 5) & 0x1F) as u8,
        (word & 0x1F) as u8,
    )
}

/// Decode one instruction at `addr` in `mem` for `chip`.
pub fn decode<M: MemoryIO>(
    mem: &mut M,
    addr: usize,
    chip: Chip,
) -> Result<DecodedInsn, DecodeError> {
    let mut reader = MemReader::new(mem);
    reader.set_addr(addr);
    let word = reader.next_word().map_err(|_| DecodeError {
        message: format!("truncated instruction at ${addr:08X}"),
    })?;
    let (op, src, dst) = decode_word(word);
    let insn = Dbase::get()
        .lookup(op as usize, chip.variant())
        .unwrap_or(&Dbase::get().unknown);
    let mut size = 2;
    let mut extra = None;
    if insn.extra32 {
        let w1 = reader.next_word().map_err(|_| DecodeError {
            message: format!("truncated movei at ${addr:08X}"),
        })?;
        let w2 = reader.next_word().map_err(|_| DecodeError {
            message: format!("truncated movei at ${addr:08X}"),
        })?;
        // word-swapped: low word first
        extra = Some(((w2 as u32) << 16) | w1 as u32);
        size = 6;
    }
    Ok(DecodedInsn {
        addr,
        word,
        insn,
        src,
        dst,
        extra,
        size,
    })
}

/// Built-in condition-code names (5-bit values, from the MadMac addendum:
/// T=%00000 NE=%00001 EQ=%00010 CC=%00100 HI=%00101 CS=%01000 PL=%10100
/// MI=%11000).
pub fn cc_name(v: u8) -> &'static str {
    match v & 0x1F {
        0 => "t",
        1 => "ne",
        2 => "eq",
        4 => "cc",
        5 => "hi",
        8 => "cs",
        20 => "pl",
        24 => "mi",
        other => {
            // unknown codes are still legal (user-definable via .ccdef)
            Box::leak(format!("cc{other}").into_boxed_str())
        }
    }
}
