# Williams Blitter Programming Guide

A practical manual for the Williams blitter as used on **Robotron: 2084**
(B1 board, MAME `WILLIAMS_BLITTER_SC1`). Written from the software side:
framebuffer model, registers, control bits, recipes, and 6809 examples.

Stargate and Defender have **no blitter**; this page applies to Robotron
and the other B1/B2 games (Joust, Sinistar, Splat).

## 1. The Framebuffer (Read This First)

The screen is a **column-major bitmap**, not a tilemap. Think of video RAM
as 256 byte-columns; every column is 256 bytes tall:

```text
byte address of (pixel_x, pixel_y) = (pixel_x / 2) * 256 + pixel_y
```

- Each byte holds two pixels: **high nibble = even `pixel_x`**, **low
  nibble = odd `pixel_x`**.
- Each nibble is a 4-bit index into the 16 colour registers at `C000-C00F`
  (`BBGGGRRR` format). No palette = wrong colours.
- The visible width is about 148 byte-columns (512 pixels wide screen,
  visible 6-297); the framebuffer addressing scheme goes to 256 columns
  but only about 38K of video RAM is populated. Keep video blits below
  `$9800`.
- The display scans columns 0 to N for each row; the CPU writes absolute
  byte addresses.

Consequences that surprise new programmers:

- **+1 address = one row down** (same column), **+256 = two pixels
  right** (next column).
- Sprites are naturally **even-x aligned** (a byte covers 2 pixels).
  Odd-x placement needs the SHIFT bit (see section 6).
- "Row" and "column" are swapped versus every other machine you have
  coded for.

## 2. Registers

| Addr  | Register     | Notes                                  |
|-------|--------------|----------------------------------------|
| CA00  | Control      | **writing this triggers the blit**     |
| CA01  | Mask/Solid   | used with SOLID / FG_ONLY              |
| CA02  | Source hi    |                                        |
| CA03  | Source lo    |                                        |
| CA04  | Dest hi      |                                        |
| CA05  | Dest lo      |                                        |
| CA06  | Width        | stored value = width XOR 4             |
| CA07  | Height       | stored value = height XOR 4            |

All eight registers are **write-only** and aliased throughout `CA00-CAFF`
(the low 3 address bits select the register). There is no status
register, no completion flag, and no blitter interrupt.

The width/height XOR-4 oddity is hardware (bit 2 of the bus is inverted
on SC1 boards): to blit w x h, write `w ^ 4` and `h ^ 4`. On Robotron
`4 ^ 4 = 0`, so a 4x4 blit stores 0 — the hardware treats 0 as 1, so a
real 4x4 blit needs width 5. Width and height are byte counts; a width of
w bytes is 2w pixels.

## 3. Control Byte (CA00)

| Bit | Name            | Effect                                  |
|-----|-----------------|-----------------------------------------|
| 7   | NO_EVEN         | Don't touch even (high) nibbles         |
| 6   | NO_ODD          | Don't touch odd (low) nibbles           |
| 5   | SHIFT           | Shift source one pixel right            |
| 4   | SOLID           | Write mask byte (CA01) instead of src   |
| 3   | FG_ONLY         | Zero source nibbles are transparent     |
| 2   | SLOW            | Double the blit time                    |
| 1   | DST_STRIDE_256  | Dest advances per framebuffer column    |
| 0   | SRC_STRIDE_256  | Source advances per framebuffer column  |

Writing CA00 performs the blit synchronously: the blitter steals bus
cycles and the CPU stalls until it is done.

## 4. Addressing & Stride Modes

The blit is an `h`-row loop of `w`-byte rows. Each step advances the
source and dest pointers; the stride bits choose the layout:

| SRC_STRIDE | SRC advance per x | per y | Layout                          |
|------------|-------------------|-------|---------------------------------|
| 0          | +1                | +w    | compact art, w bytes per row    |
| 1          | +0x100            | +1    | framebuffer (column-major)      |

Same table for the destination with DST_STRIDE_256. Source and dest
advance independently, so all four combinations are useful:

- **Compact art -> framebuffer**: `DST_STRIDE_256` only (0x02). Sprite
  data packed in ROM, w contiguous bytes per row; written as a rectangle
  on screen. The normal sprite blit.
- **Framebuffer -> framebuffer**: both bits (0x03). Copy/move a region of
  the screen.
- **Framebuffer -> compact**: `SRC_STRIDE_256` only (0x01). Read a
  screen region into contiguous RAM.
- **Neither**: raw contiguous-to-contiguous copy (rare on this hw).

Source reads go through the **CPU memory map** (banking applies); dest
writes below `$C000` always land in video RAM (banking does not apply to
writes). Both addresses wrap at 16 bits. With STRIDE_256 the row (low
byte) wraps independently of the column (high byte).

## 5. What Actually Gets Written

For each byte, the blitter reads the current dest byte, decides per
nibble whether to keep or replace it, then writes:

- A nibble is **kept** (not written) when: its NO_EVEN/NO_ODD bit is
  set, **or** FG_ONLY is set and the source nibble is zero.
- Otherwise, the nibble is **replaced**: with the source nibble, or with
  the CA01 nibble when SOLID is set.

The four common looks:

| Control bits             | Result                          |
|--------------------------|---------------------------------|
| `0x02` (DST_STRIDE)      | opaque copy (zeros included)    |
| `0x0A` + FG_ONLY         | transparent copy (0 = clear)    |
| `0x12` SOLID             | fill rectangle with CA01 colour |
| `0x1A` SOLID + FG_ONLY   | Draw mask colour where src != 0 |

## 6. Placing Sprites on Odd X

Byte-aligned blits can only start at even pixel positions. For odd x:

- Use the **SHIFT** bit: the source byte stream is shifted right by one
  pixel (4 bits) as it is written, with a leading zero nibble.
- Set the dest column to `(x - 1) / 2` and add **FG_ONLY** so the
  leading zero nibble is transparent (it would otherwise erase the pixel
  at `x - 1`).
- The shift register is continuous across the **whole blit**: the
  trailing nibble of each art row carries into the start of the next dest
  row. So each art row must end with at least one **zero pad nibble**,
  i.e. `w = (pixel_width / 2) + 1` bytes, and art rows must be stored at
  `w`-byte boundaries in the source. Without the pad, each row after the
  first is shifted one pixel out of alignment.
- Because of the leading nibble plus the per-row pad, a shape that is 2w
  pixels wide needs w+1 source bytes per row when shifted.

## 7. 6809 Examples

### Example 1: Copy a 16x16 Sprite from ROM Art to (64, 32)

Art: 16 contiguous bytes per row, zero nibbles = transparent. Screen
position (64, 32): dest column 32, row 32.

```asm
; X = source address in ROM (banked in, see notes)
        TFR     X,D
        STB     $CA03           ; source lo
        STA     $CA02           ; source hi
        LDA     #32
        STA     $CA04           ; dest hi = column 32  (col * 256)
        LDA     #32
        STA     $CA05           ; dest lo = row 32
        LDA     #(16 ^ 4)       ; width 16 bytes
        STA     $CA06
        LDA     #(16 ^ 4)       ; height 16 rows
        STA     $CA07
        LDA     #$0A            ; DST_STRIDE_256 | FG_ONLY
        STA     $CA00           ; go (CPU stalls until done)
```

### Example 2: Fill an 8x8 Region with Solid Colour 3

```asm
        LDA     #$33            ; mask byte: both pixels = colour 3
        STA     $CA01
        LDA     #$20
        STA     $CA04           ; dest hi = column 32
        LDA     #$40
        STA     $CA05           ; dest lo = row 64
        LDA     #(8 ^ 4)
        STA     $CA06
        STA     $CA07           ; 8x8
        LDA     #$12            ; DST_STRIDE_256 | SOLID
        STA     $CA00
```

### Example 3: The Same Sprite on an Odd X (65, 32)

Dest column becomes `(65 - 1) / 2 = 32`, and we add SHIFT:

```asm
; (source setup identical to example 1)
        LDA     #32
        STA     $CA04           ; dest hi = (65-1)/2 = 32
        LDA     #32
        STA     $CA05
        LDA     #(17 ^ 4)       ; 16 px + shift padding -> 17 bytes
        STA     $CA06
        LDA     #(16 ^ 4)
        STA     $CA07
        LDA     #$2A            ; SHIFT | DST_STRIDE_256 | FG_ONLY
        STA     $CA00
```

## 8. Timing & Budget

The blitter steals bus cycles; the CPU is stalled for the duration. MAME
charges roughly:

```text
fast: 4 + 2 * (2*w*h + 3) bus clocks @ 4 MHz   (~1 CPU cycle per byte)
slow: 4 + 4 * (2*w*h + 2) bus clocks @ 4 MHz   (~2 CPU cycles per byte)
```

A 16x16 sprite costs about 256 CPU cycles (~256 µs at 1 MHz), a 32x32
about 1024 cycles. At 60 fps you have ~16,700 cycles per frame, so
budget carefully: a handful of large blits, or many small ones. Full
screen clears are expensive either way (CPU loop or blit); prefer
dirty-region redraws.

## 9. Pitfalls

- **Write CA01-CA07 before CA00.** CA00 is the trigger.
- **Width/height are XOR 4** on Robotron (SC1). 4 becomes 0, which the
  hardware treats as 1 — plan for it.
- **Source banking**: source reads follow the CPU map. If art lives in
  the low ROM (0000-8FFF), select the ROM bank (`C900` bit 0 = 1) before
  the blit, then return to video RAM to draw with the CPU.
- **No polling**: the blit is synchronous; there is nothing to check.
  Just write CA00 and continue.
- **Keep video content below `$9800`** (visible region); blits beyond
  that land in work RAM.
- **16-bit wraparound** on source and dest; with STRIDE_256 the row wraps
  without touching the column.
- **Palette**: nibbles are indices into `C000-C00F`. Until you set the
  colour registers, blits show garbage colours.
- **SHIFT + FG_ONLY** for odd-x; pad each art row with a zero nibble
  (`w = width/2 + 1`) or the stream drifts.

## 10. References

- MAME `src/mame/midway/williams_v.cpp` — `blitter_w`, `blitter_core`,
  `blit_pixel`, `blitter_init`
- MAME `src/mame/midway/williams.cpp` — header comment block (blitter
  register summary), the `williams_b1` machine configuration
- MAME `src/mame/midway/williams.h` — `WMS_BLITTER_CONTROLBYTE_*`
  definitions
- See also [[robotron]] and [[base|Williams Base Platform]]
