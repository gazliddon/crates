//! emu68000 — Motorola 68000 emulator (in development).
//!
//! Status: decode and disassembly are validated against MAME's
//! Musashi-derived 68000 disassembler (identical behaviour over all 65536
//! opcode words) and the Imagitec TEST.TXT module (every ALN code symbol
//! lands on an instruction boundary). The executor runs the module's real
//! INITDSP routine end to end: the DSP program is copied verbatim to
//! $F1B000, D_CTRL/D_PC/SCLK/SMODE are programmed, the channel table at
//! $F1B800 is filled, and the routine returns with a balanced stack.
//!
//! Next: MAME trace-diff validation and precise PRM cycle timing.

#![allow(dead_code)]
pub mod cpu;
pub mod diss;
pub mod isa;
