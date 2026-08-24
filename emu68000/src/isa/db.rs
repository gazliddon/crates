#![allow(dead_code)]
//! M68000 instruction database.
//!
//! Schema of `resources/opcodes68000.json` (v2):
//! ```json
//! { "mnemonic": "asr", "mask": 0xF1F8, "pattern": 0xE000, "size": "B",
//!   "form": "shift_imm", "src": "M_IMMED", "dst": "M_DREG", "ea": "NONE",
//!   "ext": "none", "ea_mask": 0, "cycles": 6, "desc": "...", "mem": "none",
//!   "cls1": "D", "cls2": "A" }
//! ```
//! The mask/pattern/ea_mask triples mirror MAME's m68kdasm.cpp opcode table
//! (base 68000 subset; validated to behave identically over all 65536
//! words). Entries are emitted into the generated static sorted by mask
//! popcount descending, exactly like MAME's `build_opcode_table()`, so the
//! first mask match is the most specific entry.
//!
//! [`Form`] drives operand rendering and decoding; [`Insn::ext`] drives
//! instruction length. `src`/`dst` keep the MadMac operand-class spellings
//! as documentation for the future executor. `cls1`/`cls2` (exg) pick the
//! register-class letter for the two register fields.

use serde::Deserialize;
use std::fmt;

#[derive(Debug, Copy, Clone, Deserialize, PartialEq, Eq, Hash)]
pub enum Size {
    #[serde(rename = "B")]
    B,
    #[serde(rename = "W")]
    W,
    #[serde(rename = "L")]
    L,
    #[serde(rename = "-")]
    None,
}

impl Size {
    /// operand width in bytes (None => 0)
    pub fn bytes(self) -> usize {
        match self {
            Size::B => 1,
            Size::W => 2,
            Size::L => 4,
            Size::None => 0,
        }
    }
    /// immediate extension words for this size (1 for B/W, 2 for L)
    pub fn imm_words(self) -> usize {
        match self {
            Size::L => 2,
            _ => 1,
        }
    }
    pub fn suffix(self) -> &'static str {
        match self {
            Size::B => "b",
            Size::W => "w",
            Size::L => "l",
            Size::None => "",
        }
    }
}

/// Operand layout form; see the generator header for the rendering contract.
#[derive(Debug, Copy, Clone, Deserialize, PartialEq, Eq, Hash)]
pub enum Form {
    #[serde(rename = "ea_reg")]
    EaReg,
    #[serde(rename = "reg_ea")]
    RegEa,
    #[serde(rename = "ea_addr")]
    EaAddr,
    #[serde(rename = "reg_reg")]
    RegReg,
    #[serde(rename = "exg")]
    Exg,
    #[serde(rename = "predec_predec")]
    PredecPredec,
    #[serde(rename = "postinc_postinc")]
    PostincPostinc,
    #[serde(rename = "imm_ea")]
    ImmEa,
    #[serde(rename = "bitimm_ea")]
    BitImmEa,
    #[serde(rename = "imm_only")]
    ImmOnly,
    #[serde(rename = "move")]
    Move,
    #[serde(rename = "moveq")]
    Moveq,
    #[serde(rename = "addq")]
    Addq,
    #[serde(rename = "shift_imm")]
    ShiftImm,
    #[serde(rename = "shift_reg")]
    ShiftReg,
    #[serde(rename = "shift_ea")]
    ShiftEa,
    #[serde(rename = "bcc8")]
    Bcc8,
    #[serde(rename = "bcc16")]
    Bcc16,
    #[serde(rename = "dbcc")]
    Dbcc,
    #[serde(rename = "scc")]
    Scc,
    #[serde(rename = "movem_re")]
    MovemRe,
    #[serde(rename = "movem_er")]
    MovemEr,
    #[serde(rename = "movem_pd")]
    MovemPd,
    #[serde(rename = "link")]
    Link,
    #[serde(rename = "movep")]
    Movep,
    #[serde(rename = "trap")]
    Trap,
    #[serde(rename = "ea")]
    Ea,
    #[serde(rename = "dreg")]
    Dreg,
    #[serde(rename = "areg")]
    Areg,
    #[serde(rename = "imm16")]
    Imm16,
    #[serde(rename = "none")]
    None,
}

