# Stargate (1981)

Williams / Vid Kidz, 1981. MAME machine configuration: `williams_b0` (no
blitter). Clocks, video, interrupts, and sound protocol are the shared
[[base platform|Williams Base Platform]]; this page covers the Stargate
specifics.

The game assembly lives in `../stargate` (gazm toolchain), which is the
reference source for this machine.

## Memory Map

| Addr      | Type          | Notes                                 |
|-----------|---------------|---------------------------------------|
| 0000-8FFF | ROM / Video   | Banked: entry 0 = video RAM, 1 = ROM  |
| 0000-BFFF | RAM           | Writes always land in video RAM       |
| C000-C00F | Colour regs   | 16 bytes of `BBGGGRRR` (mirrored)     |
| C804-C807 | PIA #0        | Widget (I/O board)                    |
| C80C-C80F | PIA #1        | ROM PIA                               |
| C900-C9FF | RWCNTL        | VRAM/ROM bank select + cocktail flip  |
| CA00-CA07 | Blitter       | Decoded, but Stargate has no blitter  |
| CB00-CBFF | VERTCT        | 6-bit vertical beam counter (read)    |
| CBFF      | WDOG          | Watchdog; write `$39` to reset        |
| CC00-CFFF | CMOS          | 1K x 4 battery-backed CMOS            |
| D000-FFFF | ROM           | ROMs 10-12                            |

### VRAM / ROM Banking

`RWCNTL` (`$C900`) write:

- Bit 0: bank select — `0` = video RAM at 0000-8FFF, `1` = ROM.
- Bit 1: cocktail flip (screen inversion for the table version).

The ROM bank maps in ROM region offset `0x10000` (ROMs 1-9, covering
0000-8FFF in CPU space). Writes to 0000-8FFF always go to video RAM even
while ROM is banked in for reads.

Hardware note: the schematic splits RAM as 38K video RAM (0000-97FF) plus
10K work RAM (9800-BFFF); MAME allocates one 48K `videoram` for
0000-BFFF. Behaviour is identical for emulation.

## I/O Registers

| Addr      | Register            | Notes                                  |
|-----------|---------------------|----------------------------------------|
| C000-C00F | `color_registers`   | 16 bytes of `BBGGGRRR`                 |
| C804      | `widget_pia_dataa`  |                                        |
| C805      | `widget_pia_ctrla`  |                                        |
| C806      | `widget_pia_datab`  |                                        |
| C807      | `widget_pia_ctrlb`  | CB2: `110` = P2, `111` = P1 controls   |
| C80C      | `rom_pia_dataa`     |                                        |
| C80D      | `rom_pia_ctrla`     | CA1 = COUNT240, CA2 = LEDs             |
| C80E      | `rom_pia_datab`     | bits 0-5 = sound command (write)       |
| C80F      | `rom_pia_ctrlb`     | CB1 = VA11, CB2 = LEDs                 |
| C900      | RWCNTL              | VRAM/ROM bank + cocktail flip          |
| CA00      | blitter (unused)    | Stargate has no blitter                |
| CB00      | VERTCT (read)       | `vpos & 0xfc`, or `0xfc` past line 255 |
| CBFF      | WDOG (write)        | Reset only if data == `$39`            |
| CC00      | CMOS                | 4-bit wide; upper nibble reads 1       |

## Inputs

8-way joystick plus buttons. Widget PIA port A:

| Bit | Signal     |
|-----|------------|
| 0   | Fire       |
| 1   | Thrust     |
| 2   | Smart Bomb |
| 3   | HyperSpace |
| 4   | 2 Players  |
| 5   | 1 Player   |
| 6   | Reverse    |
| 7   | Down       |

Port B: bit 0 = Up, bit 1 = Inviso, bits 2-6 unused, bit 7 = 0 Upright /
1 Table. Port B control: CB2 select lines (bits 5-3 of `$C807`) — `110` =
player 2, `111` = player 1.

ROM PIA port A:

| Bit | Signal                             |
|-----|------------------------------------|
| 0   | Auto Up / Manual Down (toggle)     |
| 1   | Advance                            |
| 2   | Right Coin                         |
| 3   | High Score Reset                   |
| 4   | Left Coin                          |
| 5   | Center Coin                        |
| 6   | Slam Door Tilt                     |
| 7   | Sound board handshake (unmodeled)  |

