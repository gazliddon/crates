pub mod bus;
pub mod input;

use std::{panic::AssertUnwindSafe, path::Path};

use emu6809::cpu::{Context, CpuErr, Flags, Pins, Regs};
use emu6809::diss::Diss;
pub use input::StargateInput;

pub struct StargateMachine {
    pub bus: bus::StargateBus,
    pub regs: Regs,
    pub pins: Pins,
    pub cycles: u64,
    pub instructions: u64,
    pending_irq: bool,
    pending_firq: bool,
}

impl StargateMachine {
    pub fn from_rom_dir(path: impl AsRef<Path>) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            bus: bus::StargateBus::from_rom_dir(path.as_ref())?,
            regs: Regs::default(),
            pins: Pins::default(),
            cycles: 0,
            instructions: 0,
            pending_irq: false,
            pending_firq: false,
        })
    }

    pub fn reset(&mut self) -> Result<(), CpuErr> {
        self.pending_irq = false;
        self.pending_firq = false;
        self.pins.irq = false;
        self.pins.firq = false;
        let mut cpu = Context::new(&mut self.bus, &mut self.regs, &mut self.pins)?;
        cpu.reset()
    }

    pub fn step(&mut self) -> Result<(), CpuErr> {
        // Keep interrupt requests latched, but only expose them to the CPU
        // when the corresponding mask bit is clear.  The 6809 core otherwise
        // spends interrupt-entry cycles retrying a masked request.
        self.pins.irq = self.pending_irq && !self.regs.flags.contains(Flags::I);
        self.pins.firq = self.pending_firq && !self.regs.flags.contains(Flags::F);
        let serviced_irq = self.pins.irq;
        let serviced_firq = self.pins.firq;
        let mut cpu = Context::new(&mut self.bus, &mut self.regs, &mut self.pins)?;
        let before = cpu.cycles();
        cpu.step()?;
        let elapsed = (cpu.cycles() - before) as u64;
        self.cycles += elapsed;
        if serviced_irq {
            self.pending_irq = false;
            self.pins.irq = false;
        }
        if serviced_firq {
            self.pending_firq = false;
            self.pins.firq = false;
        }
        let interrupts = self.bus.advance_video(elapsed);
        self.pending_irq |= interrupts.irq;
        self.pending_firq |= interrupts.firq;
        self.instructions += 1;
        Ok(())
    }

    pub fn run_instructions(&mut self, count: usize) -> Result<(), CpuErr> {
        for _ in 0..count {
            self.step()?;
        }
        Ok(())
    }

    pub fn video_ascii(&self, columns: usize, rows: usize) -> Vec<String> {
        self.bus.video_ascii(columns, rows)
    }

    pub fn video_rgba(&self) -> Vec<u8> {
        self.bus.video_rgba()
    }

    pub fn video_rgba_visible(&self) -> Vec<u8> {
        self.bus.video_rgba_visible()
    }

    pub fn set_input(&mut self, input: StargateInput) {
        self.bus.set_input(input);
    }

    pub fn take_sound_commands(&mut self) -> Vec<u8> {
        self.bus.take_sound_commands()
    }

    pub fn palette(&self) -> [u8; 16] {
        self.bus.palette()
    }

    /// Decode instructions around the current PC without changing emulator state.
    pub fn disassembly_around(
        &mut self,
        start: u16,
        before: usize,
        after: usize,
    ) -> Vec<(u16, String)> {
        let diss = Diss::new();
        let start = start as usize;
        let mut prefix = Vec::new();

        // Find an instruction boundary before the PC by trying nearby byte
        // offsets and accepting the sequence that lands exactly on the PC.
        for offset in 1..=32 {
            let mut pc = start.wrapping_sub(offset);
            let mut candidate = Vec::new();
            while pc != start && candidate.len() <= before {
                let Some((next, line)) = Self::decode_line(&diss, &mut self.bus, pc) else {
                    candidate.clear();
                    break;
                };
                candidate.push((pc as u16, line));
                pc = next;
            }
            if pc == start && candidate.len() > prefix.len() {
                prefix = candidate;
                if prefix.len() == before {
                    break;
                }
            }
        }

        let mut lines = prefix;
        let mut pc = start;
        for _ in 0..=after {
            let Some((next, line)) = Self::decode_line(&diss, &mut self.bus, pc) else {
                break;
            };
            lines.push((pc as u16, line));
            pc = next;
        }
        lines
    }

    fn decode_line(diss: &Diss, bus: &mut bus::StargateBus, pc: usize) -> Option<(usize, String)> {
        let decoded = std::panic::catch_unwind(AssertUnwindSafe(|| diss.diss(bus, pc))).ok()?;
        let bytes = decoded
            .decoded
            .data
            .iter()
            .map(|byte| format!("{byte:02X}"))
            .collect::<Vec<_>>()
            .join(" ");
        Some((
            decoded.decoded.next_addr & 0xffff,
            format!("{pc:04X}  {bytes:<11} {}", decoded.text),
        ))
    }
}
