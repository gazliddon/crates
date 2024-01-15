use super::{diss, CpuResult, DisResult, Disassmbly, RegisterFileTrait, StatusRegTrait};
use crate::cpu::Ins;
use crate::cpu_core::u8_sign_extend;

use emucore::mem::{MemResult, MemoryIO};

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CpuState {
    NmiPending,
    IrqPending,
    ResetPending,
    Running,
}

impl CpuState {
    pub fn new<M, R>(m: &Machine<M, R>) -> Self
    where
        M: MemoryIO,
        R: RegisterFileTrait + StatusRegTrait,
    {
        if m.reset {
            CpuState::ResetPending
        } else if m.nmi {
            CpuState::NmiPending
        } else if m.irq && !m.regs.i() {
            CpuState::IrqPending
        } else {
            CpuState::Running
        }

    }
    pub fn will_interrupt(&self) -> bool {
        use CpuState::*;

        match self {
            NmiPending | IrqPending | ResetPending => true,
            Running => false,
        }
    }
    pub fn will_run(&self) -> bool {
        !self.will_interrupt()
    }
}

pub struct Machine<M, R>
where
    M: MemoryIO,
    R: RegisterFileTrait + StatusRegTrait,
{
    pub regs: R,
    pub mem: M,
    pub cycle: usize,
    pub nmi: bool,
    pub reset: bool,
    pub irq: bool,
    pub wai: bool,
}


impl<M, R> Machine<M, R>
where
    M: MemoryIO,
    R: RegisterFileTrait + StatusRegTrait,
{
    pub fn diss<'a>(&'a self, addr: usize) -> DisResult<Disassmbly<'a>> {
        diss(self.mem(), addr)
    }

    pub fn get_cpu_state(&self) -> CpuState {
        CpuState::new(self)
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
impl Default for StepResult {
    fn default() -> Self {
        Self::new(0,0,0)
    }
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

    pub fn step(&mut self) -> CpuResult<StepResult> {
        use super::addrmodes::*;
        let cycle = self.cycle;

        let pc = self.regs.pc() as usize;

        use CpuState::*;

        match self.get_cpu_state() {
            NmiPending => {
                let pc = self.interrupt(0xfffC)?;
                self.nmi = false;
                self.cycle += 1;
                Ok(StepResult::Nmi(pc))
            }
            IrqPending => {
                let pc = self.interrupt(0xfff8)?;
                self.irq = false;
                self.cycle += 1;
                Ok(StepResult::Irq(pc))
            }

            ResetPending => {
                self.reset = false;
                let v = self.mem_mut().load_word(0xfffe)?;
                self.regs.set_pc(v);
                self.regs.sei();
                self.cycle += 1;
                Ok(StepResult::Reset(v.into()))
            }

            Running => {
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

                let _ = op_table!(op_code, { panic!("NOT IMP PC: {:04x} {:02x}", addr,op_code ) });

                Ok(StepResult::new(
                    pc,
                    self.regs.pc().into(),
                    self.cycle - cycle,
                ))
            }
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
            wai: false,
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

    pub fn mem(&self) -> &M {
        &self.mem
    }

    // [[SP]] ← [val(LO)],
    // [[SP] - 1] ← [val(HI)],
    // [SP] ← [SP] - 2,
    pub fn push_word(&mut self, val: u16) -> MemResult<()> {
        let lo = (val & 0xff) as u8;
        let hi = (val >> 8) as u8;

        self.push_byte(lo)?;
        self.push_byte(hi)
    }

    // [res(HI)] ← [[SP] + 1],
    // [res(LO)] ← [[SP] + 2],
    // [SP] ← [SP] + 2
    pub fn pop_word(&mut self) -> MemResult<u16> {
        let hi = self.pop_byte()?;
        let lo = self.pop_byte()?;
        Ok(lo as u16 | (( hi as u16 ) << 8) )
    }

    // [[SP]] ← [A], [SP] ← [SP] - 1
    pub fn push_byte(&mut self, val: u8) -> MemResult<()> {
        let sp = self.regs.sp();
        self.mem.store_byte(sp as usize, val)?;
        self.regs.set_sp(sp.wrapping_sub(1));
        Ok(())
    }


    //[SP] ← [SP] + 1, [A] ← [[SP]]
    pub fn pop_byte(&mut self) -> MemResult<u8> {
        let sp = self.regs.sp().wrapping_add(1);
        let byte = self.mem.load_byte(sp as usize)?;
        self.regs.set_sp(sp);
        Ok(byte)
    }
}
