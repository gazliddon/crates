# Robotron: 2084 (1982)

Williams / Vid Kidz, 1982. MAME machine configuration: `williams_b1` —
the shared later-board memory map (same as [[stargate]]) **plus the
blitter**. This page covers the Robotron specifics.

## Memory Map

Identical to Stargate (base `main_map`): banked ROM/video at 0000-8FFF,
I/O at C000-CFFF, high ROM at D000-FFFF. The one addition is the active
blitter at CA00-CA07 (`WILLIAMS_BLITTER_SC1`, clip address `0xC000`).

| Addr      | Type          | Notes                                 |
|-----------|---------------|---------------------------------------|
| 0000-8FFF | ROM / Video   | Banked via `$C900` bit 0              |
| 0000-BFFF | RAM           | Writes always land in video RAM       |
| C000-C00F | Color regs    | 16 bytes of `BBGGGRRR`                |
| C804-C807 | PIA #0        | Widget                                |
| C80C-C80F | PIA #1        | ROM PIA                               |
| C900      | RWCNTL        | VRAM/ROM bank + cocktail flip         |
| CA00-CA07 | Blitter       | SC1, clip `C000`                      |
| CB00      | VERTCT        | vertical beam counter (read)          |
| CBFF      | WDOG          | write `$39` to reset                  |
| CC00-CFFF | CMOS          | 1K x 4 battery-backed                 |
| D000-FFFF | ROM           | ROMs 10-12                            |

## Blitter

Robotron is one of the first games to use the Williams blitter (B1
board). Registers and control byte are described in the
[[blitter|Blitter Programming Guide]]; writes to `CA00` trigger the blit
and consume CPU cycles. The blitter clip address is `0xC000`.

## Inputs

Two 49-way joysticks (move and fire) decoded to digital up/down/left/
right. Widget PIA port A (IN0):

| Bit | Signal             |
|-----|--------------------|
| 0   | Move Up            |
| 1   | Move Down          |
| 2   | Move Left          |
| 3   | Move Right         |
| 4   | 1 Player           |
| 5   | 2 Players          |
| 6   | Fire Up            |
| 7   | Fire Down          |

Widget PIA port B (IN1): bit 0 = Fire Left, bit 1 = Fire Right, bits 2-7
unused. ROM PIA port A (IN2) matches the family standard: Auto Up,
Advance, coins, High Score Reset, Tilt.

## Video

Visible area: 6-297 x 7-246 (base platform default). The blitter draws
sprites into the column-major framebuffer.

## Sound

Sound board is the later type with the MC6810 (same as Stargate). Sound
ROM: `video_sound_rom_3_std_767.ic12` at `$F000`, **0x1000 bytes**, CRC
`c56c1d28` (P/N A-5342-09910) — twice the size of the Defender/Stargate
sound ROMs.

## PROMs

`decoder_rom_4.3g` (horizontal, `$0000`) + `decoder_rom_6.3c` (vertical,
`$0200`), 0x200 each — the newer pair also used by late Stargate boards.

## ROM Layout

Main CPU region is `0x19000`; layout matches Stargate exactly (ROMs 1-9
banked at 0000-8FFF from region offset `0x10000`, ROMs 10-12 direct at
D000-FFFF). The "B" ROMs are labeled 3005-13 through 3005-24.

| File         | Addr   | CRC      |
|--------------|--------|----------|
| 2084_rom_1b  | 10000  | 66c7d3ef |
| 2084_rom_2b  | 11000  | 5bc6c614 |
| 2084_rom_3b  | 12000  | e99a82be |
| 2084_rom_4b  | 13000  | afb1c561 |
| 2084_rom_5b  | 14000  | 62691e77 |
| 2084_rom_6b  | 15000  | bd2c853d |
| 2084_rom_7b  | 16000  | 49ac400c |
| 2084_rom_8b  | 17000  | 3a96e88c |
| 2084_rom_9b  | 18000  | b124367b |
| 2084_rom_10b | 0d000  | 13797024 |
| 2084_rom_11b | 0e000  | 7e3c1b87 |
| 2084_rom_12b | 0f000  | 645d543e |

Sets: Solid Blue (`robotron`), Yellow/Orange (`robotronyo`), Unidesa
license (`robotronun`), plus a 1987 bug-fix patch (`robotron87`) and a
Tie-Die variant (`robotrontd`).

## References

- MAME `src/mame/midway/williams.cpp` — `williams_b1`,
  `ROM_START(robotron)`
- MAME `src/mame/midway/williams_v.cpp` — `blitter_w`, `blitter_core`
- [Arcade History: Robotron: 2084](https://www.arcade-history.com/?n=robotron&page=detail&id=2132)