/// One instruction entry. Strings are `&'static str`: JSON parsing uses an
/// owned intermediate and leaks the strings at build time only.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Insn {
    pub mnemonic: &'static str,
    pub mask: u16,
    pub pattern: u16,
    pub size: Size,
    pub form: Form,
    pub src: &'static str,
    pub dst: &'static str,
    /// which operand is the memory/EA operand: "src" | "dst" | "NONE"
    pub ea: &'static str,
    /// instruction-length driver: ea | imm | imm_only | label8 | label16 |
    /// regmask | imm16 | none
    pub ext: &'static str,
    /// valid-EA-mode bitmap (MAME convention: Dn 0x800 .. #imm 0x004)
    pub ea_mask: u16,
    pub cycles: usize,
    pub desc: &'static str,
    pub mem: &'static str,
    /// register-class letters for the two register fields (exg): 0 = D, 1 = A
    pub cls1: u8,
    pub cls2: u8,
}

/// The unknown-instruction template (rendered as `dc.w`).
pub const UNKNOWN: Insn = Insn {
    mnemonic: "dc.w",
    mask: 0,
    pattern: 0,
    size: Size::None,
    form: Form::None,
    src: "NONE",
    dst: "NONE",
    ea: "NONE",
    ext: "none",
    ea_mask: 0,
    cycles: 0,
    desc: "illegal / unmatched opcode",
    mem: "none",
    cls1: 0,
    cls2: 0,
};

/// Owned mirror of [`Insn`] used for JSON deserialization.
#[derive(Deserialize)]
struct RawInsn {
    mnemonic: String,
    mask: u16,
    pattern: u16,
    size: Size,
    form: Form,
    src: String,
    dst: String,
    ea: String,
    ext: String,
    ea_mask: u16,
    #[serde(default)]
    cycles: usize,
    #[serde(default)]
    desc: String,
    #[serde(default)]
    mem: String,
    #[serde(default)]
    cls1: String,
    #[serde(default)]
    cls2: String,
}

impl RawInsn {
    fn into_insn(self) -> Insn {
        Insn {
            mnemonic: leak(self.mnemonic),
            mask: self.mask,
            pattern: self.pattern,
            size: self.size,
            form: self.form,
            src: leak(self.src),
            dst: leak(self.dst),
            ea: leak(self.ea),
            ext: leak(self.ext),
            ea_mask: self.ea_mask,
            cycles: self.cycles,
            desc: leak(self.desc),
            mem: leak(self.mem),
            cls1: if self.cls1 == "A" { 1 } else { 0 },
            cls2: if self.cls2 == "A" { 1 } else { 0 },
        }
    }
}

fn leak(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}

/// Mask-popcount comparison used by MAME's `build_opcode_table()` (most
/// specific mask first). Stable: entries with equal popcount keep JSON order.
fn mask_popcount(mask: u16) -> u32 {
    mask.count_ones()
}

/// The instruction database: entries sorted by mask specificity.
#[derive(Debug, Clone)]
pub struct Dbase {
    pub entries: Vec<Insn>,
}

impl Dbase {
    /// Parse the JSON database (used by build.rs); sorts entries by mask
    /// popcount descending.
    pub fn from_text(json: &str) -> Self {
        #[derive(Deserialize)]
        struct Raw {
            instructions: Vec<RawInsn>,
        }
        let raw: Raw = serde_json::from_str(json).expect("opcodes68000.json parse");
        let mut entries: Vec<Insn> = raw.instructions.into_iter().map(RawInsn::into_insn).collect();
        entries.sort_by_key(|i| std::cmp::Reverse(mask_popcount(i.mask)));
        Self { entries }
    }

    /// First entry whose mask/pattern matches `word` (mask-only, no EA
    /// validation — the decoder applies EA checks).
    pub fn lookup(&self, word: u16) -> Option<&Insn> {
        self.entries.iter().find(|i| (word & i.mask) == i.pattern)
    }

    /// Resolve `word` with MAME's full gate (mask specificity order +
    /// EA validity), returning the entry index or the sentinel index.
    /// This is what `build_opcode_table()` computes per word; build.rs
    /// emits the result as the static [`DISPATCH`] table.
    pub fn resolve_index(&self, word: u16) -> usize {
        self.entries
            .iter()
            .position(|i| (word & i.mask) == i.pattern && gate_ok(i, word))
            .unwrap_or(self.entries.len())
    }
}

/// EA-validity bit for a mode/reg pair (MAME's `valid_ea` mapping).
pub fn mode_bit(mode: u8, reg: u8) -> u16 {
    match mode {
        0 => 0x800,
        1 => 0x400,
        2 => 0x200,
        3 => 0x100,
        4 => 0x080,
        5 => 0x040,
        6 => 0x020,
        7 => match reg {
            0 => 0x010,
            1 => 0x008,
            2 => 0x002,
            3 => 0x001,
            4 => 0x004,
            _ => 0,
        },
        _ => 0,
    }
}

