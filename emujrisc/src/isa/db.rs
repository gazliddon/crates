#![allow(dead_code)]
//! JRISC instruction database.
//!
//! Schema of `resources/opcodes_jrisc.json` (v2):
//! ```json
//! { "mnemonic": "add", "opcode": 0, "variant": "any|gpu|dsp",
//!   "src": "<class>", "dst": "<class>", "extra32": false, "opswap": false,
//!   "class": "alu|shift|memory|branch|move|bit|sat|mac|nop|misc",
//!   "mem": "none|read8|read16|read32|write8|write16|write32",
//!   "flags": "zcn|zn|z|none", "mac": false, "cycles": 1, "size": 2,
//!   "reloc": "none|abs5|pcrel5|abs32_swap", "pad": false,
//!   "cc": [...], "desc": "...", "seen": false }
//! ```
//! The encoding is flat: `word = (opcode6 << 10) | (src5 << 5) | dst5`.
//! `extra32` instructions (movei) carry a word-swapped 32-bit immediate in
//! two extra words. `opswap` marks instructions whose *encoded* src/dst
//! fields are reversed relative to assembly syntax (store family, jump/jr).
//! `desc`/`seen`/`cc` are JSON-only documentation fields (not loaded here).

use serde::Deserialize;
use std::fmt;

#[derive(Debug, Copy, Clone, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum OperandClass {
    /// plain register Rn
    Reg,
    /// register indirect (Rn)
    IReg,
    /// 5-bit immediate 0-31
    Imm0,
    /// 5-bit immediate 1-32
    Imm1,
    /// 5-bit immediate, 32-(1-32) (shlq)
    Imm1s,
    /// 5-bit signed immediate -16..15
    SImm,
    /// 32-bit immediate in extra longword (movei)
    ImmLw,
    /// (r14+n) — the 5-bit displacement is in the src field
    Ir14d,
    /// (r15+n) — displacement in the src field
    Ir15d,
    /// (r14+rX) — X in the src field
    Ir14r,
    /// (r15+rX) — X in the src field
    Ir15r,
    /// condition code (jump/jr) — encoded in the dst field
    Cc,
    /// relative branch offset (jr) — encoded in the src field
    Rel,
    /// the PC register
    Pc,
    /// no operand
    None,
}

/// Which chip(s) an instruction is valid for.
#[derive(Debug, Copy, Clone, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum Variant {
    Any,
    Gpu,
    Dsp,
}

impl Variant {
    /// bitmask compatible with the GASM/vasm convention (GPU=1, DSP=2)
    pub fn flags(self) -> u8 {
        match self {
            Variant::Any => 3,
            Variant::Gpu => 1,
            Variant::Dsp => 2,
        }
    }
}

/// Semantic family — drives ALU dispatch in the executor.
#[derive(Debug, Copy, Clone, Deserialize, PartialEq, Eq, Hash, Default)]
#[serde(rename_all = "lowercase")]
pub enum InsnClass {
    #[default]
    Misc,
    Alu,
    Shift,
    Memory,
    Branch,
    Move,
    Bit,
    Sat,
    Mac,
    Nop,
}

/// Bus behaviour for the memory interface.
#[derive(Debug, Copy, Clone, Deserialize, PartialEq, Eq, Hash, Default)]
#[serde(rename_all = "lowercase")]
pub enum MemAccess {
    #[default]
    None,
    Read8,
    Read16,
    Read32,
    Write8,
    Write16,
    Write32,
}

impl MemAccess {
    pub fn is_read(self) -> bool {
        matches!(self, MemAccess::Read8 | MemAccess::Read16 | MemAccess::Read32)
    }
    pub fn is_write(self) -> bool {
        matches!(self, MemAccess::Write8 | MemAccess::Write16 | MemAccess::Write32)
    }
    pub fn width_bytes(self) -> usize {
        match self {
            MemAccess::Read8 | MemAccess::Write8 => 1,
            MemAccess::Read16 | MemAccess::Write16 => 2,
            MemAccess::Read32 | MemAccess::Write32 => 4,
            MemAccess::None => 0,
        }
    }
}

/// Condition flags affected by the instruction (G_FLAGS: Z bit0, C bit1, N bit2).
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub struct FlagSet {
    pub zero: bool,
    pub carry: bool,
    pub neg: bool,
}

