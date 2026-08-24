//! Chip selection: Tom (GPU) vs Jerry (DSP).
//!
//! The two cores share the JRISC instruction set; differences:
//! - instruction legality per variant (some opcode numbers — 32, 33, 42, 48,
//!   63 — mean different instructions on each chip),
//! - default register bank (GASM assembles GPU code for bank 1, `-CGPU -R1`,
//!   DSP for bank 0, `-CDSP -R0`; `G_REGPAGE` switches at runtime),
//! - RAM size/base and the peripheral register block base
//!   (same offsets, `$F02100` vs `$F1A100`).

use crate::isa::Variant;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Chip {
    /// Tom's GPU
    #[default]
    Gpu,
    /// Jerry's DSP
    Dsp,
    /// accept both variants (disassembly of unknown binaries); ambiguous
    /// shared-opcode numbers resolve to the first (GPU) entry
    Many,
}

impl Chip {
    /// bitmask (GPU=1, DSP=2) used for instruction-legality filtering
    pub fn flags(self) -> u8 {
        match self {
            Chip::Gpu => 1,
            Chip::Dsp => 2,
            Chip::Many => 3,
        }
    }

    pub fn variant(self) -> Variant {
        match self {
            Chip::Gpu => Variant::Gpu,
            Chip::Dsp => Variant::Dsp,
            Chip::Many => Variant::Any,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Chip::Gpu => "GPU",
            Chip::Dsp => "DSP",
            Chip::Many => "GPU/DSP",
        }
    }

    /// register bank the code is assembled for (GASM `-R1` GPU, `-R0` DSP)
    pub fn default_bank(self) -> u8 {
        match self {
            Chip::Gpu => 1,
            Chip::Dsp | Chip::Many => 0,
        }
    }

    /// internal program RAM (this project's map: GPU 4 KB, DSP 8 KB)
    pub fn ram_base(self) -> u32 {
        match self {
            Chip::Gpu => 0xF03000,
            Chip::Dsp | Chip::Many => 0xF1B000,
        }
    }

    pub fn ram_size(self) -> u32 {
        match self {
            Chip::Gpu => 0x1000,
            Chip::Dsp | Chip::Many => 0x2000,
        }
    }

    /// peripheral register block base (G_CTRL=$F02114, D_CTRL=$F1A114)
    pub fn reg_base(self) -> u32 {
        match self {
            Chip::Gpu => 0xF02100,
            Chip::Dsp | Chip::Many => 0xF1A100,
        }
    }
}
