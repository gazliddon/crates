//! emujrisc — Atari Jaguar GPU/DSP (JRISC) emulator.
//!
//! Tom (GPU) and Jerry (DSP) share the JRISC core; the differences live in
//! [`cpu::Chip`]: instruction legality per variant (some opcode numbers mean
//! different instructions on GPU vs DSP), register bank, RAM size/base and
//! peripheral registers.
//!
//! Status: ISA decode + disassembly are implemented and validated against the
//! Tempest 2000 DSP program (`tests/data/dsp.bin`, extracted from
//! `MOOMOO.DAT`/`TEST.TXT` — 868 bytes, decodes to 382 instructions).
//! Execution (ALU/cycles) is a skeleton.
#![allow(dead_code)]
pub mod cpu;
pub mod diss;
pub mod mem;
pub mod isa;
pub use byteorder;
pub use emucore;