impl FlagSet {
    /// parse the JSON shorthand: "zcn" | "zn" | "z" | "none"
    pub fn parse(s: &str) -> Self {
        Self {
            zero: s.contains('z'),
            carry: s.contains('c'),
            neg: s.contains('n'),
        }
    }
    pub fn is_empty(&self) -> bool {
        !self.zero && !self.carry && !self.neg
    }
}

/// Relocation kind for the operand fields (assembler/linker side).
#[derive(Debug, Copy, Clone, Deserialize, PartialEq, Eq, Hash, Default)]
#[serde(rename_all = "lowercase")]
pub enum Reloc {
    #[default]
    None,
    Abs5,
    Pcrel5,
    #[serde(rename = "abs32_swap")]
    Abs32Swap,
}

/// Per-mnemonic instruction kind, precomputed at build time. Matching on
/// the kind (a jump table) replaces string comparisons in the executor.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Kind {
    Abs,
    Add,
    Addc,
    Addq,
    Addqmod,
    Addqt,
    And,
    Bclr,
    Bset,
    Btst,
    Cmp,
    Cmpq,
    Div,
    Imacn,
    Imult,
    Imultn,
    Jr,
    Jump,
    Load,
    Loadb,
    Loadp,
    Loadr14d,
    Loadr14r,
    Loadr15d,
    Loadr15r,
    Loadw,
    Mirror,
    Mmult,
    Move,
    Movefa,
    Movei,
    Movepc,
    Moveq,
    Moveta,
    Mtoi,
    Mult,
    Neg,
    Nop,
    Normi,
    Not,
    Or,
    Pack,
    Resmac,
    Ror,
    Rorq,
    Sat16,
    Sat16s,
    Sat24,
    Sat32s,
    Sat8,
    Sh,
    Sha,
    Sharq,
    Shlq,
    Shrq,
    Store,
    Storeb,
    Storep,
    Storer14d,
    Storer14r,
    Storer15d,
    Storer15r,
    Storew,
    Sub,
    Subc,
    Subq,
    Subqmod,
    Subqt,
    Unpack,
    Xor,
    Unknown,
}

impl Kind {
    /// Map a mnemonic to its kind (build-time).
    pub fn of(mn: &str) -> Kind {
        match mn {
            "abs" => Kind::Abs,
            "add" => Kind::Add,
            "addc" => Kind::Addc,
            "addq" => Kind::Addq,
            "addqmod" => Kind::Addqmod,
            "addqt" => Kind::Addqt,
            "and" => Kind::And,
            "bclr" => Kind::Bclr,
            "bset" => Kind::Bset,
            "btst" => Kind::Btst,
            "cmp" => Kind::Cmp,
            "cmpq" => Kind::Cmpq,
            "div" => Kind::Div,
            "imacn" => Kind::Imacn,
            "imult" => Kind::Imult,
            "imultn" => Kind::Imultn,
            "jr" => Kind::Jr,
            "jump" => Kind::Jump,
            "load" => Kind::Load,
            "loadb" => Kind::Loadb,
            "loadp" => Kind::Loadp,
            "loadr14d" => Kind::Loadr14d,
            "loadr14r" => Kind::Loadr14r,
            "loadr15d" => Kind::Loadr15d,
            "loadr15r" => Kind::Loadr15r,
            "loadw" => Kind::Loadw,
            "mirror" => Kind::Mirror,
            "mmult" => Kind::Mmult,
            "move" => Kind::Move,
            "movefa" => Kind::Movefa,
            "movei" => Kind::Movei,
            "movepc" => Kind::Movepc,
            "moveq" => Kind::Moveq,
            "moveta" => Kind::Moveta,
            "mtoi" => Kind::Mtoi,
            "mult" => Kind::Mult,
            "neg" => Kind::Neg,
            "nop" => Kind::Nop,
            "normi" => Kind::Normi,
            "not" => Kind::Not,
            "or" => Kind::Or,
            "pack" => Kind::Pack,
            "resmac" => Kind::Resmac,
            "ror" => Kind::Ror,
            "rorq" => Kind::Rorq,
            "sat16" => Kind::Sat16,
            "sat16s" => Kind::Sat16s,
            "sat24" => Kind::Sat24,
            "sat32s" => Kind::Sat32s,
            "sat8" => Kind::Sat8,
            "sh" => Kind::Sh,
            "sha" => Kind::Sha,
            "sharq" => Kind::Sharq,
            "shlq" => Kind::Shlq,
            "shrq" => Kind::Shrq,
            "store" => Kind::Store,
            "storeb" => Kind::Storeb,
            "storep" => Kind::Storep,
            "storer14d" => Kind::Storer14d,
            "storer14r" => Kind::Storer14r,
            "storer15d" => Kind::Storer15d,
            "storer15r" => Kind::Storer15r,
            "storew" => Kind::Storew,
            "sub" => Kind::Sub,
            "subc" => Kind::Subc,
            "subq" => Kind::Subq,
            "subqmod" => Kind::Subqmod,
            "subqt" => Kind::Subqt,
            "unpack" => Kind::Unpack,
            "xor" => Kind::Xor,
            _ => Kind::Unknown,
        }
    }
}

