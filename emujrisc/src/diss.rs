//! Disassembler.
//!
//! Mirrors the emu6809 pattern: a [`Diss`] holds the chip variant, a
//! [`DissCtx`] wraps the memory image. Rendering follows the assembly
//! syntax of the `.GAS` sources (e.g. `movei #$00F1B022,r0`,
//! `jump t,(r0)`, `load (r15+3),r3`, `store r29,(r9)`).

use crate::cpu::{cc_name, decode, Chip, DecodeError, DecodedInsn};
use crate::isa::OperandClass;
use crate::mem::SliceMem;
use emucore::mem::MemoryIO;

#[derive(Debug, Clone)]
pub struct Disassembly {
    pub addr: usize,
    pub size: usize,
    pub text: String,
    pub decoded: DecodedInsn,
}

/// A memory image to disassemble from (32-bit addressable).
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

/// Disassembler for one chip variant.
#[derive(Debug, Clone, Copy, Default)]
pub struct Diss {
    pub chip: Chip,
}

impl Diss {
    pub fn new(chip: Chip) -> Self {
        Self { chip }
    }

    pub fn diss<M: MemoryIO>(&self, mem: &mut M, addr: usize) -> Result<Disassembly, DecodeError> {
        let decoded = decode(mem, addr, self.chip)?;
        let text = self.render(&decoded);
        Ok(Disassembly {
            addr,
            size: decoded.size,
            text,
            decoded,
        })
    }

    fn render(&self, d: &DecodedInsn) -> String {
        let mn = d.insn.mnemonic;
        // Special cases with explicit operand order:
        match mn {
            "movei" => {
                return format!("movei #${:08X},r{}", d.extra.unwrap_or(0), d.dst);
            }
            "jump" => {
                return format!("jump {},(r{})", cc_name(d.dst), d.src);
            }
            "jr" => {
                // offset is a 5-bit signed word count (MAME:
                // (s8)(op>>2)>>3), sign-extended here from bit 4
                let off = (((d.src as i8) << 3) >> 3) as i32 * 2;
                let target = d.addr as i32 + 2 + off;
                return format!("jr {},${target:04X}", cc_name(d.dst));
            }
            _ => {}
        }
        let src_part = self.render_class(d.insn.src, d.src, d);
        let dst_part = self.render_class(d.insn.dst, d.dst, d);
        // store family (opswap): encoded src = address, dst = data;
        // display "store rD,(rA)" — data first, address second.
        if mn.starts_with("store") || mn == "mirror" {
            return match (dst_part.as_str(), src_part.as_str()) {
                ("", "") => mn.to_string(),
                ("", s) => format!("{mn} {s}"),
                (dd, s) => format!("{mn} {dd},{s}"),
            };
        }
        match (src_part.as_str(), dst_part.as_str()) {
            ("", "") => mn.to_string(),
            ("", dd) => format!("{mn} {dd}"),
            (s, "") => format!("{mn} {s}"),
            (s, dd) => format!("{mn} {s},{dd}"),
        }
    }

    /// Render one operand field: `val` is the raw 5-bit field value for this
    /// operand's class (src field for the src operand, dst for the dst).
    fn render_class(&self, class: OperandClass, val: u8, _d: &DecodedInsn) -> String {
        use OperandClass::*;
        match class {
            Reg => format!("r{val}"),
            IReg => format!("(r{val})"),
            Imm0 | Imm1 | Imm1s => format!("#{val}"),
            SImm => format!("#{}", val as i8 as i32),
            ImmLw => String::new(), // handled by the movei special case
            Ir14d => format!("(r14+{val})"),
            Ir15d => format!("(r15+{val})"),
            Ir14r => format!("(r14+r{val})"),
            Ir15r => format!("(r15+r{val})"),
            Cc => cc_name(val).to_string(),
            Rel => String::new(), // handled by the jr special case
            Pc => "pc".to_string(),
            None => String::new(),
        }
    }
}
