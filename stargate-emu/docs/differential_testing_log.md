# Differential Testing Log

Append-only log of MAME-vs-local differential runs and the fixes they
drove. Never delete old entries. See `emulation_spec.md` for the mandate
and `williams_hw/` for hardware reference.

## 2026-08-23 — First MAME differential: CMOS 4-bit masking

### Evaluated

- Full 6809 execution trace of Stargate boot: MAME 0.289 (`-debug`
  + `trace`, 60 emulated seconds, 14,416,465 instructions) vs
  `stargate-emu` PC-per-instruction trace.
- Setup: MAME rompath at `/tmp/mame_roms/stargate/` (symlinks to
  `../stargate/roms/01..12`, `sound.bin`, `decoder.4/5`); romset verified
  "good" against MAME's expected checksums.

### Result

- PC streams matched **perfectly for 2,697,819 instructions** (~10 s).
- **Divergence at instruction 2,697,820** in the CMOS audit-validation
  routine (`$E6EF` `bsr $E6F6`; `$E6F1` `beq $E6EE`):
    - MAME: takes the branch (checksum comparison equal) -> `$E6EE` rts.
    - Local: falls through -> `$E6F3` `jmp $E039` (re-init path).

### Root cause

`StargateBus` stored CMOS writes raw; MAME's `cmos_w`
(`src/mame/midway/williams_m.cpp`) forces the upper nibble:
`m_nvram[offset] = data | 0xf0` (battery-backed 4-bit RAM; only 4 bits
valid). The game's read-modify-write cycles on CMOS depend on the
forced `0xF0` upper nibble, which feeds back into the audit checksum
computed over `$CC36-$CC9E` and compared with the stored value at
`$CCA1`.

### Fix

`src/bus.rs` `store_byte` for `0xcc00..=0xcfff`:
`self.cmos[addr - 0xcc00] = value` ->
`self.cmos[addr - 0xcc00] = value | 0xf0` (with comment).

### Verification status

- **FAIL**: the PC streams still diverge at instruction 2,697,820 after
  the fix — the masking alone did not resolve the checksum mismatch.
  Open questions: MAME nvram persistence location, exact CMOS content at
  the divergence, and whether the game writes CMOS before the first
  validation.
- Docs updated: `williams_hw/base.md` CMOS section now notes the
  read-modify-write gotcha (was already documented as "upper nibble
  reads 1"; the code just didn't implement it).

## 2026-08-23 — Second differential: nvram provenance, 6821 IRQ model, 6809 core fixes

### Evaluated

- Fresh MAME 0.289 runs with controlled `-nvram_directory` states, both with and
  without a pre-populated nvram file, plus a PC-per-instruction comparison of the
  pass-path boot against `stargate-emu`.

### Findings

1. **The 12:18 "pass" trace ran with a populated nvram.** MAME with a truly empty
   nvram takes the same `$E6F1` fail branch as our emulator and spins in the
   `$E6B5` Advance-ack poll (86,978 iterations in 13 s). The saved nvram from a
   12 s run (`/tmp/mnv/stargate/nvram`) validates on the next boot
   (`$E6F1` -> `$E6EE` -> main loop), so the CMOS checksums the game stores
   during its re-init are valid. Our empty-CMOS fail path matches MAME exactly.
2. **CMOS checksum formula corrected**: the sum is the half-open interval
   `[$CC36, $CC9E)` (104 bytes), `+ $37`, low nibbles only; the stored value is
   packed across `$CCA0` (high nibble) / `$CCA1` (low nibble) as
   `(cmos[CCA0]<<4)|(cmos[CCA1]&0x0F)`. The `|0xF0` write mask makes the
   read-modify-write round-trip work.
3. **Interrupt vectors corrected**: `$FFF8/$FFF9` (IRQ) = `$9C6B` (the beam
   handler); every other vector = `$F486` (re-init). The game *does* take IRQs
   (clears I with `ANDCC #$00`), contrary to the earlier "polls only" note.
4. **6821 model implemented** in `StargateBus`: MAME register order (port A data
   at `$C80C`, control A at `$C80D`, port B data at `$C80E`, control B at
   `$C80F`), CA1 = `scanline >= 240`, CB1 = `BIT(scanline,5)` (suppressed at
   line 256), flags latch on the active edge (control bit 1), IRQ output is
   level-based (flag && control bit 0), data-port reads clear the flags.
5. **IRQ delivery must be level-based**: latching `pending_irq` with `|=` caused
   a stale re-fire after the handler cleared the flag and RTI'd (double IRQs
   every edge). `pending_irq = interrupts.irq` (the live level) matches MAME.
6. **emu6809 core bugs found and fixed** (all driven by the pass-path
   divergence):
   - Postbyte `0x9F` (`[abs]`, indirect) was decoded as 2 bytes with no
     indirection; it is a 4-byte instruction whose effective address is the
     16-bit pointer stored at the operand.
   - Cycle counts were wrong for: 16-bit stores indexed/extended (5/6 -> 6/7),
     `JMP` extended (4 -> 3), `JSR` extended (8 -> 7), `CMPX/Y/U/S/D` immediate
     (4/5 -> 5/6), `LDY` immediate (5 -> 4), direct-page memory ops (6 -> 5),
     `TST` (6/7 -> 4/5/6), `ORCC`/`ANDCC` (3 -> 2), long branches (5 -> 6),
     `SYNC` (2 -> 4), and indexed effective-address cycles were not added at
     all (now `,R+`/`,-R` +1, `,R++`/`,--R` +2, 8-bit/PC8 offsets +1,
     16-bit/PC16/D,R +2, `[abs]` +4).
   - The undocumented opcode `0x01` (NEG direct, aliasing `0x00`) was missing;
     Stargate jumps into the operand byte of `JSR $0115` at `$0120` (and
     `$0136`) and executes it as `NEG <$15` (self-modifying display-list code).
     MAME's m6809 implements it; the ISA table now allows duplicate
     (action, mode) pairs so both opcodes get table arms.
7. **Result**: with a valid nvram the pass path now runs the main loop and
   reaches the attract drawing phase (`$2AC0` sprite plotter first executed at
   instruction 14,853,599; MAME's at 11,987,278 - the residual gap is a small
   beam-phase drift from the remaining cycle-model differences). The empty-nvram
   fail path is unchanged and matches MAME.

### Open

- The remaining beam-phase drift (~0.03% cycle totals) shifts the IRQ handler's
  beam-derived values (`$7E`) and delays the attract entry; hunting the last
  cycle-count deltas against the m6809 core is the next step.
- Palette writes: neither MAME (60 s) nor ours (40 M instr) has written the
  colour registers yet - the attract colour init happens later in both.