/// One instruction entry. `mnemonic` is `&'static str`: JSON parsing uses an
/// owned intermediate and leaks the strings at build time only.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Insn {
    pub mnemonic: &'static str,
    pub kind: Kind,
    pub opcode: usize,
    pub variant: Variant,
    pub src: OperandClass,
    pub dst: OperandClass,
    pub extra32: bool,
    pub opswap: bool,
    pub class: InsnClass,
    pub mem: MemAccess,
    pub flags: FlagSet,
    pub mac: bool,
    pub cycles: usize,
    pub size: usize,
    pub reloc: Reloc,
    pub pad: bool,
}

impl Insn {
    pub fn valid_for(&self, variant: Variant) -> bool {
        self.variant.flags() & variant.flags() != 0
    }
}

/// Owned mirror of [`Insn`] used for JSON deserialization.
#[derive(Deserialize)]
struct RawInsn {
    mnemonic: String,
    opcode: usize,
    variant: Variant,
    src: OperandClass,
    dst: OperandClass,
    #[serde(default)]
    extra32: bool,
    #[serde(default)]
    opswap: bool,
    #[serde(default)]
    class: InsnClass,
    #[serde(default)]
    mem: MemAccess,
    #[serde(default = "default_flags")]
    flags: String,
    #[serde(default)]
    mac: bool,
    #[serde(default = "default_cycles")]
    cycles: usize,
    #[serde(default = "default_size")]
    size: usize,
    #[serde(default)]
    reloc: Reloc,
    #[serde(default)]
    pad: bool,
}

fn default_flags() -> String {
    "none".to_string()
}
fn default_cycles() -> usize {
    1
}
fn default_size() -> usize {
    2
}

impl RawInsn {
    fn into_insn(self) -> Insn {
        let kind = Kind::of(&self.mnemonic);
        Insn {
            mnemonic: Box::leak(self.mnemonic.into_boxed_str()),
            kind,
            opcode: self.opcode,
            variant: self.variant,
            src: self.src,
            dst: self.dst,
            extra32: self.extra32,
            opswap: self.opswap,
            class: self.class,
            mem: self.mem,
            flags: FlagSet::parse(&self.flags),
            mac: self.mac,
            cycles: self.cycles,
            size: self.size,
            reloc: self.reloc,
            pad: self.pad,
        }
    }
}

/// Loaded instruction database: unknown-instruction template + per-opcode
/// entries (multiple entries per opcode are possible: shared opcode numbers
/// with per-variant meanings).
#[derive(Debug, Clone)]
pub struct Dbase {
    pub unknown: Insn,
    pub by_opcode: Vec<Vec<Insn>>,
}

impl Dbase {
    /// Parse the JSON database (used by build.rs).
    pub fn from_text(json: &str) -> Self {
        #[derive(Deserialize)]
        struct Raw {
            unknown: RawInsn,
            instructions: Vec<RawInsn>,
        }
        let raw: Raw = serde_json::from_str(json).expect("opcodes_jrisc.json parse");
        let mut by_opcode: Vec<Vec<Insn>> = vec![Vec::new(); 64];
        for insn in raw.instructions {
            let insn = insn.into_insn();
            by_opcode[insn.opcode].push(insn);
        }
        Self { unknown: raw.unknown.into_insn(), by_opcode }
    }

