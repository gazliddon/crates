# WMS sound-board emulator

This crate is the board-level harness for the early Williams/Midway sound
board used by Stargate and related games. The initial model contains:

- the existing `emu6800` CPU;
- 128-byte board RAM at `$0000..$007f`;
- a 6821 PIA at `$0400..$0403`;
- an 8-bit DAC fed by PIA port A;
- a 2 KiB sound ROM at `$f800..$ffff`.

The host sends a sound command through PIA port B and pulses CB1, which raises
the sound CPU's IRQ input. DAC writes can be collected with
`WmsSoundBoard::take_dac_samples()`.

For a platform-independent interactive session, run the bundled stdin REPL:

```text
cargo run -p wms-sound -- ../stargate/roms/sound.bin
sound 0x19
wav coin.wav 1000000
quit
```

The harness advances the CPU, collects timestamped DAC writes, and emits
unsigned 8-bit PCM in a standard WAV file. It does not open an operating
system audio device; any player or higher-level frontend can consume the WAV.

The default CPU clock is 894,886 Hz, derived from the board's approximately
3.58 MHz crystal divided by four. `HarnessConfig` can override this for other
Williams board revisions.

This is deliberately a first hardware slice. PIA edge polarity, CA1/CA2,
CB2, and analogue DAC voltage modelling will be tightened as the board ROM is
executed and compared with traces.
