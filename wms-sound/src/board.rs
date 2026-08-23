use emu6800::cpu::{
    EffectsChecker, Machine, NoopEffects, RegisterFile, RegisterFileTrait, StepResult,
    VerifyEffects,
};
use emucore::mem::{MemErrorTypes, MemResult, MemoryIO};
use sha1::Digest;
use sha1::Sha1;

use crate::{Dac8, Pia6821, PiaEvent};

pub const SOUND_PIA_BASE: usize = 0x0400;
pub const SOUND_ROM_BASE: usize = 0xf800;

pub struct SoundBus {
    memory: [u8; 0x1_0000],
    pub pia: Pia6821,
    pub dac: Dac8,
    pub cycle: u64,
    capture_pia_accesses: bool,
    pub pia_accesses: Vec<PiaAccess>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PiaAccess {
    pub cycle: u64,
    pub address: u16,
    pub value: u8,
    pub write: bool,
}

impl SoundBus {
    pub fn new(rom: &[u8]) -> Self {
        Self::with_diagnostics(rom, true)
    }

    /// Construct a bus with optional debugger instrumentation. PIA access
    /// capture is useful for MAME comparison, but unnecessary in production
    /// emulation and can be disabled to avoid recording every transaction.
    pub fn with_diagnostics(rom: &[u8], capture_pia_accesses: bool) -> Self {
        let mut memory = [0; 0x1_0000];
        let end = (SOUND_ROM_BASE + rom.len()).min(memory.len());
        memory[SOUND_ROM_BASE..end].copy_from_slice(&rom[..end - SOUND_ROM_BASE]);
        Self {
            memory,
            pia: Pia6821::default(),
            dac: Dac8::default(),
            cycle: 0,
            capture_pia_accesses,
            pia_accesses: Vec::new(),
        }
    }

    fn pia_offset(addr: usize) -> Option<u8> {
        let addr = addr & 0x7fff; // MAME: .mirror(0x8000)
        if addr >= SOUND_PIA_BASE && addr < SOUND_PIA_BASE + 4 {
            Some((addr - SOUND_PIA_BASE) as u8)
        } else {
            None
        }
    }

    fn read_io(&mut self, addr: usize) -> Option<u8> {
        Self::pia_offset(addr).map(|offset| {
            let value = self.pia.read(offset);
            if self.capture_pia_accesses {
                self.pia_accesses.push(PiaAccess {
                    cycle: self.cycle,
                    address: addr as u16,
                    value,
                    write: false,
                });
            }
            value
        })
    }

    fn write_io(&mut self, addr: usize, value: u8) -> bool {
        let Some(offset) = Self::pia_offset(addr) else {
            return false;
        };
        if self.capture_pia_accesses {
            self.pia_accesses.push(PiaAccess {
                cycle: self.cycle,
                address: addr as u16,
                value,
                write: true,
            });
        }
        if let Some(PiaEvent::PortAOutput(value)) = self.pia.write(offset, value) {
            self.dac.write_at(self.cycle, value);
        }
        true
    }
}

impl MemoryIO for SoundBus {
    fn inspect_byte(&self, addr: usize) -> MemResult<u8> {
        if addr < self.memory.len() {
            Ok(self.memory[addr])
        } else {
            Err(MemErrorTypes::IllegalAddress(addr))
        }
    }
    fn inspect_word(&self, addr: usize) -> MemResult<u16> {
        Ok(u16::from_be_bytes([
            self.inspect_byte(addr)?,
            self.inspect_byte(addr + 1)?,
        ]))
    }
    fn upload(&mut self, addr: usize, data: &[u8]) -> MemResult<()> {
        for (i, value) in data.iter().enumerate() {
            self.store_byte(addr + i, *value)?;
        }
        Ok(())
    }
    fn get_range(&self) -> std::ops::Range<usize> {
        0..self.memory.len()
    }
    fn update_sha1(&self, digest: &mut Sha1) {
        digest.update(self.memory);
    }
    fn load_byte(&mut self, addr: usize) -> MemResult<u8> {
        self.read_io(addr)
            .or_else(|| {
                if addr < 0x0100 || addr >= SOUND_ROM_BASE {
                    self.inspect_byte(addr).ok()
                } else {
                    Some(0xff) // unmapped bus reads float high on the board
                }
            })
            .ok_or(MemErrorTypes::IllegalAddress(addr))
    }
    fn store_byte(&mut self, addr: usize, value: u8) -> MemResult<()> {
        if self.write_io(addr, value) {
            return Ok(());
        }
        if addr >= 0x0100 && addr < SOUND_ROM_BASE {
            return Ok(()); // unmapped writes have no device selected
        }
        if addr >= SOUND_ROM_BASE {
            return Err(MemErrorTypes::IllegalWrite(addr));
        }
        self.memory[addr] = value;
        Ok(())
    }
    fn store_word(&mut self, addr: usize, value: u16) -> MemResult<()> {
        let [hi, lo] = value.to_be_bytes();
        self.store_byte(addr, hi)?;
        self.store_byte(addr + 1, lo)
    }
    fn load_word(&mut self, addr: usize) -> MemResult<u16> {
        Ok(u16::from_be_bytes([
            self.load_byte(addr)?,
            self.load_byte(addr + 1)?,
        ]))
    }
}

pub struct WmsSoundBoard {
    pub cpu: Machine<SoundBus, RegisterFile>,
    pub cycles: u64,
    pia_irq_line: bool,
}

impl WmsSoundBoard {
    pub fn from_rom(rom: &[u8]) -> Self {
        Self::from_rom_with_diagnostics(rom, true)
    }

