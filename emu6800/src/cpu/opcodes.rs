use emucore::mem::MemoryIO;

use super::{Bus, RegEnum, RegisterFileTrait, StatusRegTrait};
use super::{CpuResult, Machine};
use crate::cpu_core::{AddrModeEnum, Isa, IsaDatabase};

lazy_static::lazy_static! {
    pub static ref ISA_DBASE : IsaDatabase = {
        let txt = include_str!("../../resources/opcodes6800.json");
        let isa: Isa = serde_json::from_str(txt).unwrap();
        IsaDatabase::new(&isa)
    };
}
////////////////////////////////////////////////////////////////////////////////
// Helpers

#[inline]
fn bool_as_u8(m: bool) -> u8 {
    if m {
        1u8
    } else {
        0u8
    }
}

pub struct Ins<'a, A, R, M>
where
    A: Bus,
    R: RegisterFileTrait + StatusRegTrait,
    M: MemoryIO,
{
    pub bus: A,
    pub m: &'a mut Machine<M, R>,
}

////////////////////////////////////////////////////////////////////////////////
// Utils
impl<'a, A, R, M> Ins<'a, A, R, M>
where
    A: Bus,
    R: RegisterFileTrait + StatusRegTrait,
    M: MemoryIO,
{
    pub fn new(bus: A, m: &'a mut Machine<M, R>) -> Self {
        Self { bus, m }
    }

    #[inline]
    fn fetch_operand(&mut self) -> CpuResult<u8> {
        let r = A::fetch_operand(self.m)?;
        Ok(r)
    }
    #[inline]
    fn fetch_operand_16(&mut self) -> CpuResult<u16> {
        let r = A::fetch_operand_16(self.m)?;
        Ok(r)
    }
}

////////////////////////////////////////////////////////////////////////////////
// Relative branches
impl<'a, A, R, M> Ins<'a, A, R, M>
where
    A: Bus,
    R: RegisterFileTrait + StatusRegTrait,
    M: MemoryIO,
{
    #[inline]
    fn branch_cond(&mut self, cond: bool) -> CpuResult<()> {
        let addr = A::fetch_operand_16(self.m)?;
        if cond {
            self.m.regs.set_pc(addr);
        }
        Ok(())
    }

    #[inline]
    pub fn bra(&mut self) -> CpuResult<()> {
        self.branch_cond(true)
    }

    #[inline]
    pub fn bpl(&mut self) -> CpuResult<()> {
        let n = self.m.regs.n();
        self.branch_cond(!n)
    }
    #[inline]
    pub fn blt(&mut self) -> CpuResult<()> {
        let lt = self.m.regs.lt();
        self.branch_cond(lt)
    }
    #[inline]
    pub fn bmi(&mut self) -> CpuResult<()> {
        let n = self.m.regs.n();
        self.branch_cond(n)
    }

    #[inline]
    pub fn bne(&mut self) -> CpuResult<()> {
        let z = self.m.regs.z();
        self.branch_cond(!z)
    }

    #[inline]
    pub fn beq(&mut self) -> CpuResult<()> {
        let z = self.m.regs.z();
        self.branch_cond(z)
    }

    #[inline]
    pub fn bhi(&mut self) -> CpuResult<()> {
        let hi = self.m.regs.hi();
        self.branch_cond(hi)
    }

    #[inline]
    pub fn bgt(&mut self) -> CpuResult<()> {
        let gt = self.m.regs.gt();
        self.branch_cond(gt)
    }

    #[inline]
    pub fn ble(&mut self) -> CpuResult<()> {
        let le = self.m.regs.le();
        self.branch_cond(le)
    }

    #[inline]
    pub fn bls(&mut self) -> CpuResult<()> {
        let ls = self.m.regs.ls();
        self.branch_cond(ls)
    }

    #[inline]
    pub fn bge(&mut self) -> CpuResult<()> {
        let ge = self.m.regs.ge();
        self.branch_cond(ge)
    }

    #[inline]
    pub fn bcs(&mut self) -> CpuResult<()> {
        let c = self.m.regs.c();
        self.branch_cond(c)
    }

    #[inline]
    pub fn bcc(&mut self) -> CpuResult<()> {
        let c = self.m.regs.c();
        self.branch_cond(!c)
    }

    #[inline]
    pub fn bvs(&mut self) -> CpuResult<()> {
        let v = self.m.regs.v();
        self.branch_cond(v)
    }

    #[inline]
    pub fn bvc(&mut self) -> CpuResult<()> {
        let v = self.m.regs.v();
        self.branch_cond(!v)
    }
}

