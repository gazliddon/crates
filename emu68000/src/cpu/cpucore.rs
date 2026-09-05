//! M68000 CPU core: state, fetch/decode, effective-address engine, step.

use crate::cpu::alu::execute;
use crate::cpu::bus::M68kBus;
use crate::cpu::decoder::{decode_with, DecodedInsn, Ea};
use crate::cpu::registers::{Reg68k, Registers};
use crate::isa::Size;
use emucore::cpu::ExecutionStats;
use emucore::traits::RegisterFileTrait;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecError {
    pub message: String,
}

impl fmt::Display for ExecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ExecError {}

/// A 68000 core (base M68000).
#[derive(Debug, Clone)]
pub struct Cpu {
    pub regs: Registers,
    pub stats: ExecutionStats,
    /// set by STOP (and unused-by-the-module traps)
    pub halted: bool,
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            regs: Registers::new(),
            stats: ExecutionStats::default(),
            halted: false,
        }
    }

    /// Fetch + decode the instruction at `pc`, advancing `pc` past it.
    fn fetch<B: M68kBus + ?Sized>(&mut self, bus: &mut B) -> Result<DecodedInsn, ExecError> {
        let pc = self.regs.pc;
        let d = decode_with(pc as usize, |a| -> Result<u16, crate::cpu::DecodeError> {
            Ok(bus.read_word(a as u32))
        })
        .map_err(|e| ExecError { message: e.message })?;
        self.regs.pc = pc.wrapping_add(d.size as u32);
        Ok(d)
    }

    /// Execute one instruction from `bus`.
    pub fn step<B: M68kBus + ?Sized>(&mut self, bus: &mut B) -> Result<(), ExecError> {
        if self.halted {
            return Ok(());
        }
        let d = self.fetch(bus)?;
        let cycles = execute(self, &d, bus)?;
        self.stats.record(cycles as u64);
        Ok(())
    }
}

