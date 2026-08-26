//! Disassembler.
//!
//! Mirrors the emujrisc pattern: a [`Diss`] renders decoded instructions
//! (optionally resolving branch/PC-relative targets against a label map).
//! Rendering follows the ALN/MadMac source style of the Imagitec module
//! (e.g. `move.w #$00F1A114,$0C(A0)`, `movem.l D0-D7/A0-A6,-(A7)`).

use crate::cpu::{decode, DecodeError, DecodedInsn, Ea};
use crate::isa::{Form, Size};
use emucore::mem::MemoryIO;

/// One rendered line.
#[derive(Debug, Clone)]
pub struct Disassembly {
    pub addr: usize,
    pub size: usize,
    pub text: String,
    pub decoded: DecodedInsn,
}

/// A memory image to disassemble from (24-bit-addressable).
pub struct DissCtx {
    pub base: usize,
    pub data: Vec<u8>,
}

impl DissCtx {
    pub fn from_slice(addr: usize, _name: &str, data: &[u8]) -> Self {
        Self {
            base: addr,
            data: data.to_vec(),
        }
    }

    /// Convenience: disassemble one instruction at `addr`.
    pub fn diss(&self, d: &Diss, addr: usize) -> Result<Disassembly, DecodeError> {
        let mut mem = SliceMem::new(self.base, &self.data);
        d.diss(&mut mem, addr)
    }
}

/// Read-only byte-slice memory view (24/32-bit addresses, big-endian words).
pub struct SliceMem<'a> {
    pub base: usize,
    pub data: &'a [u8],
}

impl<'a> SliceMem<'a> {
    pub fn new(base: usize, data: &'a [u8]) -> Self {
        Self { base, data }
    }

    fn idx2(&self, addr: usize) -> Option<usize> {
        let i = addr.checked_sub(self.base)?;
        (i + 2 <= self.data.len()).then_some(i)
    }
}

impl MemoryIO for SliceMem<'_> {
    #[inline(always)]
    fn inspect_byte(&self, addr: usize) -> emucore::mem::MemResult<u8> {
        let i = addr
            .checked_sub(self.base)
            .ok_or(emucore::mem::MemErrorTypes::IllegalAddress(addr))?;
        self.data
            .get(i)
            .copied()
            .ok_or(emucore::mem::MemErrorTypes::IllegalAddress(addr))
    }

    #[inline(always)]
    fn inspect_word(&self, addr: usize) -> emucore::mem::MemResult<u16> {
        let i = self
            .idx2(addr)
            .ok_or(emucore::mem::MemErrorTypes::IllegalAddress(addr))?;
        Ok(u16::from_be_bytes([self.data[i], self.data[i + 1]]))
    }

    fn upload(&mut self, addr: usize, _data: &[u8]) -> emucore::mem::MemResult<()> {
        Err(emucore::mem::MemErrorTypes::IllegalWrite(addr))
    }

    fn get_range(&self) -> std::ops::Range<usize> {
        self.base..(self.base + self.data.len())
    }

    fn update_sha1(&self, _digest: &mut emucore::sha1::Sha1) {}

    fn load_byte(&mut self, addr: usize) -> emucore::mem::MemResult<u8> {
        self.inspect_byte(addr)
    }

    fn store_byte(&mut self, addr: usize, _val: u8) -> emucore::mem::MemResult<()> {
        Err(emucore::mem::MemErrorTypes::IllegalWrite(addr))
    }

    fn store_word(&mut self, addr: usize, _val: u16) -> emucore::mem::MemResult<()> {
        Err(emucore::mem::MemErrorTypes::IllegalWrite(addr))
    }

    #[inline(always)]
    fn load_word(&mut self, addr: usize) -> emucore::mem::MemResult<u16> {
        self.inspect_word(addr)
    }
}

/// Disassembler. `labels` is an optional sorted (address, name) list used to
/// resolve branch targets and d16(PC) operands.
#[derive(Debug, Clone, Default)]
pub struct Diss {
    pub labels: Vec<(u32, String)>,
}

