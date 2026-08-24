//! CPU core — decode + execute.
//!
//! `step()` fetches one instruction, executes it, and handles the branch
//! **delay slot**: a taken `jump`/`jr` first executes the instruction at
//! `pc+2` and only then transfers control (the reason the `.GAS` sources
//! pad NOPs after jumps). Taken branches add 3 wait-state cycles.

use crate::cpu::alu::execute;
use crate::cpu::decoder::decode_word;
use crate::cpu::{Chip, DecodedInsn, Flags, Registers};
use crate::isa::Dbase;
use crate::mem::JriscBus;
use emucore::cpu::ExecutionStats;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepOutcome {
    Continue,
    Halted,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StepError {
    pub message: String,
}

/// A JRISC core for one chip.
#[derive(Debug, Clone)]
pub struct Cpu {
    pub chip: Chip,
    pub regs: Registers,
    pub pc: u32,
    /// address of the instruction currently executing (for `move_pc`)
    pub ppc: u32,
    pub flags: Flags,
    pub stats: ExecutionStats,
    /// 64-bit MAC accumulator (imacn/imultn/resmac/sat32s)
    pub accum: i64,
    /// division remainder (div)
    pub div_remainder: u32,
    /// D/G_DIVCTRL bit 0: 48-bit divide mode
    pub div_offset: bool,
    /// G/D_HIDATA (loadp/storep)
    pub hidata: u32,
    /// matrix multiply state (G/D_MTXC / G/D_MTXA)
    pub mtx_width: u8,
    pub mtx_addw: bool,
    pub mtx_addr: u32,
    /// DSP modulo mask (MOD_MASK, $F1A118) for addqmod/subqmod
    pub modulo: u32,
}

impl Cpu {
    pub fn new(chip: Chip) -> Self {
        Self {
            chip,
            regs: Registers::new(chip.default_bank() as usize),
            pc: chip.ram_base(),
            ppc: 0,
            // GPU code is assembled for bank 1 (GASM -R1), DSP for bank 0
            flags: Flags { regpage: chip.default_bank() == 1, ..Flags::default() },
            stats: ExecutionStats::default(),
            accum: 0,
            div_remainder: 0,
            div_offset: false,
            hidata: 0,
            mtx_width: 0,
            mtx_addw: false,
            mtx_addr: chip.ram_base(),
            modulo: 0xffff_ffff,
        }
    }

    /// The register bank selected by REGPAGE (forced to 0 under imask).
    pub fn effective_bank(&self) -> usize {
        if self.flags.imask {
            0
        } else if self.flags.regpage {
            1
        } else {
            0
        }
    }

    fn sync_bank(&mut self) {
        self.regs.set_bank(self.effective_bank());
    }

    /// Fetch + decode one instruction at `pc` (advancing `pc` past it).
    fn fetch(&mut self, bus: &mut dyn JriscBus) -> Result<DecodedInsn, StepError> {
        let word = bus.read_word(self.pc) as u16;
        self.pc = self.pc.wrapping_add(2);
        let (op, src, dst) = decode_word(word);
        let insn = Dbase::get()
            .lookup(op as usize, self.chip.variant())
            .copied()
            .unwrap_or(Dbase::get().unknown);
        let mut size = 2;
        let mut extra = None;
        if insn.extra32 {
            let w1 = bus.read_word(self.pc) as u16;
            let w2 = bus.read_word(self.pc.wrapping_add(2)) as u16;
            self.pc = self.pc.wrapping_add(4);
            extra = Some(((w2 as u32) << 16) | w1 as u32);
            size = 6;
        }
        Ok(DecodedInsn { addr: self.ppc as usize, word, insn, src, dst, extra, size })
    }

    fn execute_with(&mut self, d: &DecodedInsn, bus: &mut dyn JriscBus) -> Result<(), StepError> {
        execute(self, d, bus).map_err(|message| StepError { message })
    }

    /// read a register of the active bank
    pub fn r(&self, i: usize) -> u32 {
        self.regs.get_index(i)
    }

    /// write a register of the active bank
    pub fn w(&mut self, i: usize, v: u32) {
        self.regs.set_index(i, v);
    }

    /// Execute one instruction from `bus`.
    pub fn step(&mut self, bus: &mut dyn JriscBus) -> Result<StepOutcome, StepError> {
        self.sync_bank();
        self.ppc = self.pc;
        let d = self.fetch(bus)?;
        let mut extra_cycles = 0u64;
        match d.insn.mnemonic {
            "jump" => {
                let cc = d.dst;
                let reg = d.src as usize;
                if self.flags.condition(cc) {
                    // target is captured BEFORE the delay slot executes (the
                    // slot may modify the register), per MAME
                    let target = self.regs.get_index(reg);
                    let slot = self.fetch(bus)?;
                    self.execute_with(&slot, bus)?;
                    self.pc = target;
                    extra_cycles = 3;
                }
            }
            "jr" => {
                let cc = d.dst;
                if self.flags.condition(cc) {
                    // offset = signed 5-bit src, in words; relative to the
                    // pc AFTER the branch instruction (before the slot)
                    let off = (((d.src as i8) << 3) >> 3) as i32 * 2;
                    let target = (self.pc as i32).wrapping_add(off) as u32;
                    let slot = self.fetch(bus)?;
                    self.execute_with(&slot, bus)?;
                    self.pc = target;
                    extra_cycles = 3;
                }
            }
            _ => {
                self.execute_with(&d, bus)?;
            }
        }
        self.stats.record(d.insn.cycles as u64 + extra_cycles);
        Ok(StepOutcome::Continue)
    }
}