impl Default for Cpu {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------- effective-address engine ----------------

/// operand width in bytes for an entry's size
pub fn size_bytes(s: Size) -> u8 {
    match s {
        Size::B => 1,
        Size::W => 2,
        Size::L => 4,
        Size::None => 4,
    }
}

/// post/pre-increment step for an address register (A7 steps 2 for byte)
fn step_amt(reg: u8, size: u8) -> u32 {
    if size == 1 && reg == 7 {
        2
    } else {
        size as u32
    }
}

/// The memory address for a memory-mode EA. Callers must ensure the mode
/// is 2-6 or 7/0-3 (never Dn/An/#imm).
pub fn ea_addr<B: M68kBus + ?Sized>(
    cpu: &mut Cpu,
    bus: &mut B,
    d: &DecodedInsn,
    e: &Ea,
) -> Result<u32, ExecError> {
    let _ = bus;
    match e.mode {
        2 => Ok(cpu.regs.read_a(e.reg as usize)),
        3 => Ok(cpu.regs.read_a(e.reg as usize)),
        4 => Ok(cpu.regs.read_a(e.reg as usize)),
        5 => {
            let disp = e.ext[0] as i16 as i32;
            Ok(cpu.regs.read_a(e.reg as usize).wrapping_add(disp as u32))
        }
        6 => {
            let w = e.ext[0];
            let d8 = (w & 0x7F) as u8 as i8 as i32;
            let idx = index_value(cpu, w);
            Ok(cpu
                .regs
                .read_a(e.reg as usize)
                .wrapping_add(d8 as u32)
                .wrapping_add(idx))
        }
        7 => match e.reg {
            0 => Ok(e.ext[0] as i16 as i32 as u32),
            1 => Ok(((e.ext[0] as u32) << 16) | e.ext[1] as u32),
            2 => {
                let ref_pc = e.ref_pc.unwrap_or(d.addr as u32 + 4);
                Ok(ref_pc.wrapping_add(e.ext[0] as i16 as i32 as u32))
            }
            3 => {
                let ref_pc = e.ref_pc.unwrap_or(d.addr as u32 + 4);
                let w = e.ext[0];
                let d8 = (w & 0x7F) as u8 as i8 as i32;
                let idx = index_value(cpu, w);
                Ok(ref_pc.wrapping_add(d8 as u32).wrapping_add(idx))
            }
            _ => Err(ExecError {
                message: format!("invalid EA mode 7/{}", e.reg),
            }),
        },
        _ => Err(ExecError {
            message: format!("invalid EA mode {}", e.mode),
        }),
    }
}

/// d8(An,Xn) / d8(PC,Xn) index contribution: D/A bit 15, register bits
/// 14-12, size bit 11, scale bits 10-9.
fn index_value(cpu: &Cpu, w: u16) -> u32 {
    let reg = ((w >> 12) & 7) as usize;
    let long = w & 0x0800 != 0;
    let scale = 1u32 << ((w >> 9) & 3);
    let base = if w & 0x8000 != 0 {
        cpu.regs.read_a(reg)
    } else {
        cpu.regs.read_d(reg)
    };
    if long {
        base.wrapping_mul(scale)
    } else {
        (base as u16 as i16 as i32 as u32).wrapping_mul(scale)
    }
}

/// Read an operand (value) from an EA with the given width. Mode 7/4
/// (immediate) reads the extension words; Dn/An return register values.
pub fn read_ea<B: M68kBus + ?Sized>(
    cpu: &mut Cpu,
    bus: &mut B,
    d: &DecodedInsn,
    e: &Ea,
    size: u8,
) -> Result<u32, ExecError> {
    let v = match e.mode {
        0 => cpu.regs.read_d(e.reg as usize),
        1 => cpu.regs.read_a(e.reg as usize),
        4 => {
            // predecrement: adjust before the read
            let a = cpu.regs.read_a(e.reg as usize) - step_amt(e.reg, size);
            cpu.set_a(e.reg as usize, a);
            read_mem(bus, a, size)
        }
        3 => {
            let a = cpu.regs.read_a(e.reg as usize);
            cpu.set_a(e.reg as usize, a + step_amt(e.reg, size));
            read_mem(bus, a, size)
        }
        7 if e.reg == 4 => match size {
            1 => (e.ext[0] & 0xFF) as u32,
            2 => e.ext[0] as u32,
            _ => ((e.ext[0] as u32) << 16) | e.ext[1] as u32,
        },
        _ => {
            let a = ea_addr(cpu, bus, d, e)?;
            read_mem(bus, a, size)
        }
    };
    Ok(v)
}

/// Write an operand (value) to an EA. Dn writes affect the low `size`
/// bits only (upper bits unchanged — the 68000 ALU rule); callers that
/// need full-register writes (MOVE) handle that separately.
pub fn write_ea<B: M68kBus + ?Sized>(
    cpu: &mut Cpu,
    bus: &mut B,
    d: &DecodedInsn,
    e: &Ea,
    size: u8,
    v: u32,
) -> Result<(), ExecError> {
    match e.mode {
        0 => {
            let mask = match size {
                1 => 0xFF,
                2 => 0xFFFF,
                _ => 0xFFFF_FFFF,
            };
            cpu.regs.d[e.reg as usize] = (cpu.regs.d[e.reg as usize] & !mask) | (v & mask);
            Ok(())
        }
        2 | 5 | 6 | 3 => {
            if e.mode == 3 {
                let a = cpu.regs.read_a(e.reg as usize);
                cpu.set_a(e.reg as usize, a + step_amt(e.reg, size));
                return write_mem(bus, a, size, v);
            }
            let a = ea_addr(cpu, bus, d, e)?;
            write_mem(bus, a, size, v)
        }
        4 => {
            let a = cpu.regs.read_a(e.reg as usize) - step_amt(e.reg, size);
            cpu.set_a(e.reg as usize, a);
            write_mem(bus, a, size, v)
        }
        7 => match e.reg {
            0 | 1 | 2 | 3 => {
                let a = ea_addr(cpu, bus, d, e)?;
                write_mem(bus, a, size, v)
            }
            _ => Err(ExecError {
                message: format!("cannot write EA mode 7/{}", e.reg),
            }),
        },
        _ => Err(ExecError {
            message: format!("cannot write EA mode {}", e.mode),
        }),
    }
}

fn read_mem<B: M68kBus + ?Sized>(bus: &mut B, addr: u32, size: u8) -> u32 {
    match size {
        1 => bus.read_byte(addr) as u32,
        2 => bus.read_word(addr) as u32,
        _ => bus.read_long(addr),
    }
}

fn write_mem<B: M68kBus + ?Sized>(
    bus: &mut B,
    addr: u32,
    size: u8,
    v: u32,
) -> Result<(), ExecError> {
    match size {
        1 => bus.write_byte(addr, v as u8),
        2 => bus.write_word(addr, v as u16),
        _ => bus.write_long(addr, v),
    }
    Ok(())
}

/// Full-register write of a MOVE-style result (Dn: zero-extended low bits;
/// An: full value).
pub fn write_reg(cpu: &mut Cpu, reg: u8, size: u8, v: u32) {
    match size {
        1 => cpu.regs.d[reg as usize] = v & 0xFF,
        2 => cpu.regs.d[reg as usize] = v & 0xFFFF,
        _ => cpu.regs.d[reg as usize] = v,
    }
}

// ---------------- stack helpers ----------------

impl Cpu {
    /// pop a long from the active stack pointer
    pub fn pop_long<B: M68kBus + ?Sized>(&mut self, bus: &mut B) -> u32 {
        let sp = self.regs.sp();
        let v = bus.read_long(sp);
        self.regs.set_sp(sp + 4);
        v
    }

    /// push a long onto the active stack pointer
    pub fn push_long<B: M68kBus + ?Sized>(&mut self, bus: &mut B, v: u32) {
        let sp = self.regs.sp() - 4;
        self.regs.set_sp(sp);
        bus.write_long(sp, v);
    }

    /// read a register by number (0-7 = Dn, 8-15 = An)
    pub fn r(&self, n: usize) -> u32 {
        if n < 8 {
            self.regs.read_d(n)
        } else {
            self.regs.read_a(n - 8)
        }
    }

    pub fn w(&mut self, n: usize, v: u32) {
        if n < 8 {
            self.regs.d[n] = v;
        } else {
            self.set_a(n - 8, v);
        }
    }

    pub fn set_a(&mut self, n: usize, v: u32) {
        if n == 7 {
            self.regs.set_sp(v);
        } else {
            self.regs.a[n] = v;
        }
    }

    /// condition-code register (low byte of SR)
    pub fn ccr(&self) -> u8 {
        (self.regs.sr & 0xFF) as u8
    }

    /// named-register access for tests/observers
    pub fn get_reg(&self, r: Reg68k) -> u32 {
        self.regs.get(&r) as u32
    }
}
