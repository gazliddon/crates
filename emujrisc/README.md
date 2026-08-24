# emujrisc

Atari Jaguar **GPU/DSP (JRISC)** emulator, in development.

Tom (GPU) and Jerry (DSP) share the JRISC core; [`Chip`] carries the
variant differences: instruction legality (opcodes 32/33/42/48/63 mean
different instructions per chip), default register bank (GPU=1, DSP=0),
RAM size/base and the peripheral register block base.

## Status

- ✅ ISA database (`resources/opcodes_jrisc.json`, generated table via
  `build.rs`) — from vasm's jagrisc backend, per the "Jaguar Technical
  Reference Manual for Tom & Jerry" Rev 8.
- ✅ Decoder + disassembler (`cpu::decode`, `diss::Diss`), validated
  against the real Tempest 2000 DSP program (`tests/data/dsp.bin`,
  extracted from `MOOMOO.DAT` and cross-checked against `TEST.TXT`'s
  `GPUSTART` blob): 382 instructions decode, entry sequence and
  chip-disambiguation asserted in `tests/diss_dsp.rs`.
- ⏳ Execution (ALU/cycles/peripherals) — skeleton in `cpu::alu` /
  `cpu::cpucore`.

## Encoding

```
word = (opcode6 << 10) | (src5 << 5) | dst5
movei (op 38): + 2 words, 32-bit immediate word-swapped (lo word first)
nop = 0xE400; jump = op 52 (reg in src, cc in dst); jr = op 53
```

## Usage

```rust
use emujrisc::cpu::Chip;
use emujrisc::diss::{Diss, DissCtx};

let ctx = DissCtx::from_slice(0xF1B000, "dsp", dsp_bytes);
let diss = Diss::new(Chip::Dsp);
let d = ctx.diss(&diss, 0xF1B000)?;   // "movei #$00F1B022,r0"
```

## Sources

- vasm `cpus/jagrisc` backend (opcodes.h, cpu.c)
- "Jaguar Technical Reference Manual for Tom & Jerry", Rev 8
- Tempest 2000 reference tree (`MOOMOO.DAT`, `TEST.TXT`/`TEST.SYM`)