Port B: bits 0-5 = 6-bit sound command (write-only); bits 6-7 strapped
high, so the transmitted byte is `data | 0xC0`.

## Video

Visible area: 6-297 x 7-246 (base platform default). Column-major
framebuffer and palette as per the base platform.

## Sound

Stargate's sound board adds the MC6810 (0080-00FF). Sound ROM:
`video_sound_rom_2_std_744.ic12` at `$F800`, 0x800 bytes, CRC `2fcf6c4d`
(P/N A-5342-09809).

## PROMs

`decoder_rom_4.3g` (universal horizontal decoder, `$0000`) and
`decoder_rom_5.3c` (universal vertical decoder, `$0200`), 0x200 each,
7641-5 BPROMs. Early PCBs used decoders 4+5; newer boards used 4+6.

## ROM Layout

Main CPU region is `0x19000`. ROMs 1-9 load at region offset `0x10000`
(banked in at CPU 0000-8FFF); ROMs 10-12 at `0x0D000` (CPU D000-FFFF).
The "B" ROMs are labeled 3002-13 through 3002-24.

| File   | Addr   | CRC      | SHA-1                                       |
|--------|--------|----------|---------------------------------------------|
| rom_1  | 10000  | 88824d18 | f003a5a9319c4eb8991fa2aae3f10c72d6b8e81a    |
| rom_2  | 11000  | afc614c5 | 087c6da93318e8dc922d3d22e0a2af7b9759701c    |
| rom_3  | 12000  | 15077a9d | 7badb4318b208f49d7fa65e915d0aa22a1e37915    |
| rom_4  | 13000  | a8b4bf0f | 6b4d47c2899fe9f14f9dab5928499f12078c437d    |
| rom_5  | 14000  | 2d306074 | 54f871983699113e31bb756d4ca885c26c2d66b4    |
| rom_6  | 15000  | 53598dde | 54b02d944caf95283c9b6f0160e75ea8c4ccc97b    |
| rom_7  | 16000  | 23606060 | a487ffcd4920d1056b87469735f7e1002f6a2e49    |
| rom_8  | 17000  | 4ec490c7 | 8726ebaf048db9608dfe365bf434ed5ca9452db7    |
| rom_9  | 18000  | 88187b64 | efacc4a6d4b2af9a236c9d520de6d605c79cc5a8    |
| rom_10 | 0d000  | 60b07ff7 | ba833f48ddfc1bd04ddb41b1d1c840d66ee7da30    |
| rom_11 | 0e000  | 7d2c5daf | 6ca39f493eb8b370154ad46ef01976d352c929e1    |
| rom_12 | 0f000  | a0396670 | c46872550e0ca031453c6513f8f0448ecc9b5572    |

The assembled binaries in `../stargate/roms/01`-`12` match these
checksums byte-for-byte and can serve as a reference-assembler ground
truth.

## RAM Sections (from `../stargate/src/system/memory.gazm`)

| Section  | Start | Size  |
|----------|-------|-------|
| dp_ram   | 9C00  | 0100  |
| work_ram | 9D00  | 2300  |
| cmos_ram | CD00  | 0400  |

## Discrepancies in the gazm Repo

- **CMOS base**: `memory.gazm` places `cmos_ram` at `$CD00`, but both
  `hw.gazm` (`CMOS: EQU $CC00`) and MAME (`CC00-CFFF`) put CMOS at
  `$CC00`. Trust `$CC00`.
- **LED 7-segment**: `memmap.md` claims ROM PIA port B bits 6-7 plus
  CA2/CB2 drive the LED display; MAME straps bits 6-7 high and does not
  model LED segments on the B0 board. Treat the LED claim as unverified.
- **Video RAM size**: schematic split is 38K video (0000-97FF) + 10K work
  RAM (9800-BFFF); MAME unifies 0000-BFFF as one RAM. Behaviourally equal.

## References

- `../stargate/memmap.md` — memory map and PIA details
- `../stargate/src/system/hw.gazm` — register equates and bit maps
- `../stargate/src/system/memory.gazm` — ROM/RAM section layout
- `../stargate/README.md` — build instructions and checksum verification
- MAME `src/mame/midway/williams.cpp` — `ROM_START(stargate)`
- [Arcade History: Stargate](https://www.arcade-history.com/?n=stargate&page=detail&id=2625)