    /// Construct a board with debugger instrumentation enabled or disabled.
    pub fn from_rom_with_diagnostics(rom: &[u8], capture_pia_accesses: bool) -> Self {
        Self {
            cpu: Machine::new(
                SoundBus::with_diagnostics(rom, capture_pia_accesses),
                RegisterFile::default(),
            ),
            cycles: 0,
            pia_irq_line: false,
        }
    }

    /// Fast production constructor without PIA transaction recording.
    pub fn from_rom_fast(rom: &[u8]) -> Self {
        Self::from_rom_with_diagnostics(rom, false)
    }
    pub fn reset(&mut self) {
        self.cpu.reset();
    }
    pub fn step(&mut self) -> emu6800::cpu::CpuResult<StepResult> {
        self.step_with(NoopEffects)
    }

    pub fn step_strict(&mut self) -> emu6800::cpu::CpuResult<StepResult> {
        self.step_with(VerifyEffects)
    }

    fn step_with<C: EffectsChecker>(
        &mut self,
        mut checker: C,
    ) -> emu6800::cpu::CpuResult<StepResult> {
        // CB1 events are latched by the PIA even while its IRQ output is
        // disabled.  Re-check the output here so an event received during
        // board startup is delivered when the sound ROM enables IRQs.
        self.update_pia_irq_line();
        self.cpu.mem.cycle = self.cycles;
        let result = self.cpu.step_with(&mut checker)?;
        let elapsed = match &result {
            StepResult::Step { cycles, .. } => *cycles,
            StepResult::Reset(_) | StepResult::Irq(_) | StepResult::Nmi(_) => 1,
        };
        self.cycles += elapsed as u64;
        Ok(result)
    }
    pub fn send_command(&mut self, command: u8) {
        // Williams' main-board PIA drives the command's upper two bits high;
        // MAME models this in williams_state::snd_cmd_w.
        self.send_pia_value(command | 0xc0);
    }

    pub fn send_pia_value(&mut self, pia_value: u8) {
        self.cpu.mem.pia.set_port_b_input(pia_value);
        if pia_value == 0xff {
            self.cpu.mem.pia.set_cb1(false);
        } else {
            self.cpu.mem.pia.pulse_cb1();
        }
        self.update_pia_irq_line();
    }

    fn update_pia_irq_line(&mut self) {
        let pending = self.cpu.mem.pia.irq_pending();
        if !pending {
            self.pia_irq_line = false;
        } else if !self.pia_irq_line {
            self.cpu.irq();
            self.pia_irq_line = true;
        }
    }
    pub fn dac_value(&self) -> u8 {
        self.cpu.mem.dac.value()
    }
    pub fn take_dac_samples(&mut self) -> Vec<u8> {
        self.cpu.mem.dac.take_samples()
    }
    pub fn take_dac_events(&mut self) -> Vec<crate::DacEvent> {
        self.cpu.mem.dac.take_events()
    }

