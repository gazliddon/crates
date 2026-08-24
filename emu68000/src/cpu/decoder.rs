//! M68000 instruction decoder.
//!
//! Decoding mirrors MAME's Musashi-derived 68000 disassembler: the entry is
//! chosen by walking the table in mask-specificity order (first
//! `(word & mask) == pattern` match) and then validating the effective
//! address against the entry's `ea_mask` (plus MOVE's fixed destination
//! check, exactly like MAME's `build_opcode_table()` special case).
//!
//! EA byte: mode bits 5-3, reg bits 2-0 (MOVE's destination uses mode bits
//! 8-6, reg bits 11-9). Extension-word counts: modes 101/110/111(000/010/011)
//! = 1, 111(001) = 2, 111(100) = the immediate (1 word B/W, 2 words L).

use crate::isa::{Dbase, Form, Insn, Size, UNKNOWN};
use emucore::mem::{MemErrorTypes, MemoryIO};
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

impl From<MemErrorTypes> for DecodeError {
    fn from(e: MemErrorTypes) -> Self {
        Self { message: format!("{e}") }
    }
}

/// One effective-address operand.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Ea {
    /// mode bits (0-7)
    pub mode: u8,
    /// register bits (0-7)
    pub reg: u8,
    /// extension words in fetch order (d16(An) 1, d8(An,Xn) 1, abs.W 1,
    /// abs.L 2, d16(PC) 1, d8(PC,Xn) 1, #imm size-dependent)
    pub ext: Vec<u16>,
    /// PC-relative reference address (extension-word address + 2)
    pub ref_pc: Option<u32>,
}

impl Ea {
    pub fn new(word: u16) -> Self {
        Self {
            mode: ((word >> 3) & 7) as u8,
            reg: (word & 7) as u8,
            ext: Vec::new(),
            ref_pc: None,
        }
    }

    pub fn from_parts(mode: u8, reg: u8) -> Self {
        Self { mode, reg, ext: Vec::new(), ref_pc: None }
    }

    /// number of extension words for this EA (immediate mode counts the
    /// immediate's own words for the given operation size)
    pub fn ext_words(&self, size: Size) -> usize {
        match self.mode {
            5 | 6 => 1,
            7 => match self.reg {
                0 | 2 | 3 => 1,
                1 => 2,
                4 => size.imm_words(),
                _ => 0,
            },
            _ => 0,
        }
    }

    /// MAME `valid_ea()`: is this mode/reg allowed by `mask`?
    pub fn is_valid(&self, mask: u16) -> bool {
        let bit = match self.mode {
            0 => 0x800,
            1 => 0x400,
            2 => 0x200,
            3 => 0x100,
            4 => 0x080,
            5 => 0x040,
            6 => 0x020,
            7 => match self.reg {
                0 => 0x010,
                1 => 0x008,
                2 => 0x002,
                3 => 0x001,
                4 => 0x004,
                _ => 0,
            },
            _ => 0,
        };
        mask & bit != 0
    }

    /// PC-relative modes (d16(PC) / d8(PC,Xn)).
    pub fn is_pcrel(&self) -> bool {
        self.mode == 7 && (self.reg == 2 || self.reg == 3)
    }
}

/// A decoded instruction.
#[derive(Debug, Clone)]
pub struct DecodedInsn {
    pub addr: usize,
    pub word: u16,
    pub insn: Insn,
    /// total length in bytes
    pub size: usize,
    /// register fields: reg1 = bits 11-9, reg2 = bits 2-0
    pub reg1: u8,
    pub reg2: u8,
    /// the entry's EA operand (bits 5-0) when the form uses one
    pub ea: Option<Ea>,
    /// MOVE destination EA (mode bits 8-6, reg bits 11-9)
    pub ea_dst: Option<Ea>,
    /// immediate operand (imm_ea / bitimm_ea / imm_only / imm16 / movep d16)
    pub imm: Option<u32>,
    /// movem register mask
    pub regmask: Option<u16>,
    /// branch target address (bcc8 / bcc16 / dbcc)
    pub label: Option<u32>,
    /// raw 8-bit displacement of bcc8 forms
    pub disp8: Option<i8>,
}

/// Fetch one BE word at `addr`.
pub fn load_word<M: MemoryIO>(mem: &mut M, addr: usize) -> Result<u16, DecodeError> {
    mem.load_word(addr).map_err(Into::into)
}

/// Choose the table entry for `word`, applying EA validity (MAME order).
pub fn find(word: u16) -> &'static Insn {
    let db = Dbase::get();
    for i in &db.entries {
        if (word & i.mask) != i.pattern {
            continue;
        }
        if ea_ok(i, word) {
            return i;
        }
    }
    &UNKNOWN
}

/// EA validity for a candidate entry (MAME's `valid_ea` gate + the MOVE
/// destination special case).
fn ea_ok(i: &Insn, word: u16) -> bool {
    if i.form == Form::Move {
        let src = Ea::new(word);
        if !src.is_valid(i.ea_mask) {
            return false;
        }
        // MOVE's destination must be data-addressable (no An / PC / imm):
        // MAME checks it against the fixed 0xbf8 mask.
        let dst = Ea::from_parts(((word >> 6) & 7) as u8, ((word >> 9) & 7) as u8);
        return dst.is_valid(0xBF8);
    }
    match i.ea {
        "src" | "dst" => Ea::new(word).is_valid(i.ea_mask),
        _ => true,
    }
}

