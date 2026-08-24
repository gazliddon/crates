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
  against the real Tempest 2000 DSP program (`tests/data/dsp.bin`):
  382 instructions decode, entry sequence and chip-disambiguation
  asserted in `tests/diss_dsp.rs`.
- ✅ **Execution** (`cpu::alu` + `Cpu::step`): full instruction semantics
  ported from MAME's jaguar core — quirks included (N = bit 29,
  `convert_zero` quick immediates, `shlq` 32-raw, `ror` carry from bit
  30, internal-RAM long accesses for loadb/storeb, branch **delay
  slots**, +3 wait states on taken branches, the `addc` carry formula,
  `abs` of 0x80000000). `tests/exec_dsp.rs` covers 12 semantics tests
  plus a smoke run of the real T2K DSP program (trampoline entry, stack
  pointer, TEST.SYM constants).

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
