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

                // Every opcode is now covered by the v2 table, so the
                // fallback arm is unreachable; keep it as a safety net
                // (the generated macro allows unreachable patterns).
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
    fn undocumented_opcodes_execute_without_panicking() {
        // Every opcode must decode and execute (v2 table: MAME-verified
        // undocumented behaviors).  Illegal00 (0x00) is a 1-byte NOP;
        // Illegal61 (0x61) skips one operand byte; Brn (0x21) skips the
        // relative operand without branching; 0x9D is JSR direct-page.
        let program = [0x00u8, 0x61, 0xaa, 0x21, 0x01, 0x9d, 0x10];
        let mut mem: MemBlock<BigEndian> = MemBlock::new("test", false, &(0..0x10000));
        for (i, b) in program.iter().enumerate() {
            mem.store_byte(i, *b).unwrap();
        }
        let mut regs = RegisterFile::default();
        regs.set_pc(0);
        regs.set_sp(0xff00);
        let mut machine = Machine::new(mem, regs);
        let cycles_of = |r: &StepResult| match r {
            StepResult::Step { cycles, .. } => *cycles,
            _ => 0,
        };
        // 0x00: 1-byte NOP (4 cycles), pc -> 1.
        assert_eq!(cycles_of(&machine.step().unwrap()), 4);
        assert_eq!(machine.regs.pc(), 1);
        // 0x61: 2-byte NOP (4 cycles), operand skipped, pc -> 3.
        assert_eq!(cycles_of(&machine.step().unwrap()), 4);
        assert_eq!(machine.regs.pc(), 3);
        // 0x21 BRN: no branch, operand skipped, pc -> 5.
        assert_eq!(cycles_of(&machine.step().unwrap()), 4);
        assert_eq!(machine.regs.pc(), 5);
        // 0x9D JSR direct-page: pc -> 0x0010, return pushed.
        let sp_before = machine.regs.sp();
        assert_eq!(cycles_of(&machine.step().unwrap()), 6);
        assert_eq!(machine.regs.pc(), 0x10);
        assert_eq!(machine.regs.sp(), sp_before - 2);
    }

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
    fn cpx_n_v_reflect_upper_byte_only_and_c_is_preserved() {
        // Real 6800 quirk: N/V come from the high-byte comparison only;
        // Z from the full 16-bit equality; C untouched.
        // X=0x0025 vs #0x006C: 16-bit result is negative, but the upper
        // bytes are equal, so N=0 (not 1), V=0, Z=0.
        let mut mem: MemBlock<BigEndian> = MemBlock::new("test", false, &(0..0x10000));
        mem.store_byte(0, 0x8c).unwrap(); // CPX #imm
        mem.store_byte(1, 0x00).unwrap();
        mem.store_byte(2, 0x6c).unwrap();
        let mut regs = RegisterFile::default();
        regs.set_pc(0);
        regs.set_x(0x0025);
        regs.set_c(true);
        let mut machine = Machine::new(mem, regs);

        machine.step().unwrap();

        assert!(!machine.regs.n());
        assert!(!machine.regs.z());
        assert!(!machine.regs.v());
        assert!(machine.regs.c()); // carry preserved
    }

    #[test]
    fn cpx_sets_n_from_upper_byte_and_z_from_equality() {
        let mut mem: MemBlock<BigEndian> = MemBlock::new("test", false, &(0..0x10000));
        mem.store_byte(0, 0x8c).unwrap(); // CPX #imm
        mem.store_byte(1, 0x00).unwrap();
        mem.store_byte(2, 0x6c).unwrap();
        let mut regs = RegisterFile::default();
        regs.set_pc(0);
        regs.set_x(0x006c);
        let mut machine = Machine::new(mem, regs);

        machine.step().unwrap();

        assert!(machine.regs.z()); // full equality
        assert!(!machine.regs.n());
        assert!(!machine.regs.v());
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

    #[test]
    fn negb_negates_b_and_sets_flags() {
        // LDAB #$05; NEGB
        let mut mem: MemBlock<BigEndian> = MemBlock::new("test", false, &(0..0x10000));
        for (i, b) in [0xC6u8, 0x05, 0x50].iter().enumerate() {
            mem.store_byte(i, *b).unwrap();
        }
        let mut regs = RegisterFile::default();
        regs.set_pc(0);
        let mut machine = Machine::new(mem, regs);
        machine.step().unwrap(); // LDAB #5
        machine.step().unwrap(); // NEGB
        assert_eq!(machine.regs.b(), 0xFB);
        assert!(machine.regs.n());
        assert!(!machine.regs.z());
        assert!(!machine.regs.v());
        assert!(machine.regs.c());

        // Negating 0x80 overflows (V) and still borrows (C).
        machine.mem.store_byte(1, 0x80).unwrap();
        machine.regs.set_pc(0);
        machine.step().unwrap();
        machine.step().unwrap();
        assert_eq!(machine.regs.b(), 0x80);
        assert!(machine.regs.v());
        assert!(machine.regs.c());

        // Negating zero: Z set, no borrow.
        machine.mem.store_byte(1, 0x00).unwrap();
        machine.regs.set_pc(0);
        machine.step().unwrap();
        machine.step().unwrap();
        assert_eq!(machine.regs.b(), 0x00);
        assert!(machine.regs.z());
        assert!(!machine.regs.c());
    }

    #[test]
    fn daa_adjusts_bcd_sum() {
        // LDAA #$99; ADDA #$01; DAA  ->  A = $00, C = 1, Z = 1
        let mut mem: MemBlock<BigEndian> = MemBlock::new("test", false, &(0..0x10000));
        for (i, b) in [0x86u8, 0x99, 0x8B, 0x01, 0x19].iter().enumerate() {
            mem.store_byte(i, *b).unwrap();
        }
        let mut regs = RegisterFile::default();
        regs.set_pc(0);
        let mut machine = Machine::new(mem, regs);
        machine.step().unwrap(); // LDAA #$99
        machine.step().unwrap(); // ADDA #$01 -> A=$9A (no half-carry)
        assert!(!machine.regs.h());
        machine.step().unwrap(); // DAA
        assert_eq!(machine.regs.a(), 0x00);
        assert!(machine.regs.c());
        assert!(machine.regs.z());

        // LDAA #$09; ADDA #$08; DAA -> A=$17, H set by the add, C=0.
        for (i, b) in [0x86u8, 0x09, 0x8B, 0x08, 0x19].iter().enumerate() {
            machine.mem.store_byte(i, *b).unwrap();
        }
        machine.regs.set_pc(0);
        machine.step().unwrap(); // LDAA #$09
        machine.step().unwrap(); // ADDA #$08 -> A=$11, H=1
        assert!(machine.regs.h());
        machine.step().unwrap(); // DAA
        assert_eq!(machine.regs.a(), 0x17);
        assert!(!machine.regs.c());
        assert!(!machine.regs.z());
    }

    #[test]
    fn inca_sets_v_only_for_7f_to_80() {
        // LDAA #$FF; INCA -> A=$00: V must stay clear (6800 INC only
        // flags the $7F -> $80 overflow, MAME m6800 SET_V8 semantics).
        let mut mem: MemBlock<BigEndian> = MemBlock::new("test", false, &(0..0x10000));
        for (i, b) in [0x86u8, 0xff, 0x4c].iter().enumerate() {
            mem.store_byte(i, *b).unwrap();
        }
        let mut regs = RegisterFile::default();
        regs.set_pc(0);
        let mut machine = Machine::new(mem, regs);
        machine.step().unwrap(); // LDAA #$FF
        machine.step().unwrap(); // INCA
        assert_eq!(machine.regs.a(), 0x00);
        assert!(machine.regs.z());
        assert!(!machine.regs.v());

        // LDAA #$7F; INCA -> A=$80 with V set.
        for (i, b) in [0x86u8, 0x7f, 0x4c].iter().enumerate() {
            machine.mem.store_byte(i, *b).unwrap();
        }
        machine.regs.set_pc(0);
        machine.step().unwrap();
        machine.step().unwrap();
        assert_eq!(machine.regs.a(), 0x80);
        assert!(machine.regs.v());
    }

    #[test]
    fn deca_sets_v_only_for_80_to_7f() {
        // LDAA #$00; DECA -> A=$FF: V must stay clear (6800 DEC only
        // flags the $80 -> $7F step).
        let mut mem: MemBlock<BigEndian> = MemBlock::new("test", false, &(0..0x10000));
        for (i, b) in [0x86u8, 0x00, 0x4a].iter().enumerate() {
            mem.store_byte(i, *b).unwrap();
        }
        let mut regs = RegisterFile::default();
        regs.set_pc(0);
        let mut machine = Machine::new(mem, regs);
        machine.step().unwrap(); // LDAA #$00
        machine.step().unwrap(); // DECA
        assert_eq!(machine.regs.a(), 0xff);
        assert!(!machine.regs.v());

        // LDAA #$80; DECA -> A=$7F with V set.
        for (i, b) in [0x86u8, 0x80, 0x4a].iter().enumerate() {
            machine.mem.store_byte(i, *b).unwrap();
        }
        machine.regs.set_pc(0);
        machine.step().unwrap();
        machine.step().unwrap();
        assert_eq!(machine.regs.a(), 0x7f);
        assert!(machine.regs.v());
    }

    #[test]
    fn coma_sets_carry_and_clears_overflow() {
        // LDAA #$3F; COMA -> A=$C0, N=1, C=1, V=0 (the sound board's
        // volume-mask routine at $FC93 depends on C being set).
        let mut mem: MemBlock<BigEndian> = MemBlock::new("test", false, &(0..0x10000));
        for (i, b) in [0x86u8, 0x3f, 0x43].iter().enumerate() {
            mem.store_byte(i, *b).unwrap();
        }
        let mut regs = RegisterFile::default();
        regs.set_pc(0);
        let mut machine = Machine::new(mem, regs);
        machine.step().unwrap(); // LDAA #$3F
        machine.step().unwrap(); // COMA
        assert_eq!(machine.regs.a(), 0xc0);
        assert!(machine.regs.n());
        assert!(machine.regs.c());
        assert!(!machine.regs.v());
    }

    #[test]
    fn adda_overflow_flag_matches_m6800() {
        // LDAA #$FC; ADDA #$28 -> A=$24, C=1, V=0, H=1 (the sound
        // board's envelope math at $F885 depends on this).
        let mut mem: MemBlock<BigEndian> = MemBlock::new("test", false, &(0..0x10000));
        for (i, b) in [0x86u8, 0xfc, 0x8b, 0x28].iter().enumerate() {
            mem.store_byte(i, *b).unwrap();
        }
        let mut regs = RegisterFile::default();
        regs.set_pc(0);
        let mut machine = Machine::new(mem, regs);
        machine.step().unwrap(); // LDAA #$FC
        machine.step().unwrap(); // ADDA #$28
        assert_eq!(machine.regs.a(), 0x24);
        assert!(machine.regs.c());
        assert!(!machine.regs.v());
        assert!(machine.regs.h());
        assert!(!machine.regs.n());
        assert!(!machine.regs.z());
    }

    #[test]
    fn inc_indexed_is_a_read_modify_write() {
        // INC $00,X (opcode 6C): the sound board's routines hit the
        // indexed read-modify-write path (once unimplemented, it
        // panicked the emu thread at $FA37).
        let mut mem: MemBlock<BigEndian> = MemBlock::new("test", false, &(0..0x10000));
        for (i, b) in [0x6Cu8, 0x00].iter().enumerate() {
            mem.store_byte(i, *b).unwrap();
        }
        mem.store_byte(0x0040, 0x7f).unwrap();
        let mut regs = RegisterFile::default();
        regs.set_pc(0);
        regs.set_x(0x0040);
        let mut machine = Machine::new(mem, regs);
        machine.step().unwrap();
        assert_eq!(machine.mem.load_byte(0x0040).unwrap(), 0x80);
        assert!(machine.regs.v());
        assert!(!machine.regs.z());
    }
}