impl Diss {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_labels(labels: Vec<(u32, String)>) -> Self {
        let mut l = labels;
        l.sort_by_key(|(a, _)| *a);
        Self { labels: l }
    }

    /// label name at `addr`, if any
    pub fn label_at(&self, addr: u32) -> Option<&str> {
        self.labels
            .binary_search_by_key(&addr, |(a, _)| *a)
            .ok()
            .map(|i| self.labels[i].1.as_str())
    }

    pub fn diss<M: MemoryIO>(&self, mem: &mut M, addr: usize) -> Result<Disassembly, DecodeError> {
        let decoded = decode(mem, addr)?;
        let text = self.render(&decoded);
        Ok(Disassembly {
            addr,
            size: decoded.size,
            text,
            decoded,
        })
    }

    /// render one already-decoded instruction
    pub fn render_line(&self, d: &DecodedInsn) -> String {
        self.render(d)
    }

    fn render(&self, d: &DecodedInsn) -> String {
        let i = &d.insn;
        let sz = i.size.suffix();
        let mn = i.mnemonic;
        let ea = |d: &DecodedInsn| self.ea_str(d, d.ea.as_ref().unwrap());
        match i.form {
            Form::EaReg => fmt(mn, sz, &[ea(d), format!("D{}", d.reg1)]),
            Form::RegEa => fmt(mn, sz, &[format!("D{}", d.reg1), ea(d)]),
            Form::EaAddr => fmt(mn, sz, &[ea(d), format!("A{}", d.reg1)]),
            Form::RegReg => fmt(
                mn,
                sz,
                &[
                    format!("{}{}", cls(i.cls2), d.reg2),
                    format!("{}{}", cls(i.cls1), d.reg1),
                ],
            ),
            Form::Exg => fmt(
                mn,
                "",
                &[
                    format!("{}{}", cls(i.cls1), d.reg1),
                    format!("{}{}", cls(i.cls2), d.reg2),
                ],
            ),
            Form::PredecPredec => fmt(
                mn,
                sz,
                &[format!("-(A{})", d.reg2), format!("-(A{})", d.reg1)],
            ),
            Form::PostincPostinc => fmt(
                mn,
                sz,
                &[format!("(A{})+", d.reg2), format!("(A{})+", d.reg1)],
            ),
            Form::ImmEa | Form::BitImmEa => fmt(mn, sz, &[self.imm_str(d, i.size), ea(d)]),
            Form::ImmOnly => fmt(mn, "", &[self.imm_str(d, i.size)]),
            Form::Move => fmt(mn, sz, &[ea(d), self.ea_str(d, d.ea_dst.as_ref().unwrap())]),
            Form::Moveq => fmt(
                "moveq",
                "",
                &[
                    format!("#{}", (d.word & 0xFF) as u8 as i8),
                    format!("D{}", d.reg1),
                ],
            ),
            Form::Addq => fmt(mn, sz, &[format!("#{}", qdata(d.reg1)), ea(d)]),
            Form::ShiftImm => fmt(
                mn,
                sz,
                &[format!("#{}", qdata(d.reg1)), format!("D{}", d.reg2)],
            ),
            Form::ShiftReg => fmt(mn, sz, &[format!("D{}", d.reg1), format!("D{}", d.reg2)]),
            Form::ShiftEa => fmt(mn, "w", &[ea(d)]),
            Form::Bcc8 | Form::Bcc16 => {
                let target = d.label.unwrap_or(0);
                let name = self.label_at(target);
                let suffix = if i.form == Form::Bcc16 { ".w" } else { "" };
                match name {
                    Some(n) => format!("{mn}{suffix} {n}"),
                    None => format!("{mn}{suffix} ${target:04X}"),
                }
            }
            Form::Dbcc => {
                let target = d.label.unwrap_or(0);
                let disp = match self.label_at(target) {
                    Some(n) => n.to_string(),
                    None => format!("${target:04X}"),
                };
                format!("{mn} D{}, {disp}", d.reg2)
            }
            Form::Scc => fmt(mn, "", &[ea(d)]),
            Form::MovemRe | Form::MovemEr | Form::MovemPd => {
                let list = self.reglist_str(d);
                let ea = ea(d);
                match i.form {
                    Form::MovemEr => fmt(mn, sz, &[ea, list]),
                    _ => fmt(mn, sz, &[list, ea]),
                }
            }
            Form::Link => fmt(mn, "", &[format!("A{}", d.reg2), self.imm_str(d, Size::W)]),
            Form::Movep => {
                let disp = format!("${:04X}", d.imm.unwrap_or(0) & 0xFFFF);
                let er = i.src == "M_AIND"; // d16(An),Dn direction
                if er {
                    fmt(
                        mn,
                        sz,
                        &[format!("{}(A{})", disp, d.reg2), format!("D{}", d.reg1)],
                    )
                } else {
                    fmt(
                        mn,
                        sz,
                        &[format!("D{}", d.reg1), format!("{}(A{})", disp, d.reg2)],
                    )
                }
            }
            Form::Trap => fmt(mn, "", &[format!("#{}", d.word & 0xF)]),
            Form::Ea => {
                let t = ea(d);
                if sz.is_empty() {
                    format!("{mn} {t}")
                } else {
                    format!("{mn}.{sz} {t}")
                }
            }
            Form::Dreg => fmt(mn, sz, &[format!("D{}", d.reg2)]),
            Form::Areg => fmt(mn, "", &[format!("A{}", d.reg2)]),
            Form::Imm16 => fmt(mn, "", &[self.imm_str(d, Size::W)]),
            Form::None => {
                // dc.w pseudo-ops / unknown words show the raw word;
                // plain mnemonics (nop/rts/...) render bare
                if mn == "dc.w" {
                    format!("dc.w ${:04X}", d.word)
                } else {
                    mn.to_string()
                }
            }
        }
    }

