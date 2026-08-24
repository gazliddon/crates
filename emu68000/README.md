# emu68000

In-development Motorola 68000 emulator, built for the Tempest 2000 (Atari
Jaguar) decompilation project: the Jaguar's main CPU is a 68000 running the
game code, and the Imagitec sound module (TEST.TXT / TEST.DB) is 68000 code.

## Status

- **Instruction database** — `resources/opcodes68000.json` (282 entries)
  mirrors MAME's Musashi-derived 68000 disassembler opcode table (base-68000
  subset), including each entry's effective-address validity mask. Verified
  behaviorally identical to MAME over all 65536 opcode words.
- **Decoder** — EA decode, extension-word lengths, immediates, branch
  targets, movem register masks; MAME's mask-specificity search order and
  EA-validity gating.
- **Disassembler** — ALN-style rendering with optional symbol-table label
  resolution. Validated against the Imagitec TEST.TXT module (loaded at
  `$4000`) with TEST.SYM: every code symbol lands on an instruction
  boundary.
- **Not yet** — the execution core (addressing-mode reads/writes, ALU, SR
  flags) and a MAME trace-diff harness.

## Layout

```
resources/opcodes68000.json   instruction table (generated)
build.rs                      builds the static ISA table from the JSON
src/isa/db.rs                 table types + JSON reader (build-safe)
src/isa/mod.rs                generated-table include, Dbase singleton
src/cpu/decoder.rs            M68000 decoder (MAME-faithful)
src/cpu/registers.rs          base-68000 programmer's model
src/diss.rs                   disassembler + SliceMem view
tests/diss_module.rs          TEST.TXT / TEST.SYM validation
```

The table generator lives in the T2K repo: `t2kdecomp/tools/gen68000.py`.