    pub fn pending_dac_events(&self) -> usize {
        self.cpu.mem.dac.pending_events()
    }

    pub fn take_pia_accesses(&mut self) -> Vec<PiaAccess> {
        std::mem::take(&mut self.cpu.mem.pia_accesses)
    }

    /// Deterministic FNV-1a hash of the complete emulated board state.
    pub fn state_hash(&self) -> u64 {
        let mut hash = 0xcbf29ce484222325u64;
        let mut regs = self.cpu.regs;
        let mut add = |value: u8| {
            hash ^= value as u64;
            hash = hash.wrapping_mul(0x100000001b3);
        };
        for value in [
            (self.cycles >> 56) as u8,
            (self.cycles >> 48) as u8,
            (self.cycles >> 40) as u8,
            (self.cycles >> 32) as u8,
            (self.cycles >> 24) as u8,
            (self.cycles >> 16) as u8,
            (self.cycles >> 8) as u8,
            self.cycles as u8,
            regs.a,
            regs.b,
            (regs.x >> 8) as u8,
            regs.x as u8,
            (regs.sp >> 8) as u8,
            regs.sp as u8,
            (regs.pc >> 8) as u8,
            regs.pc as u8,
            regs.sr(),
            self.cpu.mem.dac.value(),
            self.cpu.irq as u8,
            self.cpu.nmi as u8,
            self.cpu.reset as u8,
            self.cpu.wai as u8,
        ] {
            add(value);
        }
        for address in 0..0x100 {
            add(self.cpu.mem.inspect_byte(address).unwrap_or(0));
        }
        for value in self.cpu.mem.pia.state_bytes() {
            add(value);
        }
        hash
    }
}

#[cfg(test)]
mod tests {
    use super::{WmsSoundBoard, SOUND_PIA_BASE, SOUND_ROM_BASE};
    use emu6800::cpu::RegisterFileTrait;
    use emucore::mem::MemoryIO;

    #[test]
    fn board_routes_pia_port_a_to_dac() {
        let mut board = WmsSoundBoard::from_rom(&[]);
        board.cpu.mem.store_byte(SOUND_PIA_BASE + 1, 0x00).unwrap();
        board.cpu.mem.store_byte(SOUND_PIA_BASE, 0xff).unwrap();
        board.cpu.mem.store_byte(SOUND_PIA_BASE + 1, 0x04).unwrap();
        board.cpu.mem.store_byte(SOUND_PIA_BASE, 0x5a).unwrap();
        assert_eq!(board.dac_value(), 0x5a);
        assert_eq!(board.take_dac_samples(), vec![0x5a]);
    }

    #[test]
    fn reset_vector_is_read_from_sound_rom() {
        let mut rom = vec![0u8; 0x800];
        rom[0x7fe] = 0xf8;
        rom[0x7ff] = 0x00;
        let mut board = WmsSoundBoard::from_rom(&rom);
        board.reset();
        let result = board.step().unwrap();
        assert!(matches!(
            result,
            emu6800::cpu::StepResult::Reset(SOUND_ROM_BASE)
        ));
        assert_eq!(board.cpu.regs.pc(), SOUND_ROM_BASE as u16);
    }

    #[test]
    fn host_command_asserts_sound_cpu_irq() {
        let mut board = WmsSoundBoard::from_rom(&[]);
        board.cpu.mem.pia.write(3, 0x05);
        board.send_command(0x19);
        assert_eq!(board.cpu.mem.pia.port_b_input(), 0xd9);
        assert!(board.cpu.irq);
    }

    #[test]
    fn pia_is_mirrored_at_the_mame_decode_alias() {
        let mut board = WmsSoundBoard::from_rom(&[]);
        board.cpu.mem.store_byte(SOUND_PIA_BASE + 1, 0x00).unwrap();
        board.cpu.mem.store_byte(SOUND_PIA_BASE, 0xff).unwrap();
        board.cpu.mem.store_byte(0x8401, 0x04).unwrap();
        board.cpu.mem.store_byte(0x8400, 0x5a).unwrap();
        assert_eq!(board.dac_value(), 0x5a);
    }
}