/// Decode one instruction at `addr`.
pub fn decode<M: MemoryIO>(mem: &mut M, addr: usize) -> Result<DecodedInsn, DecodeError> {
    let word = load_word(mem, addr)?;
    let insn = *find(word);
    let reg1 = ((word >> 9) & 7) as u8;
    let reg2 = (word & 7) as u8;

    let mut size = 2usize;
    let mut at = addr + 2;
    let mut ea: Option<Ea> = None;
    let mut ea_dst: Option<Ea> = None;
    let mut imm: Option<u32> = None;
    let mut regmask: Option<u16> = None;
    let mut label: Option<u32> = None;
    let mut disp8: Option<i8> = None;

    /// fetch the EA's extension words, advancing `at`/`size`; records the
    /// PC-relative reference address.
    fn fetch_ea_ext<M: MemoryIO>(
        mem: &mut M,
        at: &mut usize,
        size: &mut usize,
        e: &mut Ea,
        op_size: Size,
    ) -> Result<(), DecodeError> {
        let n = e.ext_words(op_size);
        let mut ext = Vec::with_capacity(n);
        for _ in 0..n {
            ext.push(load_word(mem, *at)?);
            *at += 2;
            *size += 2;
        }
        if e.is_pcrel() {
            // reference = extension-word address + 2
            e.ref_pc = Some((*at - n * 2) as u32 + 2);
        }
        e.ext = ext;
        Ok(())
    }

    match insn.form {
        Form::Move => {
            let mut s = Ea::new(word);
            fetch_ea_ext(mem, &mut at, &mut size, &mut s, insn.size)?;
            ea = Some(s);
            let mut d = Ea::from_parts(((word >> 6) & 7) as u8, ((word >> 9) & 7) as u8);
            fetch_ea_ext(mem, &mut at, &mut size, &mut d, insn.size)?;
            ea_dst = Some(d);
        }
        Form::ImmEa => {
            let n = insn.size.imm_words();
            let mut v: u32 = 0;
            for _ in 0..n {
                v = (v << 16) | load_word(mem, at)? as u32;
                at += 2;
                size += 2;
            }
            imm = Some(v);
            let mut e = Ea::new(word);
            fetch_ea_ext(mem, &mut at, &mut size, &mut e, insn.size)?;
            ea = Some(e);
        }
        Form::BitImmEa => {
            imm = Some(load_word(mem, at)? as u32);
            at += 2;
            size += 2;
            let mut e = Ea::new(word);
            fetch_ea_ext(mem, &mut at, &mut size, &mut e, Size::B)?;
            ea = Some(e);
        }
        Form::ImmOnly | Form::Imm16 | Form::Link | Form::Movep => {
            let n = if insn.form == Form::ImmOnly { insn.size.imm_words() } else { 1 };
            let mut v: u32 = 0;
            for _ in 0..n {
                v = (v << 16) | load_word(mem, at)? as u32;
                at += 2;
                size += 2;
            }
            imm = Some(v);
        }
        Form::Bcc8 => {
            // 8-bit displacement is in the opcode word itself (bits 7-0)
            let d = (word & 0xFF) as u8 as i8;
            disp8 = Some(d);
            label = Some((addr as i64 + 2 + d as i64) as u32);
        }
        Form::Bcc16 => {
            let d = load_word(mem, at)? as i16;
            size += 2;
            label = Some((addr as i64 + 2 + d as i64) as u32);
        }
        Form::Dbcc => {
            let d = load_word(mem, at)? as i16;
            size += 2;
            // displacement is relative to the displacement word itself
            // (addr + 2), matching MAME's d68000_dbcc
            label = Some((addr as i64 + 2 + d as i64) as u32);
        }
        Form::MovemRe | Form::MovemEr | Form::MovemPd => {
            regmask = Some(load_word(mem, at)?);
            at += 2;
            size += 2;
            let mut e = Ea::new(word);
            fetch_ea_ext(mem, &mut at, &mut size, &mut e, Size::W)?;
            ea = Some(e);
        }
        _ => {
            if insn.ea == "src" || insn.ea == "dst" {
                let mut e = Ea::new(word);
                fetch_ea_ext(mem, &mut at, &mut size, &mut e, insn.size)?;
                ea = Some(e);
            }
        }
    }

    Ok(DecodedInsn {
        addr,
        word,
        insn,
        size,
        reg1,
        reg2,
        ea,
        ea_dst,
        imm,
        regmask,
        label,
        disp8,
    })
}

/// Disassemble a whole region starting at `start` until `end` (exclusive)
/// or until memory runs out. Returns (list, next address).
pub fn decode_range<M: MemoryIO>(
    mem: &mut M,
    start: usize,
    end: usize,
) -> Result<Vec<DecodedInsn>, DecodeError> {
    let mut out = Vec::new();
    let mut pc = start;
    while pc < end {
        match decode(mem, pc) {
            Ok(d) => {
                if d.size == 0 {
                    break;
                }
                pc += d.size;
                out.push(d);
            }
            Err(e) => {
                // truncated at the end of the image: stop quietly
                if pc + 2 > end {
                    break;
                }
                return Err(e);
            }
        }
    }
    Ok(out)
}
