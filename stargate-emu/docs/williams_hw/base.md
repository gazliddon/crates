# Williams Base Platform (1980-82)

The Williams arcade hardware family — Defender (1980), Stargate (1981),
Robotron: 2084 (1982), plus Joust, Bubbles, Sinistar, Splat and others —
shares a common two-board platform built around the Motorola 6809. MAME
models it in `src/mame/midway/williams.cpp`; the shared machine
configuration is `williams_base`, with board revisions `williams_b0`,
`williams_b1` and `williams_b2`.

Per-machine details:

- [[defender]]
- [[stargate]]
- [[robotron]]

## Board Revisions

| Rev | MAME configuration | Blitter           | Example machines          |
|-----|--------------------|-------------------|---------------------------|
| B0  | williams_b0        | none              | Defender, Stargate        |
| B1  | williams_b1        | SC1, clip `C000`  | Robotron, Joust, Sinistar |
| B2  | williams_b2        | SC2, clip `C000`  | Splat                     |

B0 boards have no blitter (Defender, Stargate). The blitter appeared on
the B1 board with Joust/Robotron.

## System Clocks

| Signal       | Derivation              | Value       |
|--------------|-------------------------|-------------|
| Master clock | 12 MHz XTAL             | 12 MHz      |
| Main CPU     | MC6809E, `/3/4`         | 1 MHz (E)   |
| Sound CPU    | M6808, `/4` internal    | 894.886 kHz |
| Pixel clock  | master `*2/3`           | 8 MHz       |

The sound crystal is 3.579545 MHz; the M6808's internal divider makes the
effective frequency 894.886 kHz. The 6809E E/Q clock is 1 MHz.

## Video

The Williams video is a **column-major linear framebuffer**, not a tilemap:

- Pixel byte for scanline `y`, column `c` lives at `videoram[c * 256 + y]`
  (256 columns x 256 rows of bytes).
- Each byte holds two pixels: high nibble = even `x`, low nibble = odd `x`.
- Each pixel is a 4-bit index into the 16 colour registers.

Screen: 512 x 260 raster at 8 MHz; the visible area differs per game (see
machine docs). Scanline 256 is skipped for the VA11 interrupt.

### Colour Registers & Palette

- `C000-C00F` (later boards; `C000` on Defender): 16 pen values, format
  `BBGGGRRR` per byte.
- Expansion uses resistor weights: red/green 3 bits each (1200/560/330
  Ohm), blue 2 bits (560/330 Ohm).

### Video Counter

The vertical beam position register returns `vpos & 0xfc` for lines
0-255, otherwise `0xfc`; only 6 bits (2-7) are meaningful. It sits at
`CB00` on later boards and `C800` on Defender.

## Interrupts

| PIA   | Pin    | Signal    | Meaning                          |
|-------|--------|-----------|----------------------------------|
| PIA#1 | CA1    | COUNT240  | high on scanline >= 240          |
| PIA#1 | CB1    | VA11      | scanline bit 5 (every 32 lines)  |
| PIA#1 | IRQA/B | (none)    | -> 6809 IRQ line (merged)        |
| PIA#2 | IRQA/B | (none)    | -> M6808 IRQ line (merged)       |
| PIA#0 | (none) | (none)    | widget PIA IRQs unconnected      |

Two main-board timers drive the PIA lines: a 32-scanline timer toggles
VA11 into CB1, and a scanline-240 timer drives COUNT240 into CA1 (the
"16 ms" IRQ the games use for frame timing).

## PIAs

All boards use MC6821 PIAs. Register layout per PIA:

```text
data_a, ctrl_a, data_b, ctrl_b  (4 bytes, base + 0..3)
```

- **Widget PIA** (PIA#0): player controls (joystick + buttons).
- **ROM PIA** (PIA#1): coin door, tilt, and the sound command output.
- **Sound PIA** (PIA#2, on the sound board): DAC output and sound
  commands.

Bit maps differ per machine — see the machine docs.

## Sound System

### Command Path

1. CPU writes ROM PIA port B.
2. `snd_cmd_w` ORs `0xC0` (high two bits strapped high on the board) and
   forwards the byte to the sound board PIA (PIA#2) port B.
3. PIA#2 CB1 is strobed as the handshake: CB1 = 0 when the command byte is
   `0xFF`, else 1.

### Sound Board

| Addr      | Type                 |
|-----------|----------------------|
| 0000-007F | M6808 internal RAM   |
| 0080-00FF | MC6810 RAM (later)   |
| 0400-0403 | PIA #2 (mirror 8000) |
| B000-FFFF | ROM                  |

Defender's sound board omits the MC6810 (internal RAM only). Output: PIA#2
port A drives an MC1408 8-bit DAC into the speaker.

## CMOS NVRAM

- `CC00-CFFF` on later boards (`C400-C4FF` on Defender), 1K x 4, battery
  backed (5101 on Defender; 5114/6514 on later boards).
- Only 4 data bits are valid: writes are stored as `data | 0xF0`, so the
  upper nibble always reads as ones.

## Watchdog

The watchdog register resets **only** when the data byte is exactly
`0x39` (`watchdog_reset_w` checks the value). It sits at `CBFF` on later
boards and `C3FF` on Defender.

## Blitter (B1/B2 only)

Registers at `CA00-CA07` (later boards):

| Addr | Register       |
|------|----------------|
| CA00 | start / control|
| CA01 | mask / remap   |
| CA02 | source hi      |
| CA03 | source lo      |
| CA04 | dest hi        |
| CA05 | dest lo        |
| CA06 | width          |
| CA07 | height         |

Control byte bits (CA00): bit 7 = skip high nibble, bit 6 = skip low
nibble, bit 5 = shift right one pixel, bit 4 = remap, bit 3 = source mode,
bit 1 = transparent. Width/height are XORed with 4 (except Splat). Writing
CA00 triggers the blit; the CPU is charged the estimated blit time in
cycles.

## ROM Layout Conventions

- **Main CPU region** is `0x19000` (100K). ROMs load at `0x0D000` (hi
  ROM, CPU `D000-FFFF`) and `0x10000`+ (banked low ROM).
- **Sound CPU region** is `0x10000`; one ROM at `0xF000` or `0xF800`.
- **PROM region** is `0x0400`; two 0x200 decoder ROMs (horizontal +
  vertical video decoders).

## Sources & References

- MAME `src/mame/midway/williams.cpp` — drivers, memory maps, inputs
- MAME `src/mame/midway/williams_m.cpp` — banking, CMOS, watchdog, IRQs
- MAME `src/mame/midway/williams_v.cpp` — video update, palette, blitter
- MC6821 datasheet: https://sowerbutts.com/replica1-serial/6821.pdf
- [Sean Riddle: Williams Hardware](http://seanriddle.com/willhard.html)