/// MAME's EA gate for a candidate entry: the entry's `ea_mask` applied to
/// its EA operand (bits 5-0), plus MOVE's fixed destination check (mode
/// bits 8-6, reg bits 11-9 against 0xBF8). Build-safe: no `Ea` type.
pub fn gate_ok(insn: &Insn, word: u16) -> bool {
    if insn.form == Form::Move {
        if mode_bit(((word >> 3) & 7) as u8, (word & 7) as u8) & insn.ea_mask == 0 {
            return false;
        }
        let dst = (((word >> 6) & 7) << 3) | ((word >> 9) & 7);
        return mode_bit(((dst >> 3) & 7) as u8, (dst & 7) as u8) & 0xBF8 != 0;
    }
    match insn.ea {
        "src" | "dst" => mode_bit(((word >> 3) & 7) as u8, (word & 7) as u8) & insn.ea_mask != 0,
        _ => true,
    }
}

/// Render one instruction as a fully-qualified const expression.
fn insn_literal(i: &Insn) -> String {
    let size = match i.size {
        Size::B => "Size::B",
        Size::W => "Size::W",
        Size::L => "Size::L",
        Size::None => "Size::None",
    };
    let form = format!("Form::{}", form_variant(i.form));
    format!(
        "Insn {{ mnemonic: {:?}, mask: 0x{:04X}, pattern: 0x{:04X}, size: {}, form: {}, \
src: {:?}, dst: {:?}, ea: {:?}, ext: {:?}, ea_mask: 0x{:04X}, cycles: {}, desc: {:?}, \
mem: {:?}, cls1: {}, cls2: {} }}",
        i.mnemonic, i.mask, i.pattern, size, form, i.src, i.dst, i.ea, i.ext, i.ea_mask,
        i.cycles, i.desc, i.mem, i.cls1, i.cls2,
    )
}

fn form_variant(f: Form) -> &'static str {
    match f {
        Form::EaReg => "EaReg",
        Form::RegEa => "RegEa",
        Form::EaAddr => "EaAddr",
        Form::RegReg => "RegReg",
        Form::Exg => "Exg",
        Form::PredecPredec => "PredecPredec",
        Form::PostincPostinc => "PostincPostinc",
        Form::ImmEa => "ImmEa",
        Form::BitImmEa => "BitImmEa",
        Form::ImmOnly => "ImmOnly",
        Form::Move => "Move",
        Form::Moveq => "Moveq",
        Form::Addq => "Addq",
        Form::ShiftImm => "ShiftImm",
        Form::ShiftReg => "ShiftReg",
        Form::ShiftEa => "ShiftEa",
        Form::Bcc8 => "Bcc8",
        Form::Bcc16 => "Bcc16",
        Form::Dbcc => "Dbcc",
        Form::Scc => "Scc",
        Form::MovemRe => "MovemRe",
        Form::MovemEr => "MovemEr",
        Form::MovemPd => "MovemPd",
        Form::Link => "Link",
        Form::Movep => "Movep",
        Form::Trap => "Trap",
        Form::Ea => "Ea",
        Form::Dreg => "Dreg",
        Form::Areg => "Areg",
        Form::Imm16 => "Imm16",
        Form::None => "None",
    }
}

/// Emits the generated static tables: the instruction array, the unknown
/// sentinel, and the 64K per-word dispatch table (MAME's
/// `build_opcode_table()` result, resolved at build time).
impl fmt::Display for Dbase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "// generated from resources/opcodes68000.json — do not edit")?;
        writeln!(f, "use crate::isa::{{Insn, Size, Form}};")?;
        writeln!(f, "pub static INSNS: [Insn; {}] = [", self.entries.len())?;
        for insn in &self.entries {
            writeln!(f, "    {},", insn_literal(insn))?;
        }
        writeln!(f, "];")?;
        writeln!(f, "pub static UNKNOWN: Insn = {};", insn_literal(&UNKNOWN))?;
        // one entry per opcode word; `sentinel` is the index past the last
        // real instruction, pointing at UNKNOWN
        let sentinel = self.entries.len();
        writeln!(f, "pub static DISPATCH: [&'static Insn; 0x10000] = [")?;
        for chunk in (0..0x10000u32).collect::<Vec<_>>().chunks(16) {
            let line: Vec<String> = chunk
                .iter()
                .map(|&w| {
                    let i = self.resolve_index(w as u16);
                    if i == sentinel {
                        "&UNKNOWN".to_string()
                    } else {
                        format!("&INSNS[{i}]")
                    }
                })
                .collect();
            writeln!(f, "    {},", line.join(", "))?;
        }
        writeln!(f, "];")
    }
}
