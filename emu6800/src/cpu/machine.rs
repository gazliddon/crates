use super::{
    diss, CpuErrKind, CpuResult, DisResult, Disassmbly, RegisterFileTrait, StatusRegTrait,
};
use crate::cpu::Ins;
use crate::cpu_core::{u8_sign_extend, RegEnum, StatusReg};

use emucore::mem::{MemResult, MemoryIO};

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CpuState {
    NmiPending,
    IrqPending,
    ResetPending,
    Waiting,
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
        } else if m.wai && !(m.irq && !m.regs.i()) {
            CpuState::Waiting
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
            Running | Waiting => false,
        }
    }
    pub fn will_run(&self) -> bool {
        !self.will_interrupt()
    }
}

#[derive(Clone)]
pub struct Machine<M, R>
where
    M: MemoryIO,
    R: RegisterFileTrait + StatusRegTrait,
{
    pub regs: R,
    pub mem: M,
    pub cycle: usize,
    pub instructions: u64,
    pub nmi: bool,
    pub reset: bool,
    pub irq: bool,
    pub wai: bool,
}

/// Hook used to validate the architectural effects of an instruction.
/// `NoopEffects` is the default and is optimized away from the normal step
/// path; `VerifyEffects` is intended for emulator development and tests.
pub trait EffectsChecker {
    type Snapshot;

    fn before<R: RegisterFileTrait + StatusRegTrait>(
        &mut self,
        regs: &mut R,
        opcode: u8,
    ) -> Self::Snapshot;

    fn after<R: RegisterFileTrait + StatusRegTrait>(
        &mut self,
        regs: &mut R,
        opcode: u8,
        snapshot: Self::Snapshot,
    ) -> CpuResult<()>;
}

#[derive(Default)]
pub struct NoopEffects;

impl EffectsChecker for NoopEffects {
    type Snapshot = ();

    #[inline(always)]
    fn before<R: RegisterFileTrait + StatusRegTrait>(
        &mut self,
        _regs: &mut R,
        _opcode: u8,
    ) -> Self::Snapshot {
    }

    #[inline(always)]
    fn after<R: RegisterFileTrait + StatusRegTrait>(
        &mut self,
        _regs: &mut R,
        _opcode: u8,
        _snapshot: Self::Snapshot,
    ) -> CpuResult<()> {
        Ok(())
    }
}

#[derive(Clone, Copy)]
pub struct RegisterSnapshot {
    a: u8,
    b: u8,
    x: u16,
    pc: u16,
    sp: u16,
    sr: u8,
}

#[derive(Default)]
pub struct VerifyEffects;

impl EffectsChecker for VerifyEffects {
    type Snapshot = RegisterSnapshot;

    fn before<R: RegisterFileTrait + StatusRegTrait>(
        &mut self,
        regs: &mut R,
        _opcode: u8,
    ) -> Self::Snapshot {
        RegisterSnapshot {
            a: regs.a(),
            b: regs.b(),
            x: regs.x(),
            pc: regs.pc(),
            sp: regs.sp(),
            sr: regs.sr(),
        }
    }

    fn after<R: RegisterFileTrait + StatusRegTrait>(
        &mut self,
        regs: &mut R,
        opcode: u8,
        before: Self::Snapshot,
    ) -> CpuResult<()> {
        let Some(info) =
            super::opcodes::ISA_DBASE.get_instruction_info_from_opcode(opcode as usize)
        else {
            return Ok(());
        };

        let expected = &info.opcode_data.regs_written;
        let mut unexpected = Vec::new();
        if before.a != regs.a() && !expected.contains(&RegEnum::A) {
            unexpected.push("A");
        }
        if before.b != regs.b() && !expected.contains(&RegEnum::B) {
            unexpected.push("B");
        }
        if before.x != regs.x() && !expected.contains(&RegEnum::X) {
            unexpected.push("X");
        }
        if before.sp != regs.sp() && !expected.contains(&RegEnum::SP) {
            unexpected.push("SP");
        }

        // PC advances during fetch for every instruction, so it is checked
        // only by control-flow tests rather than treated as an unexpected
        // write here.
        let _pc_changed = before.pc != regs.pc();

        let changed_flags = StatusReg::from_bits_truncate(before.sr ^ regs.sr());
        let unexpected_flags = changed_flags & !info.instruction.flags_written;
        if !unexpected_flags.is_empty() {
            unexpected.push("flags");
        }

        if unexpected.is_empty() {
            Ok(())
        } else {
            Err(CpuErrKind::Effects(format!(
                "opcode ${opcode:02x} ({:?}) changed unexpected state: {}",
                info.mnemonic,
                unexpected.join(", ")
            )))
        }
    }
}

impl<M, R> Machine<M, R>
where
    M: MemoryIO,
    R: RegisterFileTrait + StatusRegTrait,
{
    pub fn get_state(&self) {}
    pub fn diss<'a>(&self, addr: usize) -> DisResult<Disassmbly<'a>> {
        diss(self.mem(), addr)
    }

    pub fn get_cpu_state(&self) -> CpuState {
        CpuState::new(self)
    }
}