////////////////////////////////////////////////////////////////////////////////
// Flags
impl<'a, A, R, M> Ins<'a, A, R, M>
where
    A: Bus,
    R: RegisterFileTrait + StatusRegTrait,
    M: MemoryIO,
{
    #[inline]
    pub fn clc(&mut self) -> CpuResult<()> {
        self.m.regs.clc();
        Ok(())
    }

    #[inline]
    pub fn sec(&mut self) -> CpuResult<()> {
        self.m.regs.sec();
        Ok(())
    }

    #[inline]
    pub fn clv(&mut self) -> CpuResult<()> {
        self.m.regs.clv();
        Ok(())
    }

    #[inline]
    pub fn sev(&mut self) -> CpuResult<()> {
        self.m.regs.sev();
        Ok(())
    }

    #[inline]
    pub fn cli(&mut self) -> CpuResult<()> {
        self.m.regs.cli();
        Ok(())
    }
    #[inline]
    pub fn sei(&mut self) -> CpuResult<()> {
        self.m.regs.sei();
        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////
// Stack stuff
impl<'a, A, R, M> Ins<'a, A, R, M>
where
    A: Bus,
    R: RegisterFileTrait + StatusRegTrait,
    M: MemoryIO,
{
    #[inline]
    pub fn pul(&mut self) -> CpuResult<()> {
        let val = self.m.pop_byte()?;
        A::store_byte(self.m, val)?;
        Ok(())
    }

    #[inline]
    pub fn psh(&mut self) -> CpuResult<()> {
        let val = self.fetch_operand()?;
        self.m.push_byte(val)?;
        Ok(())
    }

    #[inline]
    /// Increment stack ptr
    pub fn ins(&mut self) -> CpuResult<()> {
        let r = self.regs_mut();
        let sp = r.sp();
        r.set_sp(sp.wrapping_add(1));
        Ok(())
    }

    #[inline]
    /// Decrement stack ptr
    pub fn des(&mut self) -> CpuResult<()> {
        let r = self.regs_mut();
        let sp = r.sp();
        r.set_sp(sp.wrapping_sub(1));
        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////
// Misc
impl<'a, A, R, M> Ins<'a, A, R, M>
where
    A: Bus,
    R: RegisterFileTrait + StatusRegTrait,
    M: MemoryIO,
{
    #[inline]
    pub fn nop(&mut self) -> CpuResult<()> {
        Ok(())
    }
    #[inline]
    pub fn wai(&mut self) -> CpuResult<()> {
        panic!()
    }
}

////////////////////////////////////////////////////////////////////////////////

////////////////////////////////////////////////////////////////////////////////
// jmp / sub / returns
impl<'a, A, R, M> Ins<'a, A, R, M>
where
    A: Bus,
    R: RegisterFileTrait + StatusRegTrait,
    M: MemoryIO,
{
    #[inline]
    pub fn rti(&mut self) -> CpuResult<()> {
        let sr = self.m.pop_byte()?;
        let b = self.m.pop_byte()?;
        let a = self.m.pop_byte()?;
        let x = self.m.pop_word()?;
        let pc = self.m.pop_word()?;

        let regs = self.regs_mut();

        regs.set_sr(sr).set_a(a).set_b(b).set_x(x).set_pc(pc);

        Ok(())
    }

    #[inline]
    pub fn jmp(&mut self) -> CpuResult<()> {
        let addr = A::fetch_effective_address(self.m)?;
        self.m.regs.set_pc(addr);
        Ok(())
    }

    #[inline]
    pub fn bsr(&mut self) -> CpuResult<()> {
        let addr = A::fetch_operand_16(self.m)?;
        let pc = self.m.regs.pc();
        self.m.push_word(pc)?;
        self.m.regs.set_pc(addr);
        Ok(())
    }

    #[inline]
    pub fn swi(&mut self) -> CpuResult<()> {
        let pc = self.m.regs.pc();
        let x = self.m.regs.x();
        let a = self.m.regs.a();
        let b = self.m.regs.b();
        let sr = self.m.regs.sr();

        self.m.push_word(pc)?;
        self.m.push_word(x)?;
        self.m.push_byte(a)?;
        self.m.push_byte(b)?;
        self.m.push_byte(sr)?;

        let addr = self.m.mem_mut().load_word(0xfffa)?;
        let regs = self.regs_mut();
        regs.set_pc(addr);

        Ok(())
    }

    #[inline]
    pub fn jsr(&mut self) -> CpuResult<()> {
        let addr = A::fetch_effective_address(self.m)?;
        let pc = self.m.regs.pc();
        self.m.push_word(pc)?;
        self.m.regs.set_pc(addr);
        Ok(())
    }

    #[inline]
    pub fn rts(&mut self) -> CpuResult<()> {
        let addr = self.m.pop_word()?;
        self.m.regs.set_pc(addr);
        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////
// bool logic
impl<'a, A, R, M> Ins<'a, A, R, M>
where
    A: Bus,
    R: RegisterFileTrait + StatusRegTrait,
    M: MemoryIO,
{
    #[inline]
    fn eor_val(&mut self, val: u8) -> CpuResult<u8> {
        let operand = self.m.fetch_byte()?;
        let res = val ^ operand;
        self.m.regs.set_nz_from_u8(res);
        self.m.regs.clv();
        Ok(res)
    }

    fn and_val(&mut self, val: u8) -> CpuResult<u8> {
        let operand = self.m.fetch_byte()?;
        let res = val & operand;
        self.m.regs.set_nz_from_u8(res);
        self.m.regs.clv();
        Ok(res)
    }

    fn or_val(&mut self, val: u8) -> CpuResult<u8> {
        let res = val | self.m.fetch_byte()?;
        self.m.regs.set_nz_from_u8(res);
        self.m.regs.clv();
        Ok(res)
    }

    #[inline]
    pub fn com(&mut self) -> CpuResult<()> {
        let (_, new) = A::read_mod_write(self.m, |v| !v)?;
        let regs = self.regs_mut();
        regs.sec().clv().set_nz_from_u8(new);
        Ok(())
    }

    #[inline]
    pub fn eora(&mut self) -> CpuResult<()> {
        let a = self.m.regs.get_reg_8(RegEnum::A);
        let new_a = self.eor_val(a)?;
        self.m.regs.set_reg_8(RegEnum::A, new_a);
        Ok(())
    }

    #[inline]
    pub fn eorb(&mut self) -> CpuResult<()> {
        let b = self.m.regs.get_reg_8(RegEnum::B);
        let new_b = self.eor_val(b)?;
        self.m.regs.set_reg_8(RegEnum::B, new_b);
        Ok(())
    }

    #[inline]
    pub fn anda(&mut self) -> CpuResult<()> {
        let a = self.m.regs.get_reg_8(RegEnum::A);
        let new_a = self.and_val(a)?;
        self.m.regs.set_reg_8(RegEnum::A, new_a);
        Ok(())
    }

    #[inline]
    pub fn andb(&mut self) -> CpuResult<()> {
        let b = self.m.regs.get_reg_8(RegEnum::B);
        let new_b = self.and_val(b)?;
        self.m.regs.set_reg_8(RegEnum::B, new_b);
        Ok(())
    }

    #[inline]
    pub fn oraa(&mut self) -> CpuResult<()> {
        let a = self.m.regs.get_reg_8(RegEnum::A);
        let new_a = self.or_val(a)?;
        self.m.regs.set_reg_8(RegEnum::A, new_a);
        Ok(())
    }

    #[inline]
    pub fn orab(&mut self) -> CpuResult<()> {
        let b = self.m.regs.get_reg_8(RegEnum::B);
        let new_b = self.or_val(b)?;
        self.m.regs.set_reg_8(RegEnum::B, new_b);
        Ok(())
    }
}

pub trait UnsignedVal {
    fn is_neg(&self) -> bool;
    fn bit(&self, v: u8) -> bool;
}

impl UnsignedVal for u8 {
    fn is_neg(&self) -> bool {
        *self & 0x80 == 0x80
    }

    fn bit(&self, v: u8) -> bool {
        if v < 8 {
            self & (1 << v) != 0
        } else {
            false
        }
    }
}

impl UnsignedVal for u16 {
    fn is_neg(&self) -> bool {
        *self & 0x8000 == 0x8000
    }

    fn bit(&self, v: u8) -> bool {
        if v < 16 {
            self & (1 << v) == 0
        } else {
            false
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// Adds, subs
impl<'a, A, R, M> Ins<'a, A, R, M>
where
    A: Bus,
    R: RegisterFileTrait + StatusRegTrait,
    M: MemoryIO,
{
    fn do_sub(&mut self, c: bool, val: u8, op: u8) -> CpuResult<u8> {
        let new_val = val.wrapping_sub(op).wrapping_sub(bool_as_u8(c));

        let n = new_val.is_neg();
        let z = new_val == 0;
        let v = !n;
        let c = new_val > val;

        self.m.regs.set_n(n);
        self.m.regs.set_z(z);
        self.m.regs.set_v(v);
        self.m.regs.set_c(c);

        Ok(val)
    }

    fn do_add(&mut self, c: bool, val: u8, op: u8) -> CpuResult<u8> {
        let new_val = val.wrapping_add(op).wrapping_add(bool_as_u8(c));

        let n = new_val.is_neg();
        let z = new_val == 0;
        let v = new_val.is_neg() != val.is_neg();
        let c = new_val > val;

        self.m.regs.set_n(n);
        self.m.regs.set_z(z);
        self.m.regs.set_v(v);
        self.m.regs.set_c(c);

        Ok(val)
    }

    #[inline]
    pub fn neg(&mut self) -> CpuResult<()> {
        let val = self.m.fetch_byte()? as i8;
        let new_val = -val;
        A::store_byte(self.m, new_val as u8)?;
        panic!()
    }

    #[inline]
    pub fn cmpa(&mut self) -> CpuResult<()> {
        let op = self.fetch_operand()?;
        let a = self.m.regs.a();
        let _ = self.do_sub(false, a, op)?;
        Ok(())
    }

    #[inline]
    pub fn cmpb(&mut self) -> CpuResult<()> {
        let op = self.fetch_operand()?;
        let b = self.m.regs.b();
        let _ = self.do_sub(false, b, op)?;
        Ok(())
    }

    #[inline]
    pub fn addb(&mut self) -> CpuResult<()> {
        let op = self.fetch_operand()?;
        let b = self.m.regs.b();
        let b = self.do_add(false, b, op)?;
        self.regs_mut().set_b(b);
        Ok(())
    }

    #[inline]
    pub fn adcb(&mut self) -> CpuResult<()> {
        let op = self.fetch_operand()?;
        let b = self.m.regs.b();
        let c = self.m.regs.c();
        let new_b = self.do_add(c, b, op)?;
        self.m.regs.set_b(new_b);
        Ok(())
    }

    #[inline]
    pub fn adda(&mut self) -> CpuResult<()> {
        let op = A::fetch_operand(self.m)?;
        let a = self.m.regs.a();
        let new_a = self.do_add(false, a, op)?;
        self.m.regs.set_a(new_a);
        Ok(())
    }
    #[inline]
    pub fn adca(&mut self) -> CpuResult<()> {
        let op = self.fetch_operand()?;
        let a = self.m.regs.a();
        let c = self.m.regs.c();
        let a = self.do_add(c, a, op)?;
        self.m.regs.set_a(a);
        Ok(())
    }

    #[inline]
    pub fn aba(&mut self) -> CpuResult<()> {
        let a = self.m.regs.a();
        let b = self.m.regs.b();
        let a = self.do_add(false, a, b)?;
        self.m.regs.set_a(a);
        Ok(())
    }

    #[inline]
    pub fn sba(&mut self) -> CpuResult<()> {
        let a = self.m.regs.a();
        let b = self.m.regs.b();
        let a = self.do_sub(false, a, b)?;
        self.m.regs.set_a(a);
        Ok(())
    }

    #[inline]
    pub fn suba(&mut self) -> CpuResult<()> {
        self.sub_reg(false, RegEnum::A)
    }

    #[inline]
    pub fn subb(&mut self) -> CpuResult<()> {
        self.sub_reg(false, RegEnum::B)
    }

    #[inline]
    pub fn sbca(&mut self) -> CpuResult<()> {
        let c = self.m.regs.c();
        self.sub_reg(c, RegEnum::A)
    }
    #[inline]
    pub fn sbcb(&mut self) -> CpuResult<()> {
        let c = self.m.regs.c();
        self.sub_reg(c, RegEnum::B)
    }

    #[inline]
    pub fn sub_reg(&mut self, c: bool, reg: RegEnum) -> CpuResult<()> {
        let op = self.fetch_operand()?;
        let r = self.m.regs.get_reg_8(reg);
        let v = self.do_sub(c, r, op)?;
        self.m.regs.set_reg_8(reg, v);
        Ok(())
    }

    #[inline]
    pub fn cpx(&mut self) -> CpuResult<()> {
        let op = self.fetch_operand_16()?;
        let x = self.m.regs.x();
        let new_x = x.wrapping_sub(op);
        self.m.regs.set_x(new_x);

        let n = new_x.is_neg();
        let z = new_x == 0;
        let v = new_x.is_neg() != x.is_neg();

        self.m.regs.set_n(n);
        self.m.regs.set_z(z);
        self.m.regs.set_z(v);

        Ok(())
    }

    #[inline]
    pub fn cba(&mut self) -> CpuResult<()> {
        let a = self.m.regs.a();
        let b = self.m.regs.b();
        let _ = self.do_sub(false, a, b)?;
        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////
// inc / dec
impl<'a, A, R, M> Ins<'a, A, R, M>
where
    A: Bus,
    R: RegisterFileTrait + StatusRegTrait,
    M: MemoryIO,
{
    #[inline]
    pub fn inx(&mut self) -> CpuResult<()> {
        let x = self.m.regs.x().wrapping_add(1);
        // dex only sets zero flag - no bpl/bmu x loops on 6800
        self.m.regs.set_z(x == 0);
        self.m.regs.set_x(x);
        Ok(())
    }

    #[inline]
    pub fn dex(&mut self) -> CpuResult<()> {
        let x = self.m.regs.x().wrapping_sub(1);
        // dex only sets zero flag - no bpl/bmi x loops on 6800
        self.m.regs.set_z(x == 0);
        self.m.regs.set_x(x);
        Ok(())
    }

    #[inline]
    pub fn inc(&mut self) -> CpuResult<()> {
        let (old, new) = A::read_mod_write(self.m, |v| v.wrapping_add(1))?;
        self.m.regs.set_nz_from_u8(new);
        self.m.regs.set_v(new < old);
        Ok(())
    }

    #[inline]
    pub fn dec(&mut self) -> CpuResult<()> {
        let (old, new) = A::read_mod_write(self.m, |v| v.wrapping_sub(1))?;
        self.m.regs.set_nz_from_u8(new);
        self.m.regs.set_v(new > old);
        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////
// Shifts
//
impl<'a, A, R, M> Ins<'a, A, R, M>
where
    A: Bus,
    R: RegisterFileTrait + StatusRegTrait,
    M: MemoryIO,
{
    #[inline]
    fn post_shift(&mut self, c: bool, val: u8, new_val: u8) -> CpuResult<()> {
        A::store_byte(self.m, new_val)?;
        let v = val.is_neg() != new_val.is_neg();
        self.m.regs.set_c(c);
        self.m.regs.set_nz_from_u8(new_val);
        self.m.regs.set_v(v);
        Ok(())
    }

    #[inline]
    pub fn asr(&mut self) -> CpuResult<()> {
        let val = self.fetch_operand()?;
        let new_val = val.wrapping_shr(1) | if val.is_neg() { 1 << 7 } else { 0 };
        self.post_shift(val.bit(7), val, new_val)
    }

    #[inline]
    pub fn asl(&mut self) -> CpuResult<()> {
        let val = self.fetch_operand()?;
        let new_val = val.wrapping_shl(1);
        let c = val.is_neg();
        self.post_shift(c, val, new_val)
    }

    #[inline]
    pub fn lsr(&mut self) -> CpuResult<()> {
        let val = self.fetch_operand()?;
        let new_val = val.wrapping_shr(1);
        let c = val.bit(0);
        self.post_shift(c, val, new_val)
    }

    #[inline]
    pub fn ror(&mut self) -> CpuResult<()> {
        let val = self.fetch_operand()?;
        let new_val = val.wrapping_shr(1) | if self.m.regs.c() { 1 << 7 } else { 0 };
        self.post_shift(val.bit(0), val, new_val)
    }

    #[inline]
    pub fn rol(&mut self) -> CpuResult<()> {
        let val = self.fetch_operand()?;
        let new_val = val.wrapping_shl(1) | if self.m.regs.c() { 1 } else { 0 };
        self.post_shift(val.bit(1), val, new_val)
    }
}

////////////////////////////////////////////////////////////////////////////////
// Transfers
impl<'a, A, R, M> Ins<'a, A, R, M>
where
    A: Bus,
    R: RegisterFileTrait + StatusRegTrait,
    M: MemoryIO,
{
    #[inline]
    pub fn tpa(&mut self) -> CpuResult<()> {
        let regs = self.regs_mut();
        let sr = regs.sr();
        regs.set_a(sr);
        Ok(())
    }
    #[inline]
    pub fn tap(&mut self) -> CpuResult<()> {
        let regs = self.regs_mut();
        let a = regs.a();
        regs.set_sr(a);
        Ok(())
    }
    #[inline]
    pub fn tba(&mut self) -> CpuResult<()> {
        let regs = &mut self.m.regs;
        let b = regs.b();
        regs.set_a(b).set_nz_from_u8(b).clv();
        Ok(())
    }

    #[inline]
    pub fn tab(&mut self) -> CpuResult<()> {
        let regs = &mut self.m.regs;
        let b = regs.b();
        regs.set_b(b);
        regs.set_nz_from_u8(b).clv();
        Ok(())
    }

    #[inline]
    pub fn tsx(&mut self) -> CpuResult<()> {
        let regs = &mut self.m.regs;
        let sp = regs.sp();
        regs.set_x(sp);
        Ok(())
    }
    #[inline]
    pub fn txs(&mut self) -> CpuResult<()> {
        let regs = &mut self.m.regs;
        let x = regs.x();
        regs.set_sp(x);
        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////
// Load / stores
impl<'a, A, R, M> Ins<'a, A, R, M>
where
    A: Bus,
    R: RegisterFileTrait + StatusRegTrait,
    M: MemoryIO,
{
    fn regs_mut(&mut self) -> &mut R {
        &mut self.m.regs
    }
    fn regs(&self) -> &R {
        &self.m.regs
    }

    fn fetch_operand_8_fl(&mut self) -> CpuResult<u8> {
        let val = self.fetch_operand()?;
        let regs = self.regs_mut();
        regs.set_nz_from_u8(val);
        regs.clc();
        Ok(val)
    }

    fn fetch_operand_16_fl(&mut self) -> CpuResult<u16> {
        let val = self.fetch_operand_16()?;
        let regs = self.regs_mut();
        regs.set_nz_from_u16(val);
        regs.clc();
        Ok(val)
    }

    #[inline]
    pub fn lds(&mut self) -> CpuResult<()> {
        let sp = self.fetch_operand_16_fl()?;
        self.m.regs.set_sp(sp);
        Ok(())
    }

    #[inline]
    pub fn ldaa(&mut self) -> CpuResult<()> {
        let a = self.fetch_operand_8_fl()?;
        self.regs_mut().set_a(a);
        Ok(())
    }

    #[inline]
    pub fn ldab(&mut self) -> CpuResult<()> {
        let r = self.fetch_operand_8_fl()?;
        self.regs_mut().set_b(r);
        Ok(())
    }

    #[inline]
    pub fn ldx(&mut self) -> CpuResult<()> {
        let r = self.fetch_operand_16_fl()?;
        self.regs_mut().set_x(r);
        Ok(())
    }

    #[inline]
    fn st_mem_8(&mut self, val: u8) -> CpuResult<()> {
        self.regs_mut().set_nz_from_u8(val).clv();
        let addr = A::fetch_effective_address(self.m)?;
        self.m.mem_mut().store_byte(addr as usize, val)?;
        Ok(())
    }

    #[inline]
    fn st_mem_16(&mut self, val: u16) -> CpuResult<()> {
        self.regs_mut().set_nz_from_u16(val).clv();
        let addr = A::fetch_effective_address(self.m)?;
        self.m.mem_mut().store_word(addr as usize, val)?;
        Ok(())
    }

    #[inline]
    pub fn staa(&mut self) -> CpuResult<()> {
        let a = self.m.regs.a();
        self.st_mem_8(a)
    }

    #[inline]
    pub fn stab(&mut self) -> CpuResult<()> {
        let b = self.m.regs.b();
        self.st_mem_8(b)
    }

    #[inline]
    pub fn sts(&mut self) -> CpuResult<()> {
        let sp = self.m.regs.sp();
        self.st_mem_16(sp)
    }
    #[inline]
    pub fn stx(&mut self) -> CpuResult<()> {
        let x = self.m.regs.x();
        self.st_mem_16(x)
    }
}

////////////////////////////////////////////////////////////////////////////////
impl<'a, A, R, M> Ins<'a, A, R, M>
where
    A: Bus,
    R: RegisterFileTrait + StatusRegTrait,
    M: MemoryIO,
{
    #[inline]
    pub fn clr(&mut self) -> CpuResult<()> {
        let regs = self.regs_mut();
        regs.cln().clc().clv().sez();
        A::store_byte(self.m, 0)?;
        Ok(())
    }

    #[inline]
    fn do_bit(&mut self, val: u8) -> CpuResult<()> {
        let operand = self.fetch_operand()?;
        let new_val = val & operand;
        let regs = self.regs_mut();
        regs.set_nz_from_u8(new_val).clc();
        A::store_byte(self.m, new_val)?;
        Ok(())
    }

    #[inline]
    pub fn bita(&mut self) -> CpuResult<()> {
        let val = self.regs_mut().a();
        self.do_bit(val)
    }

    #[inline]
    pub fn bitb(&mut self) -> CpuResult<()> {
        let val = self.regs_mut().a();
        self.do_bit(val)
    }

    #[inline]
    pub fn daa(&mut self) -> CpuResult<()> {
        panic!()
    }

    #[inline]
    pub fn tst(&mut self) -> CpuResult<()> {
        let val = self.fetch_operand()?;
        let regs = self.regs_mut();
        regs.set_nz_from_u8(val).clv().clc();
        Ok(())
    }
}