    /// render one EA operand
    fn ea_str(&self, d: &DecodedInsn, e: &Ea) -> String {
        let size = d.insn.size;
        match e.mode {
            0 => format!("D{}", e.reg),
            1 => format!("A{}", e.reg),
            2 => format!("(A{})", e.reg),
            3 => format!("(A{})+", e.reg),
            4 => format!("-(A{})", e.reg),
            5 => format!("{}(A{})", signed_hex(e.ext[0] as i16), e.reg),
            6 => {
                let w = e.ext[0];
                let long = w & 0x8000 != 0;
                let scale = scale_str(w);
                let idx = if w & 0x0080 != 0 { 'A' } else { 'D' };
                let d8 = (w & 0x7F) as u8 as i8;
                format!(
                    "{}(A{},{}{}.{}{})",
                    signed_hex(d8 as i16),
                    e.reg,
                    idx,
                    (w >> 8) & 7,
                    if long { "l" } else { "w" },
                    scale
                )
            }
            7 => match e.reg {
                0 => format!("${:04X}.w", e.ext[0]),
                1 => format!("${:04X}{:04X}.l", e.ext[0], e.ext[1]),
                2 => {
                    let target = e.ref_pc.unwrap_or(0) as i64 + e.ext[0] as i16 as i64;
                    match self.label_at(target as u32) {
                        Some(n) => n.to_string(),
                        None => format!("${:04X}(PC)", e.ext[0] as u16),
                    }
                }
                3 => {
                    let w = e.ext[0];
                    let long = w & 0x8000 != 0;
                    let scale = scale_str(w);
                    let idx = if w & 0x0080 != 0 { 'A' } else { 'D' };
                    let d8 = (w & 0x7F) as u8 as i8;
                    format!(
                        "{}(PC,{}{}.{}{})",
                        signed_hex(d8 as i16),
                        idx,
                        (w >> 8) & 7,
                        if long { "l" } else { "w" },
                        scale
                    )
                }
                4 => match size {
                    Size::B => format!("#${:02X}", (e.ext[0] & 0xFF) as u8),
                    Size::L => format!("#${:08X}", ((e.ext[0] as u32) << 16) | e.ext[1] as u32),
                    _ => format!("#${:04X}", e.ext[0]),
                },
                _ => format!("(??{})", e.reg),
            },
            _ => format!("(??{})", e.reg),
        }
    }

