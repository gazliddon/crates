use super::{diss, CpuResult, DisResult, Disassmbly, RegisterFileTrait, StatusRegTrait};
use crate::cpu::Ins;

use emucore::{
    mem::{MemResult, MemoryIO},
    traits::FlagsTrait,
};

pub struct Machine<M, R>
where
    M: MemoryIO,
    R: RegisterFileTrait + StatusRegTrait,
{
    pub regs: R,
    pub mem: M,
    pub cycle: usize,
    pub irq: bool,
    pub nmi: bool,
    pub reset: bool,
}

fn u8_sign_extend(byte: u8) -> u16 {
    if (byte & 0x80) == 0x80 {
        byte as u16
    } else {
        (byte as u16) | 0xff00
    }
}
impl<M, R> Machine<M, R>
where
    M: MemoryIO,
    R: RegisterFileTrait + StatusRegTrait,
{
    pub fn diss<'a>(&'a self, addr: usize) -> DisResult<Disassmbly<'a>> {
        diss(self.mem(), addr)
    }
}

pub enum StepResult {
    Reset(usize),
    Irq(usize),
    Nmi(usize),
    Step {
        pc: usize,
        next_pc: usize,
        cycles: usize,
    },
}

impl StepResult {
    pub fn new(pc: usize, next_pc: usize, cycles: usize) -> Self {
        Self::Step {
            pc,
            cycles,
            next_pc,
        }
    }
}

impl<M, R> Machine<M, R>
where
    M: MemoryIO,
    R: RegisterFileTrait + StatusRegTrait,
{
    pub fn step_cycles(&mut self, cycles: usize) -> CpuResult<()> {
        self.cycle = 0;

        loop {
            if self.cycle >= cycles {
                break;
            } else {
                self.step()?;
            }
        }

        Ok(())
    }

    pub fn interrupt(&mut self, vec_addr: usize) -> CpuResult<usize> {
        let pc = self.regs.pc();
        let x = self.regs.x();
        let a = self.regs.a();
        let b = self.regs.b();
        let sr = self.regs.sr();

        self.push_word(pc)?;
        self.push_word(x)?;
        self.push_byte(a)?;
        self.push_byte(b)?;
        self.push_byte(sr)?;

        self.regs.sei();

        let addr = self.mem_mut().load_word(vec_addr)?;
        self.regs.set_pc(addr);
        self.regs.sei();
        Ok(addr.into())
    }

    pub fn step(&mut self) -> CpuResult<StepResult> {
        use super::addrmodes::*;
        let cycle = self.cycle;

        let pc = self.regs.pc() as usize;

        if self.reset {
            self.reset = false;
            let v = self.mem_mut().load_word(0xfffe)?;
            self.regs.set_pc(v);
            self.regs.sei();
            self.cycle += 1;
            Ok(StepResult::Reset(v.into()))
        } else if self.nmi {
            self.nmi = false;
            let pc = self.interrupt(0xfffC)?;
            self.cycle += 1;
            Ok(StepResult::Nmi(pc))
        } else if self.irq && !self.regs.i() {
            let pc = self.interrupt(0xfff8)?;
            self.irq = false;
            self.cycle += 1;
            Ok(StepResult::Irq(pc))
        } else {
            let addr = self.regs.pc();
            let op_code = self.mem_mut().load_byte(addr as usize)?;
            self.regs.inc_pc();

            macro_rules! handle_op {
                ($action:ident, $addr:ident, $cycles:expr, $size:expr) => {{
                    let mut ins = Ins {
                        bus: $addr {},
                        m: self,
                    };
                    ins.$action()?;
                    self.cycle += $cycles;
                }};
            }

            let _ = op_table!(op_code, { panic!("NOT IMP") });

            Ok(StepResult::new(
                pc,
                self.regs.pc().into(),
                self.cycle - cycle,
            ))
        }
    }

    pub fn reset(&mut self) {
        self.reset = true;
    }

    pub fn irq(&mut self) {
        self.irq = true;
    }

    pub fn nmi(&mut self) {
        self.nmi = true;
    }

    pub fn new(mem: M, regs: R) -> Self {
        Self {
            mem,
            regs,
            cycle: 0,
            irq: false,
            reset: false,
            nmi: false,
        }
    }

    pub fn mem_mut(&mut self) -> &mut M {
        &mut self.mem
    }

    #[inline]
    pub fn inc_pc(&mut self) {
        self.regs.inc_pc();
    }

    #[inline]
    pub fn inc_inc_pc(&mut self) {
        self.regs.inc_pc();
        self.regs.inc_pc();
    }

    #[inline]
    pub fn fetch_rel_addr(&mut self) -> MemResult<u16> {
        let pc = self.regs.pc();
        let byte = self.mem.load_byte(pc as usize)?;
        let res = pc.wrapping_add(u8_sign_extend(byte));
        Ok(res)
    }

    pub fn fetch_byte(&mut self) -> MemResult<u8> {
        let pc = self.regs.pc();
        let byte = self.mem.load_byte(pc as usize)?;
        self.inc_pc();
        Ok(byte)
    }

    // pub fn fetch_word(&mut self) -> MemResult<u16> {
    //     let pc = self.regs.pc();
    //     let b = self.mem.load_word(pc as usize)?;
    //     self.inc_inc_pc();
    //     Ok(b)
    // }

    pub fn mem(&self) -> &M {
        &self.mem
    }

    // [[SP]] ← [val(LO)],
    // [[SP] - 1] ← [val(HI)],
    // [SP] ← [SP] - 2,
    pub fn push_word(&mut self, val: u16) -> MemResult<()> {
        let sp = self.regs.sp().wrapping_sub(1);
        self.mem.store_word(sp as usize, val)?;
        self.regs.set_sp(sp.wrapping_sub(1));
        Ok(())
    }

    // [[SP]] ← [A], [SP] ← [SP] - 1
    pub fn push_byte(&mut self, val: u8) -> MemResult<()> {
        let sp = self.regs.sp();
        self.mem.store_byte(sp as usize, val)?;
        self.regs.set_sp(sp.wrapping_sub(1));
        Ok(())
    }

    // [res(HI)] ← [[SP] + 1],
    // [res(LO)] ← [[SP] + 2],
    // [SP] ← [SP] + 2
    pub fn pop_word(&mut self) -> MemResult<u16> {
        let sp = self.regs.sp().wrapping_add(1);
        let word = self.mem.load_word(sp as usize)?;
        self.regs.set_sp(sp.wrapping_add(1));
        Ok(word)
    }

    //[SP] ← [SP] + 1, [A] ← [[SP]]
    pub fn pop_byte(&mut self) -> MemResult<u8> {
        let sp = self.regs.sp().wrapping_add(1);
        let byte = self.mem.load_byte(sp as usize)?;
        self.regs.set_sp(sp);
        Ok(byte)
    }
}