    /// Look up an opcode for a variant. `Any` matches both; shared opcode
    /// numbers resolve by variant. Returns `None` when the opcode is not
    /// legal for the chip (e.g. GPU-only `sat24` decoded as DSP).
    pub fn lookup(&self, opcode: usize, variant: Variant) -> Option<&Insn> {
        self.by_opcode
            .get(opcode)?
            .iter()
            .find(|i| i.valid_for(variant))
    }
}

/// Title-case a mnemonic for its Kind variant name (non-identifier
/// mnemonics — e.g. the JSON "??" unknown template — map to Unknown).
fn cap(mn: &str) -> String {
    let mut c = mn.chars();
    match c.next() {
        Some(f) if f.is_ascii_alphabetic() => format!("{}{}", f.to_ascii_uppercase(), c.as_str()),
        _ => "Unknown".to_string(),
    }
}

/// Render one instruction as a fully-qualified const expression.
fn insn_literal(i: &Insn) -> String {
    let v = match i.variant {
        Variant::Any => "Variant::Any",
        Variant::Gpu => "Variant::Gpu",
        Variant::Dsp => "Variant::Dsp",
    };
    let src = format!("OperandClass::{i:?}", i = i.src);
    let dst = format!("OperandClass::{i:?}", i = i.dst);
    let class = format!("InsnClass::{i:?}", i = i.class);
    let mem = format!("MemAccess::{i:?}", i = i.mem);
    let reloc = format!("Reloc::{i:?}", i = i.reloc);
    format!(
        "Insn {{ mnemonic: {:?}, kind: Kind::{}, opcode: {}, variant: {}, src: {}, dst: {}, \
extra32: {}, opswap: {}, class: {}, mem: {}, flags: FlagSet {{ zero: {}, carry: {}, neg: {} }}, \
mac: {}, cycles: {}, size: {}, reloc: {}, pad: {} }}",
        i.mnemonic, cap(i.mnemonic), i.opcode, v, src, dst, i.extra32, i.opswap, class, mem,
        i.flags.zero, i.flags.carry, i.flags.neg, i.mac, i.cycles, i.size, reloc, i.pad,
    )
}

/// Emits the generated static tables consumed by `Dbase::new()`.
impl fmt::Display for Dbase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "// generated from resources/opcodes_jrisc.json — do not edit")?;
        writeln!(f, "use crate::isa::{{Insn, Kind, FlagSet, OperandClass, Variant, InsnClass, MemAccess, Reloc}};")?;
        writeln!(f, "pub static UNKNOWN: Insn = {};", insn_literal(&self.unknown))?;
        // entries in opcode order (all variants interleaved)
        let mut flat: Vec<&Insn> = Vec::new();
        writeln!(f, "pub static INSNS: [Insn; {}] = [", self.by_opcode.iter().map(Vec::len).sum::<usize>())?;
        for op in 0..64 {
            for insn in &self.by_opcode[op] {
                writeln!(f, "    {},", insn_literal(insn))?;
                flat.push(insn);
            }
        }
        writeln!(f, "];")?;
        writeln!(f, "pub static ALL: &[Insn] = &INSNS;")?;
        // per-variant 64-entry dispatch: [variant as usize][opcode] — the
        // first entry legal for that variant, else UNKNOWN (MAME's
        // per-chip legality; shared opcode numbers differ GPU/DSP)
        writeln!(f, "pub static DISPATCH: [[&'static Insn; 64]; 3] = [")?;
        for variant in [Variant::Any, Variant::Gpu, Variant::Dsp] {
            writeln!(f, "    [")?;
            for op in 0..64 {
                let hit = self.by_opcode[op].iter().find(|i| i.valid_for(variant));
                let target = match hit {
                    Some(insn) => {
                        let idx = flat.iter().position(|e| std::ptr::eq(*e, insn)).unwrap();
                        format!("&INSNS[{idx}]")
                    }
                    None => "&UNKNOWN".to_string(),
                };
                writeln!(f, "        {target},")?;
            }
            writeln!(f, "    ],")?;
        }
        writeln!(f, "];")
    }
}