    /// immediate rendering (width by operation size)
    fn imm_str(&self, d: &DecodedInsn, size: Size) -> String {
        let v = d.imm.unwrap_or(0);
        match size {
            Size::B => format!("#${:02X}", v & 0xFF),
            Size::L => format!("#${:08X}", v),
            _ => format!("#${:04X}", v & 0xFFFF),
        }
    }

    /// movem register list (mask bits: D0..D7 then A0..A7)
    fn reglist_str(&self, d: &DecodedInsn) -> String {
        let mask = d.regmask.unwrap_or(0);
        let predec = d.insn.form == Form::MovemPd;
        let mut parts: Vec<String> = Vec::new();
        // Scan register bits in the given direction; runs keep ascending
        // names (MAME convention). For predecrement the mask is bit-reversed
        // (bit 15 = D0 .. bit 0 = A7, MAME's movem_pd handlers).
        let emit_run = |from: usize,
                        to: usize,
                        name_of: fn(usize) -> usize,
                        prefix: char,
                        parts: &mut Vec<String>| {
            let desc = from > to;
            let mut i = from;
            loop {
                if mask & (1 << i) != 0 {
                    let mut j = i;
                    loop {
                        let next = if desc { j.checked_sub(1) } else { Some(j + 1) };
                        let past = match next {
                            Some(n) => {
                                if desc {
                                    n < to
                                } else {
                                    n > to
                                }
                            }
                            None => true,
                        };
                        let n = match next {
                            Some(n) => n,
                            None => break,
                        };
                        if past || mask & (1 << n) == 0 {
                            break;
                        }
                        j = n;
                    }
                    // runs keep ascending register names (MAME convention):
                    // high mask bit -> low register number for predecrement
                    let a = name_of(i);
                    let b = name_of(j);
                    parts.push(if a == b {
                        format!("{prefix}{a}")
                    } else {
                        format!("{prefix}{a}-{prefix}{b}")
                    });
                    i = j;
                }
                if i == to {
                    break;
                }
                if desc {
                    i -= 1;
                } else {
                    i += 1;
                }
            }
        };
        fn d_map(b: usize) -> usize {
            b
        }
        fn a_map(b: usize) -> usize {
            b - 8
        }
        if predec {
            // store order (bit-reversed mask): D7..D0 then A7..A0
            emit_run(15, 8, |b| 15 - b, 'D', &mut parts);
            emit_run(7, 0, |b| 7 - b, 'A', &mut parts);
        } else {
            emit_run(0, 7, d_map, 'D', &mut parts);
            emit_run(8, 15, a_map, 'A', &mut parts);
        }
        parts.join("/")
    }
}

fn cls(c: u8) -> char {
    if c == 1 {
        'A'
    } else {
        'D'
    }
}

/// quick-immediate value: 0 encodes 8
fn qdata(v: u8) -> u8 {
    if v == 0 {
        8
    } else {
        v
    }
}

/// signed hex rendering: "-$4" for negative, "$4" otherwise
fn signed_hex(v: i16) -> String {
    if v < 0 {
        format!("-${:X}", (-(v as i32)) & 0xFFFF)
    } else {
        format!("${:X}", v)
    }
}

fn scale_str(w: u16) -> &'static str {
    match (w >> 12) & 3 {
        0 => "",
        1 => "*2",
        2 => "*4",
        _ => "*8",
    }
}

/// "{mn}.{sz} op1,op2" — size suffix omitted when empty
fn fmt(mn: &str, sz: &str, ops: &[String]) -> String {
    let head = if sz.is_empty() {
        mn.to_string()
    } else {
        format!("{mn}.{sz}")
    };
    if ops.is_empty() {
        head
    } else {
        format!("{head} {}", ops.join(","))
    }
}
