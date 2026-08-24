//! emujrisc — Atari Jaguar GPU/DSP (JRISC) emulator.
//!
//! Tom (GPU) and Jerry (DSP) share the JRISC core; the differences live in
//! [`cpu::Chip`]: instruction legality per variant (some opcode numbers mean
//! different instructions on GPU vs DSP), register bank, RAM size/base and
//! peripheral registers.
//!
//! Status: ISA decode, disassembly **and instruction execution** are
//! implemented. The decoder/disassembler are validated against the Tempest
//! 2000 DSP program (`tests/data/dsp.bin` — 868 bytes, 382 instructions);
//! execution semantics are ported from MAME's jaguar core (quirks included:
//! N = bit 29, convert_zero immediates, shlq 32-raw, internal-RAM long
//! accesses, branch delay slots, +3 wait states on taken branches).
#![allow(dead_code)]
pub mod cpu;
pub mod diss;
pub mod mem;
pub mod isa;
pub use byteorder;
pub use emucore;
