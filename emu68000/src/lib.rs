//! emu68000 — Motorola 68000 emulator (in development).
//!
//! Status: the instruction database and decoder are complete and validated
//! against MAME's Musashi-derived 68000 disassembler (identical behaviour
//! over all 65536 opcode words; see `tools/gen68000.py` and
//! `resources/opcodes68000.json`). The disassembler is validated against the
//! Imagitec Tempest 2000 sound module (`tests/data/TEST.TXT`, loaded at
//! `$4000`) with its ALN symbol table (`TEST.SYM`) — every code symbol lands
//! on an instruction boundary.
//!
//! Next milestones: the execution core (addressing-mode fetches, ALU
//! semantics, SR/flags) and a MAME trace-diff harness, mirroring how
//! `emujrisc` was built.

#![allow(dead_code)]
pub mod cpu;
pub mod diss;
pub mod isa;