impl<M, R> emucore::cpu::Cpu for Machine<M, R>
where
    M: MemoryIO,
    R: RegisterFileTrait + StatusRegTrait,
{
    type Error = CpuErrKind;

    fn reset(&mut self) -> Result<(), Self::Error> {
        Machine::reset(self);
        Ok(())
    }

    fn step(&mut self) -> Result<emucore::cpu::CpuStep, Self::Error> {
        let before = self.cycle;
        Machine::step(self).map(|_| emucore::cpu::CpuStep {
            cycles: (self.cycle - before) as u64,
            instruction: self.instructions,
        })
    }

    fn stats(&self) -> emucore::cpu::ExecutionStats {
        emucore::cpu::ExecutionStats {
            cycles: self.cycle as u64,
            instructions: self.instructions,
        }
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
        Self::new(0, 0, 0)
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
        if self.wai {
            self.wai = false;
        } else {
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
        }

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
        let mut checker = NoopEffects;
        self.step_with(&mut checker)
    }

    pub fn step_with<C: EffectsChecker>(&mut self, checker: &mut C) -> CpuResult<StepResult> {
        self.instructions += 1;
        const IRQ_VEC: usize = 0xfff8;
        const SWI_VEC: usize = 0xfffa;
        const NMI_VEC: usize = 0xfffc;
        const RESET_VEC: usize = 0xfffe;

        use super::addrmodes::*;
        let cycle = self.cycle;

        let pc = self.regs.pc() as usize;

        use CpuState::*;

        match self.get_cpu_state() {
            NmiPending => {
                let pc = self.interrupt(NMI_VEC)?;
                self.nmi = false;
                self.cycle += 1;
                Ok(StepResult::Nmi(pc))
            }
            IrqPending => {
                let pc = self.interrupt(IRQ_VEC)?;
                self.irq = false;
                self.cycle += 1;
                Ok(StepResult::Irq(pc))
            }

            ResetPending => {
                self.reset = false;
                let v = self.mem_mut().load_word(RESET_VEC)?;
                self.regs.set_pc(v);
                self.regs.sei();
                self.cycle += 1;
                Ok(StepResult::Reset(v.into()))
            }

            Waiting => {
                // A waiting MPU consumes a clock while it holds the bus. It
                // resumes when an eligible interrupt or reset is asserted.
                self.cycle += 1;
                let pc = self.regs.pc();
                Ok(StepResult::new(pc.into(), pc.into(), 1))
            }

            Running => {
                let addr = self.regs.pc();
                let op_code = self.mem_mut().load_byte(addr as usize)?;
                self.regs.inc_pc();
                let snapshot = checker.before(&mut self.regs, op_code);

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

                op_table!(op_code, {
                    panic!("NOT IMP PC: {:04x} {:02x}", addr, op_code)
                });

                checker.after(&mut self.regs, op_code, snapshot)?;

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
        self.wai = false;
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
            instructions: 0,
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
        Ok(lo as u16 | ((hi as u16) << 8))
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

#[cfg(test)]
mod tests {
    use super::{Machine, StepResult, VerifyEffects};
    use crate::cpu::{RegisterFile, RegisterFileTrait, StatusRegTrait};
    use emucore::byteorder::BigEndian;
    use emucore::mem::{MemBlock, MemoryIO};

    #[test]
    fn step_executes_immediate_ldaa() {
        let mut mem: MemBlock<BigEndian> = MemBlock::new("test", false, &(0..0x10000));
        mem.store_byte(0, 0x86).unwrap(); // LDAA #imm
        mem.store_byte(1, 0x42).unwrap();
        let mut regs = RegisterFile::default();
        regs.set_pc(0);
        let mut machine = Machine::new(mem, regs);

        let result = machine.step().unwrap();

        assert!(matches!(
            result,
            StepResult::Step {
                pc: 0,
                next_pc: 2,
                cycles: 2
            }
        ));
        assert_eq!(machine.regs.a(), 0x42);
        assert_eq!(machine.regs.pc(), 2);
    }

    #[test]
    fn tab_transfers_a_to_b() {
        let mut mem: MemBlock<BigEndian> = MemBlock::new("test", false, &(0..0x10000));
        mem.store_byte(0, 0x16).unwrap(); // TAB
        let mut regs = RegisterFile::default();
        regs.set_pc(0);
        regs.set_a(0x12);
        regs.set_b(0);
        let mut machine = Machine::new(mem, regs);

        machine.step().unwrap();

        assert_eq!(machine.regs.b(), 0x12);
        assert!(!machine.regs.z());
    }

    #[test]
    fn verify_effects_accepts_ldaa() {
        let mut mem: MemBlock<BigEndian> = MemBlock::new("test", false, &(0..0x10000));
        mem.store_byte(0, 0x86).unwrap(); // LDAA #imm
        mem.store_byte(1, 0x42).unwrap();
        let mut regs = RegisterFile::default();
        regs.set_pc(0);
        let mut machine = Machine::new(mem, regs);
        let mut checker = VerifyEffects;

        machine.step_with(&mut checker).unwrap();

        assert_eq!(machine.regs.a(), 0x42);
    }

    #[test]
    fn suba_updates_accumulator_with_result() {
        let mut mem: MemBlock<BigEndian> = MemBlock::new("test", false, &(0..0x10000));
        mem.store_byte(0, 0x80).unwrap(); // SUBA #imm
        mem.store_byte(1, 0x01).unwrap();
        let mut regs = RegisterFile::default();
        regs.set_pc(0);
        regs.set_a(3);
        let mut machine = Machine::new(mem, regs);

        machine.step().unwrap();

        assert_eq!(machine.regs.a(), 2);
    }

    #[test]
    fn cpx_only_updates_flags() {
        let mut mem: MemBlock<BigEndian> = MemBlock::new("test", false, &(0..0x10000));
        mem.store_byte(0, 0x8c).unwrap(); // CPX #imm
        mem.store_byte(1, 0x00).unwrap();
        mem.store_byte(2, 0x20).unwrap();
        let mut regs = RegisterFile::default();
        regs.set_pc(0);
        regs.set_x(0x1234);
        let mut machine = Machine::new(mem, regs);

        machine.step().unwrap();

        assert_eq!(machine.regs.x(), 0x1234);
        assert!(!machine.regs.z());
    }

    #[test]
    fn indexed_ldaa_uses_x_plus_offset() {
        let mut mem: MemBlock<BigEndian> = MemBlock::new("test", false, &(0..0x10000));
        mem.store_byte(0, 0xa6).unwrap(); // LDAA 0,X
        mem.store_byte(1, 0x00).unwrap();
        mem.store_byte(0x24, 0x6e).unwrap();
        mem.store_byte(0x26, 0xff).unwrap();
        let mut regs = RegisterFile::default();
        regs.set_pc(0);
        regs.set_x(0x26);
        let mut machine = Machine::new(mem, regs);

        machine.step().unwrap();

        assert_eq!(machine.regs.a(), 0xff);
    }

    #[test]
    fn jsr_rts_round_trip_preserves_return_address() {
        let mut mem: MemBlock<BigEndian> = MemBlock::new("test", false, &(0..0x10000));
        mem.store_byte(0, 0xbd).unwrap(); // JSR extended
        mem.store_byte(1, 0x00).unwrap();
        mem.store_byte(2, 0x06).unwrap();
        mem.store_byte(3, 0x01).unwrap(); // NOP after return
        mem.store_byte(6, 0x39).unwrap(); // RTS
        let mut regs = RegisterFile::default();
        regs.set_pc(0);
        regs.set_sp(0x7f);
        let mut machine = Machine::new(mem, regs);
        machine.step().unwrap();
        assert_eq!(machine.regs.pc(), 6);
        machine.step().unwrap();
        assert_eq!(machine.regs.pc(), 3);
    }

    #[test]
    fn reset_loads_reset_vector_and_sets_interrupt_mask() {
        let mut mem: MemBlock<BigEndian> = MemBlock::new("test", false, &(0..0x10000));
        mem.store_byte(0xfffe, 0x12).unwrap();
        mem.store_byte(0xffff, 0x34).unwrap();
        let regs = RegisterFile::default();
        let mut machine = Machine::new(mem, regs);
        machine.reset();

        let result = machine.step().unwrap();

        assert!(matches!(result, StepResult::Reset(0x1234)));
        assert_eq!(machine.regs.pc(), 0x1234);
        assert!(machine.regs.i());
    }

    #[test]
    fn wai_stacks_state_and_resumes_on_unmasked_irq() {
        let mut mem: MemBlock<BigEndian> = MemBlock::new("test", false, &(0..0x10000));
        mem.store_byte(0, 0x3e).unwrap(); // WAI
        mem.store_byte(0xfff8, 0x12).unwrap();
        mem.store_byte(0xfff9, 0x34).unwrap();
        let mut regs = RegisterFile::default();
        regs.set_pc(0);
        regs.set_sp(0x0100);
        let mut machine = Machine::new(mem, regs);

        let result = machine.step().unwrap();
        assert!(matches!(
            result,
            StepResult::Step {
                pc: 0,
                next_pc: 1,
                cycles: 9
            }
        ));
        assert!(machine.wai);
        assert_eq!(machine.regs.sp(), 0x00f9);

        let waiting = machine.step().unwrap();
        assert!(matches!(waiting, StepResult::Step { cycles: 1, .. }));
        machine.irq();
        let interrupt = machine.step().unwrap();
        assert!(matches!(interrupt, StepResult::Irq(0x1234)));
        assert!(!machine.wai);
        assert_eq!(machine.regs.sp(), 0x00f9);
    }
}
