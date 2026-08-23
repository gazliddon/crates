# Defender (1980)

The first Williams 6809 game (Williams, 1980). MAME machine configuration:
`defender` = `williams_b0` plus a banked `C000` window and a different
visible area. The CPU and sound clocks are identical to the [[base
platform|Williams Base Platform]].

## Memory Map

Defender is the odd one out in the family: **no low-ROM banking**. The
entire 0000-BFFF is always video RAM, and code lives in the high ROMs plus
a banked I/O/ROM window at C000-CFFF.

| Addr      | Type          | Notes                               |
|-----------|---------------|-------------------------------------|
| 0000-BFFF | Video RAM     | 48K, always RAM, no banking         |
| C000-CFFF | Banked window | Bank 0 = I/O, banks 1-15 = ROM      |
| D000-DFFF | Bank select   | write selects window bank (0-15)    |
| D000-FFFF | ROM           | 24K direct ROM                      |

The C000 window is selected by writing the bank number to `D000`
(`bank_select_w`, `data & 0x0f`). Bank 0 exposes the I/O registers; higher
banks page in the remaining program ROM (region offset `0x10000`+).

### I/O Registers (window bank 0)

| Addr      | Register      | Notes                                |
|-----------|---------------|--------------------------------------|
| C000-C00F | Colour regs   | 16 bytes of `BBGGGRRR`               |
| C010-C01F | Video control | bit 0 = cocktail flip                |
| C3FF      | Watchdog      | write `$39` to reset                 |
| C400-C4FF | CMOS          | 1K x 4, battery backed               |
| C800-CBFF | Video count   | vertical beam position (read)        |
| CC00-CC03 | PIA #1        | ROM PIA                              |
| CC04-CC07 | PIA #0        | Widget PIA                           |

Note the PIA order on Defender: the ROM PIA is at the lower address
(`CC00`), widget PIA above it (`CC04`) — the reverse of the later boards'
layout at C804/C80C.

## Inputs

2-way joystick (up/down only) plus buttons. Widget PIA port A:

| Bit | Signal      |
|-----|-------------|
| 0   | Fire        |
| 1   | Thrust      |
| 2   | Smart Bomb  |
| 3   | Hyperspace  |
| 4   | 2 Players   |
| 5   | 1 Player    |
| 6   | Reverse     |
| 7   | Down (2-way)|

Port B: bit 0 = Up (2-way); bits 1-7 unused. ROM PIA port A (IN2) matches
the family standard: Auto Up, Advance, coins, High Score Reset, Tilt.

## Video

Visible area: 12-303 x 7-246 (set wider than the later games). Cocktail
table inversion is controlled by video control bit 0 at `C010`.

## Sound

Defender's sound board is the original: M6808 with **internal RAM only**
(no MC6810 at 0080-00FF). Sound ROM: `video_sound_rom_1.ic12` at `$F800`,
0x800 bytes, CRC `fefd5b48`.

## PROMs

- Early PCBs: `decoder.2` (horizontal) + `decoder.3` (vertical), 0x200
  each.
- Green-label set: single `decoder.1` (identical code to decoder 2).
- White was the first ROMset; green/blue only differ in the decoder chips
  used; red is the final version (only red runs in cocktail tables).

## ROM Layout

Main CPU region is `0x19000`. `defend.1`-`defend.4` are direct at
`0x0D000` (CPU D000-FFFF); the rest are banked via the C000 window.

| File      | Region addr | Size  |
|-----------|-------------|-------|
| defend.1  | 0d000       | 0800  |
| defend.4  | 0d800       | 0800  |
| defend.2  | 0e000       | 1000  |
| defend.3  | 0f000       | 1000  |
| defend.9  | 10000       | 0800  |
| defend.12 | 10800       | 0800  |
| defend.8  | 11000       | 0800  |
| defend.11 | 11800       | 0800  |
| defend.7  | 12000       | 0800  |
| defend.10 | 12800       | 0800  |
| defend.6  | 16000       | 0800  |

Sets: red (`defender`), green (`defenderg`), blue (`defenderb`), white
(`defenderw`), Taito license (`defenderj`), plus bootlegs. Some bootlegs
use a 6802 sound CPU (`defender_6802snd`).

## References

- MAME `src/mame/midway/williams.cpp` — `defender` machine, `main_map`,
  `bankc000_map`, `ROM_START(defender)`
- MAME `src/mame/midway/williams_m.cpp` — `bank_select_w`,
  `video_control_w`
- [Arcade History: Defender](https://www.arcade-history.com/?n=defender&page=detail&id=596)
