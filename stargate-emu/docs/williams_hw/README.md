# Williams Hardware Docs

Documentation for the Williams 6809 arcade hardware family, organized as
a shared base platform plus per-machine pages.

- [[base|Base Platform]] — clocks, video, interrupts, PIAs, sound, CMOS,
  watchdog, blitter, ROM conventions shared by all machines
- [[blitter|Blitter Programming Guide]] — the blitter from a software
  developer's perspective (Robotron and other B1/B2 games)
- [[defender]] — Defender (1980): no low-ROM banking, banked C000 window
- [[stargate]] — Stargate (1981): low-ROM banking, widget/rom PIAs at
  C804/C80C
- [[robotron]] — Robotron: 2084 (1982): B1 board with blitter, dual
  49-way joysticks

## Family Quick Reference

| Machine  | Year | MAME configuration | Blitter | Joystick     | Low-ROM bank | Sound ROM        |
|----------|------|--------------------|---------|--------------|--------------|------------------|
| Defender | 1980 | defender           | none    | 2-way        | no           | std_1 @ F800     |
| Stargate | 1981 | williams_b0        | none    | 8-way        | yes (C900)   | std_744 @ F800   |
| Robotron | 1982 | williams_b1        | SC1     | dual 49-way  | yes (C900)   | std_767 @ F000   |

## Where Things Live

| Path                                    | Contents                            |
|-----------------------------------------|-------------------------------------|
| `docs/williams_hw/`                     | This documentation                  |
| `../stargate/`                          | Stargate assembly source (gazm)     |
| `../emucore/`                           | Core emulator traits                |
| `../emu6809/`                           | 6809 CPU engine                     |
| `../emu6800/`                           | 6800 CPU engine (ignored for now)   |
| `/Users/garyliddon/development/mame`    | MAME source (authoritative ref)     |

## Authority

Everything marked **(MAME)** in these pages is verified against the MAME
source (`src/mame/midway/williams.cpp`, `williams_m.cpp`,
`williams_v.cpp`). When sources disagree, MAME and the original ROMs win.
