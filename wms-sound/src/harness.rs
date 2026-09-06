use std::io::{self, Write};
use std::path::Path;

use crate::{DacEvent, WmsSoundBoard};

#[derive(Clone, Copy, Debug)]
pub struct HarnessConfig {
    pub cpu_hz: u32,
    pub sample_rate: u32,
    /// Base address of the sound ROM (Stargate: $F800, Robotron: $F000).
    pub rom_base: usize,
}

impl Default for HarnessConfig {
    fn default() -> Self {
        Self {
            // The exact board oscillator can be supplied by the caller when
            // accurate pitch is required; this is a useful portable default.
            cpu_hz: 894_886,
            sample_rate: 44_100,
            rom_base: 0xf800,
        }
    }
}

pub struct SoundHarness {
    pub board: WmsSoundBoard,
    pub config: HarnessConfig,
    command_log: Vec<SoundCommandEvent>,
    /// Held DAC level carried across render calls: the hardware DAC
    /// keeps its last written value until the next write, so a chunk
    /// boundary must not reset it to mid-scale.
    dac_value: u8,
    /// DAC events not yet consumed by a render call (events beyond the
    /// chunk's cycle window carry into the next chunk).
    pending_events: Vec<DacEvent>,
    /// Absolute board-cycle position of the render cursor: chunks are
    /// contiguous windows, so the first event after a silence is placed
    /// at its exact cycle rather than at the next chunk boundary.
    rendered_cycles: u64,
    /// Command-latch writes queued with their target sound cycle; the
    /// run loop delivers them at the next instruction boundary so the
    /// PIA sees every CB1 edge (chunk-boundary delivery loses the
    /// intermediate edges and can miss interrupts entirely).
    pending_commands: Vec<(u64, u8)>,
}

/// A value presented by the main-board PIA to the sound CPU.
///
/// `cycle` is relative to the start of a capture. Values are written in the
/// same form as the hardware sees them (including Williams' upper two bits).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SoundCommandEvent {
    pub cycle: u64,
    pub pia_value: u8,
}

impl SoundHarness {
    pub fn from_rom(rom: &[u8], config: HarnessConfig) -> Self {
        Self::from_rom_at(rom, config, config.rom_base)
    }

    /// Board with the ROM loaded at an explicit base address.
    pub fn from_rom_at(rom: &[u8], config: HarnessConfig, base: usize) -> Self {
        Self::from_board(WmsSoundBoard::from_rom_at(rom, base), config)
    }

    /// Construct a harness without PIA transaction capture for production
    /// playback and throughput benchmarks.
    pub fn from_rom_fast(rom: &[u8], config: HarnessConfig) -> Self {
        Self::from_board(WmsSoundBoard::from_rom_fast(rom), config)
    }

    fn from_board(mut board: WmsSoundBoard, config: HarnessConfig) -> Self {
        board.reset();
        Self {
            board,
            config,
            command_log: Vec::new(),
            dac_value: 0x80,
            pending_events: Vec::new(),
            rendered_cycles: 0,
            pending_commands: Vec::new(),
        }
    }

    pub fn send_command(&mut self, command: u8) {
        self.board.send_command(command);
        self.command_log.push(SoundCommandEvent {
            cycle: self.board.cycles,
            pia_value: command | 0xc0,
        });
    }

    /// Inject a value exactly as a captured main-board PIA event.
    pub fn send_pia_value(&mut self, pia_value: u8) {
        self.board.send_pia_value(pia_value);
    }

    /// Queue a command-latch write for delivery when the sound CPU's
    /// cycle counter reaches `at_cycle` (MAME's scheduler-synchronized
    /// `snd_cmd_w` semantics).  Delivery happens just before the next
    /// instruction at or past that cycle.
    pub fn send_pia_value_at(&mut self, pia_value: u8, at_cycle: u64) {
        self.pending_commands.push((at_cycle, pia_value));
    }

    pub fn take_command_log(&mut self) -> Vec<SoundCommandEvent> {
        std::mem::take(&mut self.command_log)
    }

    /// Run while applying events at instruction boundaries. The 6800 executes
    /// whole instructions, so an event that falls inside one is delivered at
    /// the next boundary (the same granularity available to this emulator).
    pub fn replay_events(
        &mut self,
        events: &[SoundCommandEvent],
        cycles: u64,
        strict: bool,
    ) -> emu6800::cpu::CpuResult<u64> {
        let end = self.board.cycles + cycles;
        let mut index = 0;
        while self.board.cycles < end {
            while index < events.len() && events[index].cycle <= self.board.cycles {
                self.send_pia_value(events[index].pia_value);
                index += 1;
            }
            if strict {
                self.board.step_strict()?;
            } else {
                self.board.step()?;
            }
        }
        Ok(self.board.cycles)
    }

