//! CPU core — skeleton.
//!
//! Holds the chip variant, register file and program counter; `step()`
//! execution is not implemented yet (see `alu`). The peripheral block
//! (blitter access on GPU; SSI/DAC/waveform ROM on DSP) will plug in here.

use crate::cpu::{Chip, Registers};
use emucore::cpu::ExecutionStats;
use emucore::mem::MemoryIO;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepOutcome {
    Continue,
    Halted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StepError {
    pub message: &'static str,
}

/// A JRISC core for one chip.
#[derive(Debug, Clone)]
pub struct Cpu {
    pub chip: Chip,
    pub regs: Registers,
    pub pc: u32,
    pub stats: ExecutionStats,
}

impl Cpu {
    pub fn new(chip: Chip) -> Self {
        Self {
            chip,
            regs: Registers::new(chip.default_bank()),
            pc: chip.ram_base(),
            stats: ExecutionStats::default(),
        }
    }

    /// Execute one instruction from `mem`. NOT IMPLEMENTED YET.
    pub fn step(&mut self, _mem: &dyn MemoryIO) -> Result<StepOutcome, StepError> {
        Err(StepError { message: "execution not implemented yet — decode/diss only" })
    }
}
