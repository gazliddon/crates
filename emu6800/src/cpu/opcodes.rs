use emucore::mem::MemoryIO;
use emucore::sha1::digest::typenum::operator_aliases;

use super::{Bus, RegisterFileTrait, StatusRegTrait};
use super::{CpuResult, Machine};
use crate::cpu::{AccA, AccB};
use crate::cpu_core::{AddrModeEnum, Isa, IsaDatabase, RegEnum};

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
    pub fn pula(&mut self) -> CpuResult<()> {
        let val = self.m.pop_byte()?;
        self.m.regs.set_a(val);
        Ok(())
    }

    #[inline]
    pub fn pulb(&mut self) -> CpuResult<()> {
        let val = self.m.pop_byte()?;
        self.m.regs.set_b(val);
        Ok(())
    }

    #[inline]
    pub fn psha(&mut self) -> CpuResult<()> {
        let val = self.m.regs.a();
        self.m.push_byte(val)?;
        Ok(())
    }
    #[inline]
    pub fn pshb(&mut self) -> CpuResult<()> {
        let val = self.m.regs.b();
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

        self.regs_mut()
            .set_sr(sr)
            .set_a(a)
            .set_b(b)
            .set_x(x)
            .set_pc(pc);

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
    fn do_logic<REGBUS: Bus, F: Fn(u8, u8) -> u8>(&mut self, f: F) -> CpuResult<()> {
        let val = self.fetch_operand()?;
        let (_, res) = REGBUS::read_mod_write(self.m, |op| f(val, op))?;
        self.set_nz_from_u8(res).clv();
        Ok(())
    }

    #[inline]
    pub fn post_com(&mut self, new: u8) -> CpuResult<()> {
        self.set_nz_from_u8(new).clc().clv();
        Ok(())
    }

    #[inline]
    pub fn com(&mut self) -> CpuResult<()> {
        let (_, new) = A::read_mod_write(self.m, |v| !v)?;
        self.post_com(new)
    }

    #[inline]
    pub fn coma(&mut self) -> CpuResult<()> {
        let (_, new) = AccA::read_mod_write(self.m, |v| !v)?;
        self.post_com(new)
    }

    #[inline]
    pub fn comb(&mut self) -> CpuResult<()> {
        let (_, new) = AccB::read_mod_write(self.m, |v| !v)?;
        self.post_com(new)
    }

    #[inline]
    pub fn eora(&mut self) -> CpuResult<()> {
        self.do_logic::<AccA, _>(|a, b| a ^ b)
    }

    #[inline]
    pub fn eorb(&mut self) -> CpuResult<()> {
        self.do_logic::<AccB, _>(|a, b| a ^ b)
    }

    #[inline]
    pub fn anda(&mut self) -> CpuResult<()> {
        self.do_logic::<AccA, _>(|a, b| a & b)
    }

    #[inline]
    pub fn andb(&mut self) -> CpuResult<()> {
        self.do_logic::<AccB, _>(|a, b| a & b)
    }

    #[inline]
    pub fn oraa(&mut self) -> CpuResult<()> {
        self.do_logic::<AccA, _>(|a, b| a | b)
    }

    #[inline]
    pub fn orab(&mut self) -> CpuResult<()> {
        self.do_logic::<AccB, _>(|a, b| a | b)
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
    fn post_math(&mut self, c: bool, new_val: u8, val: u8) -> CpuResult<u8> {
        let n = new_val.is_neg();
        let z = new_val == 0;
        let v = c && (new_val.is_neg() != val.is_neg());
        self.m.regs.set_n(n);
        self.m.regs.set_z(z);
        self.m.regs.set_v(v);
        self.m.regs.set_c(c);
        Ok(val)
    }

    fn do_sub(&mut self, c: bool, val: u8, op: u8) -> CpuResult<u8> {
        let new_val = val.wrapping_sub(op).wrapping_sub(bool_as_u8(c));
        let c = new_val > val;
        self.post_math(c, new_val, val)
    }

    fn do_add(&mut self, c: bool, val: u8, op: u8) -> CpuResult<u8> {
        let new_val = val.wrapping_add(op).wrapping_add(bool_as_u8(c));
        let c = new_val < val;
        self.post_math(c, new_val, val)
    }

    #[inline]
    pub fn neg(&mut self) -> CpuResult<()> {
        let (_, new) = A::read_mod_write(self.m, |v| (-(v as i8)) as u8)?;
        A::store_byte(self.m, new as u8)?;
        panic!()
    }

    #[inline]
    pub fn nega(&mut self) -> CpuResult<()> {
        let (_, new) = AccA::read_mod_write(self.m, |v| (-(v as i8)) as u8)?;
        A::store_byte(self.m, new as u8)?;
        panic!()
    }
    #[inline]
    pub fn negb(&mut self) -> CpuResult<()> {
        let (_, new) = AccB::read_mod_write(self.m, |v| (-(v as i8)) as u8)?;
        A::store_byte(self.m, new as u8)?;
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
        self.sub_reg(self.m.regs.c(), RegEnum::A)
    }
    #[inline]
    pub fn sbcb(&mut self) -> CpuResult<()> {
        self.sub_reg(self.m.regs.c(), RegEnum::B)
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
    pub fn post_inc(&mut self, old: u8, new: u8) -> CpuResult<()> {
        self.m.regs.set_nz_from_u8(new);
        self.m.regs.set_v(new < old);
        Ok(())
    }

    #[inline]
    pub fn inc(&mut self) -> CpuResult<()> {
        let (old, new) = A::read_mod_write(self.m, |v| v.wrapping_add(1))?;
        self.post_inc(old, new)
    }

    #[inline]
    pub fn inca(&mut self) -> CpuResult<()> {
        let (old, new) = AccA::read_mod_write(self.m, |v| v.wrapping_add(1))?;
        self.post_inc(old, new)
    }
    #[inline]
    pub fn incb(&mut self) -> CpuResult<()> {
        let (old, new) = AccB::read_mod_write(self.m, |v| v.wrapping_add(1))?;
        self.post_inc(old, new)
    }

    #[inline]
    pub fn dec(&mut self) -> CpuResult<()> {
        let (old, new) = A::read_mod_write(self.m, |v| v.wrapping_sub(1))?;
        self.m.regs.set_nz_from_u8(new);
        self.m.regs.set_v(new > old);
        Ok(())
    }
    #[inline]
    pub fn deca(&mut self) -> CpuResult<()> {
        self.dec()
    }
    #[inline]
    pub fn decb(&mut self) -> CpuResult<()> {
        self.dec()
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

    pub fn do_asr<X: Bus>(&mut self) -> CpuResult<(u8, u8)> {
        let ret = X::read_mod_write(self.m, |val| {
            val.wrapping_shr(1) | if val.is_neg() { 1 << 7 } else { 0 }
        })?;

        Ok(ret)
    }

    #[inline]
    pub fn asr(&mut self) -> CpuResult<()> {
        let (val, new_val) = self.do_asr::<A>()?;
        self.post_shift(val.bit(7), val, new_val)
    }
    #[inline]
    pub fn asra(&mut self) -> CpuResult<()> {
        let (val, new_val) = self.do_asr::<AccA>()?;
        self.post_shift(val.bit(7), val, new_val)
    }

    #[inline]
    pub fn asrb(&mut self) -> CpuResult<()> {
        let (val, new_val) = self.do_asr::<AccB>()?;
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
    pub fn asla(&mut self) -> CpuResult<()> {
        self.asl()
    }

    #[inline]
    pub fn aslb(&mut self) -> CpuResult<()> {
        self.asl()
    }

    #[inline]
    pub fn lsr(&mut self) -> CpuResult<()> {
        let val = self.fetch_operand()?;
        let new_val = val.wrapping_shr(1);
        let c = val.bit(0);
        self.post_shift(c, val, new_val)
    }

    #[inline]
    pub fn lsra(&mut self) -> CpuResult<()> {
        self.lsr()
    }

    #[inline]
    pub fn lsrb(&mut self) -> CpuResult<()> {
        self.lsr()
    }

    #[inline]
    pub fn ror(&mut self) -> CpuResult<()> {
        let val = self.fetch_operand()?;
        let new_val = val.wrapping_shr(1) | if self.m.regs.c() { 1 << 7 } else { 0 };
        self.post_shift(val.bit(0), val, new_val)
    }

    pub fn rora(&mut self) -> CpuResult<()> {
        self.ror()
    }

    pub fn rorb(&mut self) -> CpuResult<()> {
        self.ror()
    }

    #[inline]
    pub fn rol(&mut self) -> CpuResult<()> {
        let val = self.fetch_operand()?;
        let new_val = val.wrapping_shl(1) | if self.m.regs.c() { 1 } else { 0 };
        self.post_shift(val.bit(1), val, new_val)
    }
    pub fn rola(&mut self) -> CpuResult<()> {
        self.rol()
    }
    pub fn rolb(&mut self) -> CpuResult<()> {
        self.rol()
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
        self.set_nz_from_u8(val).clc();
        Ok(val)
    }

    fn fetch_operand_16_fl(&mut self) -> CpuResult<u16> {
        let val = self.fetch_operand_16()?;
        self.set_nz_from_u16(val).clc();
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
    fn set_nz_from_u8(&mut self, val: u8) -> &mut R {
        let regs = self.regs_mut();
        regs.set_nz_from_u8(val);
        regs
    }

    #[inline]
    fn set_nz_from_u16(&mut self, val: u16) -> &mut R {
        let regs = self.regs_mut();
        regs.set_nz_from_u16(val);
        regs
    }

    #[inline]
    pub fn do_clr<X: Bus>(&mut self) -> CpuResult<()> {
        let regs = self.regs_mut();
        regs.cln().clc().clv().sez();
        X::store_byte(self.m, 0)?;
        Ok(())
    }

    #[inline]
    pub fn clr(&mut self) -> CpuResult<()> {
        self.do_clr::<A>()
    }

    #[inline]
    pub fn clra(&mut self) -> CpuResult<()> {
        self.do_clr::<AccA>()
    }

    #[inline]
    pub fn clrb(&mut self) -> CpuResult<()> {
        self.do_clr::<AccB>()
    }

    #[inline]
    pub fn bita(&mut self) -> CpuResult<()> {
        let op = self.fetch_operand()?;
        let (_, new) = AccA::read_mod_write(self.m, |val| val & op)?;
        self.set_nz_from_u8(new).clc();
        Ok(())
    }

    #[inline]
    pub fn bitb(&mut self) -> CpuResult<()> {
        let op = self.fetch_operand()?;
        let (_, new) = AccB::read_mod_write(self.m, |val| val & op)?;
        self.set_nz_from_u8(new).clc();
        Ok(())
    }

    #[inline]
    pub fn daa(&mut self) -> CpuResult<()> {
        panic!()
    }

    #[inline]
    pub fn post_tst(&mut self, val: u8) -> CpuResult<()> {
        self.set_nz_from_u8(val).clv().clc();
        Ok(())
    }

    #[inline]
    pub fn tst(&mut self) -> CpuResult<()> {
        let val = self.fetch_operand()?;
        self.post_tst(val)
    }

    #[inline]
    pub fn tsta(&mut self) -> CpuResult<()> {
        let val = self.m.regs.a();
        self.post_tst(val)
    }
    #[inline]
    pub fn tstb(&mut self) -> CpuResult<()> {
        let val = self.m.regs.b();
        self.post_tst(val)
    }
}