    pub fn write_command_log<P: AsRef<Path>>(&mut self, path: P) -> io::Result<()> {
        let mut file = std::fs::File::create(path)?;
        for event in self.take_command_log() {
            writeln!(file, "{} {:02x}", event.cycle, event.pia_value)?;
        }
        Ok(())
    }

    pub fn read_command_log<P: AsRef<Path>>(path: P) -> io::Result<Vec<SoundCommandEvent>> {
        let text = std::fs::read_to_string(path)?;
        text.lines()
            .enumerate()
            .filter_map(|(line_no, line)| {
                let line = line.split('#').next()?.trim();
                if line.is_empty() {
                    return None;
                }
                let mut fields = line.split_whitespace();
                let cycle = fields.next()?.parse().map_err(|_| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("invalid cycle on line {}", line_no + 1),
                    )
                });
                let raw = fields
                    .next()?
                    .trim_start_matches("0x")
                    .trim_start_matches('$');
                let value = u8::from_str_radix(raw, 16).map_err(|_| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("invalid PIA value on line {}", line_no + 1),
                    )
                });
                Some(cycle.and_then(|cycle| {
                    value.map(|pia_value| SoundCommandEvent { cycle, pia_value })
                }))
            })
            .collect()
    }

    pub fn run_cycles(&mut self, cycles: u64) -> emu6800::cpu::CpuResult<u64> {
        self.run_cycles_with(cycles, false)
    }

    pub fn run_cycles_strict(&mut self, cycles: u64) -> emu6800::cpu::CpuResult<u64> {
        self.run_cycles_with(cycles, true)
    }

    fn run_cycles_with(&mut self, cycles: u64, strict: bool) -> emu6800::cpu::CpuResult<u64> {
        let target = self.board.cycles + cycles;
        while self.board.cycles < target {
            while self
                .pending_commands
                .first()
                .map(|(at, _)| *at <= self.board.cycles)
                .unwrap_or(false)
            {
                let (_, value) = self.pending_commands.remove(0);
                self.board.send_pia_value(value);
            }
            if strict {
                self.board.step_strict()?;
            } else {
                self.board.step()?;
            }
        }
        Ok(self.board.cycles)
    }

    pub fn take_dac_events(&mut self) -> Vec<DacEvent> {
        self.board.take_dac_events()
    }

    pub fn render_audio(&mut self, duration_cycles: u64) -> Vec<f32> {
        let window_start = self.rendered_cycles;
        self.rendered_cycles += duration_cycles;
        let mut events = self.board.take_dac_events();
        self.pending_events.append(&mut events);
        let total_samples = ((duration_cycles as u128 * self.config.sample_rate as u128)
            / self.config.cpu_hz as u128) as usize;
        let mut pcm = vec![0.0; total_samples];
        let mut event_index = 0;
        for (sample_index, sample) in pcm.iter_mut().enumerate() {
            let cycle = (sample_index as u128 * self.config.cpu_hz as u128
                / self.config.sample_rate as u128) as u64;
            let limit = window_start + cycle;
            while event_index < self.pending_events.len()
                && self.pending_events[event_index].cycle <= limit
            {
                self.dac_value = self.pending_events[event_index].value;
                event_index += 1;
            }
            // The byte DAC maps 0..255 to -1..+1; MAME routes the
            // board's DAC to the speaker with 0.25 gain (verified
            // against a -wavwrite capture: ~0.248 of full scale), so
            // match that or the square-wave tones clip hard.
            *sample = (self.dac_value as f32 - 128.0) / 128.0 * 0.25;
        }
        // Consumed events are drained; anything beyond this chunk's
        // window stays pending for the next render call.
        self.pending_events.drain(..event_index);
        pcm
    }

    /// Render DAC write events into unsigned 8-bit PCM and write a minimal
    /// RIFF/WAVE file. The DAC value is held between CPU writes.
    pub fn write_wav<P: AsRef<Path>>(&mut self, path: P, duration_cycles: u64) -> io::Result<()> {
        let pcm: Vec<u8> = self
            .render_audio(duration_cycles)
            .into_iter()
            .map(|sample| ((sample * 128.0) + 128.0).clamp(0.0, 255.0) as u8)
            .collect();

        let mut file = std::fs::File::create(path)?;
        let data_size = pcm.len() as u32;
        let riff_size = 36 + data_size;
        file.write_all(b"RIFF")?;
        file.write_all(&riff_size.to_le_bytes())?;
        file.write_all(b"WAVEfmt ")?;
        file.write_all(&16u32.to_le_bytes())?;
        file.write_all(&1u16.to_le_bytes())?;
        file.write_all(&1u16.to_le_bytes())?;
        file.write_all(&self.config.sample_rate.to_le_bytes())?;
        file.write_all(&self.config.sample_rate.to_le_bytes())?;
        file.write_all(&1u16.to_le_bytes())?;
        file.write_all(&8u16.to_le_bytes())?;
        file.write_all(b"data")?;
        file.write_all(&data_size.to_le_bytes())?;
        file.write_all(&pcm)?;
        Ok(())
    }
}
