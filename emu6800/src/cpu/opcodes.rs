use emucore::mem::MemoryIO;

use super::{Bus, RegEnum, RegisterFileTrait, StatusRegTrait};
use super::{CpuResult, Machine};

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

struct Ins<'a, A, R, M>
where
    A: Bus,
    R: RegisterFileTrait,
    M: MemoryIO,
{
    bus: A,
    m: &'a mut Machine<M, R>,
}
////////////////////////////////////////////////////////////////////////////////
// Utils
impl<'a, A, R, M> Ins<'a, A, R, M>
where
    A: Bus,
    R: RegisterFileTrait,
    M: MemoryIO,
{
    #[inline]
    fn fetch_operand(&mut self) -> CpuResult<u8> {
        let r = self.bus.fetch_operand(self.m)?;
        Ok(r)
    }
    #[inline]
    fn fetch_operand_16(&mut self) -> CpuResult<u16> {
        let r = self.bus.fetch_operand_16(self.m)?;
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
        let addr = self.bus.fetch_rel_addr(self.m)?;
        if cond {
            self.m.regs.set_pc(addr);
        }
        Ok(())
    }

    #[inline]
    fn bra(&mut self) -> CpuResult<()> {
        self.branch_cond(true)
    }

    #[inline]
    fn bpl(&mut self) -> CpuResult<()> {
        let n = self.m.regs.n();
        self.branch_cond(!n)
    }
    #[inline]
    fn blt(&mut self) -> CpuResult<()> {
        let lt = self.m.regs.lt();
        self.branch_cond(lt)
    }
    #[inline]
    fn bmi(&mut self) -> CpuResult<()> {
        let n = self.m.regs.n();
        self.branch_cond(n)
    }

    #[inline]
    fn bne(&mut self) -> CpuResult<()> {
        let z = self.m.regs.z();
        self.branch_cond(!z)
    }

    #[inline]
    fn beq(&mut self) -> CpuResult<()> {
        let z = self.m.regs.z();
        self.branch_cond(z)
    }

    #[inline]
    fn bhi(&mut self) -> CpuResult<()> {
        let hi = self.m.regs.hi();
        self.branch_cond(hi)
    }

    #[inline]
    fn bgt(&mut self) -> CpuResult<()> {
        let gt = self.m.regs.gt();
        self.branch_cond(gt)
    }

    #[inline]
    fn ble(&mut self) -> CpuResult<()> {
        let le = self.m.regs.le();
        self.branch_cond(le)
    }

    #[inline]
    fn bls(&mut self) -> CpuResult<()> {
        let ls = self.m.regs.ls();
        self.branch_cond(ls)
    }

    #[inline]
    fn bge(&mut self) -> CpuResult<()> {
        let ge = self.m.regs.ge();
        self.branch_cond(ge)
    }

    #[inline]
    fn bcs(&mut self) -> CpuResult<()> {
        let c = self.m.regs.c();
        self.branch_cond(c)
    }

    #[inline]
    fn bcc(&mut self) -> CpuResult<()> {
        let c = self.m.regs.c();
        self.branch_cond(!c)
    }

    #[inline]
    fn bvs(&mut self) -> CpuResult<()> {
        let v = self.m.regs.v();
        self.branch_cond(v)
    }

    #[inline]
    fn bvc(&mut self) -> CpuResult<()> {
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
    fn clc(&mut self) -> CpuResult<()> {
        self.m.regs.clc();
        Ok(())
    }

    #[inline]
    fn sec(&mut self) -> CpuResult<()> {
        self.m.regs.sec();
        Ok(())
    }

    #[inline]
    fn clv(&mut self) -> CpuResult<()> {
        self.m.regs.clv();
        Ok(())
    }

    #[inline]
    fn sev(&mut self) -> CpuResult<()> {
        self.m.regs.sev();
        Ok(())
    }

    #[inline]
    fn cli(&mut self) -> CpuResult<()> {
        self.m.regs.cli();
        Ok(())
    }
    #[inline]
    fn sei(&mut self) -> CpuResult<()> {
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
    fn pul(&mut self) -> CpuResult<()> {
        let val = self.m.pop_byte()?;
        self.bus.store_byte(self.m, val)?;
        Ok(())
    }

    #[inline]
    fn psh(&mut self) -> CpuResult<()> {
        let val = self.fetch_operand()?;
        self.m.push_byte(val)?;
        Ok(())
    }

    #[inline]
    /// Increment stack ptr
    fn ins(&mut self) -> CpuResult<()> {
        let r = self.regs_mut();
        let sp = r.sp();
        r.set_sp(sp.wrapping_add(1));
        Ok(())
    }

    #[inline]
    /// Decrement stack ptr
    fn des(&mut self) -> CpuResult<()> {
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
    fn nop(&mut self) -> CpuResult<()> {
        Ok(())
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
    fn rti(&mut self) -> CpuResult<()> {
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
    fn jmp(&mut self) -> CpuResult<()> {
        let addr = self.m.fetch_word()?;
        self.m.regs.set_pc(addr);
        Ok(())
    }

    #[inline]
    fn bsr(&mut self) -> CpuResult<()> {
        let addr = self.bus.fetch_rel_addr(self.m)?;
        let pc = self.m.regs.pc();
        self.m.push_word(pc)?;
        self.m.regs.set_pc(addr);
        Ok(())
    }

    #[inline]
    fn swi(&mut self) -> CpuResult<()> {
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
    fn jsr(&mut self) -> CpuResult<()> {
        let addr = self.bus.fetch_operand_16(self.m)?;
        let pc = self.m.regs.pc();
        self.m.push_word(pc)?;
        self.m.regs.set_pc(addr);
        Ok(())
    }

    #[inline]
    fn rts(&mut self) -> CpuResult<()> {
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
    fn com(&mut self) -> CpuResult<()> {
        let val = !self.bus.fetch_operand(self.m)?;
        self.bus.store_byte(self.m, val)?;
        let regs = self.regs_mut();
        regs.sec().clv().set_nz_from_u8(val);
        Ok(())
    }

    #[inline]
    fn eor_a(&mut self) -> CpuResult<()> {
        let a = self.m.regs.get_reg_8(RegEnum::A);
        let new_a = self.eor_val(a)?;
        self.m.regs.set_reg_8(RegEnum::A, new_a);
        Ok(())
    }

    #[inline]
    fn eor_b(&mut self) -> CpuResult<()> {
        let b = self.m.regs.get_reg_8(RegEnum::B);
        let new_b = self.eor_val(b)?;
        self.m.regs.set_reg_8(RegEnum::B, new_b);
        Ok(())
    }

    #[inline]
    fn and_a(&mut self) -> CpuResult<()> {
        let a = self.m.regs.get_reg_8(RegEnum::A);
        let new_a = self.and_val(a)?;
        self.m.regs.set_reg_8(RegEnum::A, new_a);
        Ok(())
    }

    #[inline]
    fn and_b(&mut self) -> CpuResult<()> {
        let b = self.m.regs.get_reg_8(RegEnum::B);
        let new_b = self.and_val(b)?;
        self.m.regs.set_reg_8(RegEnum::B, new_b);
        Ok(())
    }

    #[inline]
    fn ora_a(&mut self) -> CpuResult<()> {
        let a = self.m.regs.get_reg_8(RegEnum::A);
        let new_a = self.or_val(a)?;
        self.m.regs.set_reg_8(RegEnum::A, new_a);
        Ok(())
    }

    #[inline]
    fn ora_b(&mut self) -> CpuResult<()> {
        let b = self.m.regs.get_reg_8(RegEnum::B);
        let new_b = self.or_val(b)?;
        self.m.regs.set_reg_8(RegEnum::B, new_b);
        Ok(())
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

        let n = new_val & 0x80 == 0x80;
        let z = new_val == 0;
        let v = new_val & 0x80 != val & 0x80;
        let c = new_val > val;

        self.m.regs.set_n(n);
        self.m.regs.set_z(z);
        self.m.regs.set_v(v);
        self.m.regs.set_c(c);

        Ok(val)
    }

    fn do_add(&mut self, c: bool, val: u8, op: u8) -> CpuResult<u8> {
        let new_val = val.wrapping_add(op).wrapping_add(bool_as_u8(c));

        let n = new_val & 0x80 == 0x80;
        let z = new_val == 0;
        let v = new_val & 0x80 != val & 0x80;
        let c = new_val > val;

        self.m.regs.set_n(n);
        self.m.regs.set_z(z);
        self.m.regs.set_v(v);
        self.m.regs.set_c(c);

        Ok(val)
    }

    #[inline]
    fn neg(&mut self) -> CpuResult<()> {
        let val = self.m.fetch_byte()? as i8;
        let new_val = -val;
        self.bus.store_byte(self.m, new_val as u8)?;
        panic!()
    }

    #[inline]
    fn cmp_a(&mut self) -> CpuResult<()> {
        let op = self.fetch_operand()?;
        let a = self.m.regs.a();
        let _ = self.do_sub(false, a, op)?;
        Ok(())
    }

    #[inline]
    fn cmp_b(&mut self) -> CpuResult<()> {
        let op = self.fetch_operand()?;
        let b = self.m.regs.b();
        let _ = self.do_sub(false, b, op)?;
        Ok(())
    }

    #[inline]
    fn add_b(&mut self) -> CpuResult<()> {
        let op = self.fetch_operand()?;
        let b = self.m.regs.b();
        let b = self.do_add(false, b, op)?;
        self.regs_mut().set_b(b);
        Ok(())
    }

    #[inline]
    fn adc_b(&mut self) -> CpuResult<()> {
        let op = self.fetch_operand()?;
        let b = self.m.regs.b();
        let c = self.m.regs.c();
        let new_b = self.do_add(c, b, op)?;
        self.m.regs.set_b(new_b);
        Ok(())
    }

    #[inline]
    fn add_a(&mut self) -> CpuResult<()> {
        let op = self.bus.fetch_operand(self.m)?;
        let a = self.m.regs.a();
        let new_a = self.do_add(false, a, op)?;
        self.m.regs.set_a(new_a);
        Ok(())
    }
    #[inline]
    fn adc_a(&mut self) -> CpuResult<()> {
        let op = self.fetch_operand()?;
        let a = self.m.regs.a();
        let c = self.m.regs.c();
        let a = self.do_add(c, a, op)?;
        self.m.regs.set_a(a);
        Ok(())
    }

    #[inline]
    fn aba(&mut self) -> CpuResult<()> {
        let a = self.m.regs.a();
        let b = self.m.regs.b();
        let a = self.do_add(false, a, b)?;
        self.m.regs.set_a(a);
        Ok(())
    }

    #[inline]
    fn sba(&mut self) -> CpuResult<()> {
        let a = self.m.regs.a();
        let b = self.m.regs.b();
        let a = self.do_sub(false, a, b)?;
        self.m.regs.set_a(a);
        Ok(())
    }

    #[inline]
    fn sub_a(&mut self) -> CpuResult<()> {
        self.sub_reg(false, RegEnum::A)
    }

    #[inline]
    fn sub_b(&mut self) -> CpuResult<()> {
        self.sub_reg(false, RegEnum::B)
    }

    #[inline]
    fn sbc_a(&mut self) -> CpuResult<()> {
        let c = self.m.regs.c();
        self.sub_reg(c, RegEnum::A)
    }

    fn sub_reg(&mut self, c: bool, reg: RegEnum) -> CpuResult<()> {
        let op = self.fetch_operand()?;
        let r = self.m.regs.get_reg_8(reg);
        let v = self.do_sub(c, r, op)?;
        self.m.regs.set_reg_8(reg, v);
        Ok(())
    }

    fn sbc_b(&mut self) -> CpuResult<()> {
        let c = self.m.regs.c();
        self.sub_reg(c, RegEnum::B)
    }

    #[inline]
    fn cpx(&mut self) -> CpuResult<()> {
        let op = self.fetch_operand_16()?;
        let x = self.m.regs.x();
        let new_x = x.wrapping_sub(op);
        self.m.regs.set_x(new_x);

        let n = new_x & 0x8000 == 0x8000;
        let z = new_x == 0;
        let v = new_x & 0x8000 != x & 0x8000;

        self.m.regs.set_n(n);
        self.m.regs.set_z(z);
        self.m.regs.set_z(v);

        Ok(())
    }

    #[inline]
    fn cba(&mut self) -> CpuResult<()> {
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
    fn inx(&mut self) -> CpuResult<()> {
        let x = self.m.regs.x().wrapping_add(1);
        // dex only sets zero flag - no bpl/bmu x loops on 6800
        self.m.regs.set_z(x == 0);
        self.m.regs.set_x(x);
        Ok(())
    }

    #[inline]
    fn dex(&mut self) -> CpuResult<()> {
        let x = self.m.regs.x().wrapping_sub(1);
        // dex only sets zero flag - no bpl/bmi x loops on 6800
        self.m.regs.set_z(x == 0);
        self.m.regs.set_x(x);
        Ok(())
    }

    #[inline]
    fn inc(&mut self) -> CpuResult<()> {
        let val = self.fetch_operand()?;
        let new_val = val.wrapping_add(1);
        self.bus.store_byte(self.m, new_val)?;
        self.m.regs.set_nz_from_u8(new_val);
        self.m.regs.set_v(new_val < val);
        Ok(())
    }

    #[inline]
    fn dec(&mut self) -> CpuResult<()> {
        let val = self.fetch_operand()?;
        let new_val = val.wrapping_sub(1);
        self.bus.store_byte(self.m, new_val)?;
        self.m.regs.set_nz_from_u8(new_val);
        self.m.regs.set_v(new_val > val);
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
    fn finish_a_shift(&mut self, c: bool, val: u8, new_val: u8) -> CpuResult<()> {
        self.bus.store_byte(self.m, new_val)?;
        let v = val & 0x80 != new_val & 0x80;
        self.m.regs.set_c(c);
        self.m.regs.set_nz_from_u8(new_val);
        self.m.regs.set_v(v);
        Ok(())
    }

    #[inline]
    fn asr(&mut self) -> CpuResult<()> {
        let val = self.fetch_operand()?;
        let new_val = val.wrapping_shr(1);
        let new_val = new_val | (val & 0x80);
        let c = (val & 0x80) == 0x80;
        self.finish_a_shift(c, val, new_val)
    }

    #[inline]
    fn asl(&mut self) -> CpuResult<()> {
        let val = self.fetch_operand()?;
        let new_val = val.wrapping_shl(1);
        let c = (val & 0x80) == 0x80;
        self.finish_a_shift(c, val, new_val)
    }

    #[inline]
    fn lsr(&mut self) -> CpuResult<()> {
        let val = self.fetch_operand()?;
        let new_val = val.wrapping_shr(1);
        let c = (val & 1) == 1;
        self.finish_a_shift(c, val, new_val)
    }

    #[inline]
    fn ror(&mut self) -> CpuResult<()> {
        let val = self.fetch_operand()?;

        let new_val = if self.m.regs.c() {
            val.wrapping_shr(1) | 0x80
        } else {
            val.wrapping_shr(1)
        };

        let c = (val & 1) == 1;

        self.finish_a_shift(c, val, new_val)
    }

    #[inline]
    fn rol(&mut self) -> CpuResult<()> {
        let val = self.fetch_operand()?;

        let new_val = if self.m.regs.c() {
            val.wrapping_shl(1) | 1
        } else {
            val.wrapping_shr(1)
        };

        let c = (val & 0x80) == 0x80;

        self.finish_a_shift(c, val, new_val)
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
    fn tpa(&mut self) -> CpuResult<()> {
        let regs = self.regs_mut();
        let sr = regs.sr();
        regs.set_a(sr);
        Ok(())
    }
    #[inline]
    fn tap(&mut self) -> CpuResult<()> {
        let regs = self.regs_mut();
        let a = regs.a();
        regs.set_sr(a);
        Ok(())
    }
    #[inline]
    fn tba(&mut self) -> CpuResult<()> {
        let regs = &mut self.m.regs;
        let b = regs.b();
        regs.set_a(b).set_nz_from_u8(b).clv();
        Ok(())
    }

    #[inline]
    fn tab(&mut self) -> CpuResult<()> {
        let regs = &mut self.m.regs;
        let b = regs.b();
        regs.set_b(b);
        regs.set_nz_from_u8(b).clv();
        Ok(())
    }

    #[inline]
    fn tsx(&mut self) -> CpuResult<()> {
        let regs = &mut self.m.regs;
        let sp = regs.sp();
        regs.set_x(sp);
        Ok(())
    }
    #[inline]
    fn txs(&mut self) -> CpuResult<()> {
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

    fn ld8(&mut self) -> CpuResult<u8> {
        let val = self.fetch_operand()?;
        let regs = self.regs_mut();
        regs.set_nz_from_u8(val);
        regs.clc();
        Ok(val)
    }

    fn ld16(&mut self) -> CpuResult<u16> {
        let val = self.fetch_operand_16()?;
        let regs = self.regs_mut();
        regs.set_nz_from_u16(val);
        regs.clc();
        Ok(val)
    }

    #[inline]
    fn lds(&mut self) -> CpuResult<()> {
        let sp = self.ld16()?;
        self.m.regs.set_sp(sp);
        Ok(())
    }

    #[inline]
    fn lda_a(&mut self) -> CpuResult<()> {
        let a = self.ld8()?;
        self.regs_mut().set_a(a);
        Ok(())
    }

    #[inline]
    fn lda_b(&mut self) -> CpuResult<()> {
        let r = self.ld8()?;
        self.regs_mut().set_b(r);
        Ok(())
    }

    #[inline]
    fn ldx(&mut self) -> CpuResult<()> {
        let r = self.ld16()?;
        self.regs_mut().set_x(r);
        Ok(())
    }

    #[inline]
    fn st8(&mut self, val: u8) -> CpuResult<()> {
        let regs = self.regs_mut();
        regs.set_nz_from_u8(val);
        regs.clv();
        self.bus.store_byte(self.m, val)?;
        Ok(())
    }
    #[inline]
    fn st16(&mut self, val: u16) -> CpuResult<()> {
        let regs = self.regs_mut();
        regs.set_nz_from_u16(val);
        regs.clv();
        self.bus.store_word(self.m, val)?;
        Ok(())
    }

    #[inline]
    fn sta_a(&mut self) -> CpuResult<()> {
        let a = self.m.regs.a();
        self.st8(a)
    }

    #[inline]
    fn sta_b(&mut self) -> CpuResult<()> {
        let b = self.m.regs.b();
        self.st8(b)
    }

    #[inline]
    fn sts(&mut self) -> CpuResult<()> {
        let sp = self.m.regs.sp();
        self.st16(sp)
    }
    #[inline]
    fn stx(&mut self) -> CpuResult<()> {
        let x = self.m.regs.x();
        self.st16(x)
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
    fn clr(&mut self) -> CpuResult<()> {
        let regs = self.regs_mut();
        regs.cln().clc().clv().sez();
        self.bus.store_byte(self.m, 0)?;
        Ok(())
    }

    #[inline]
    fn do_bit(&mut self, val: u8) -> CpuResult<()> {
        let operand = self.fetch_operand()?;
        let new_val = val & operand;
        let regs = self.regs_mut();
        regs.set_nz_from_u8(new_val).clc();
        self.bus.store_byte(self.m, new_val)?;
        Ok(())
    }

    #[inline]
    fn bit_a(&mut self) -> CpuResult<()> {
        let val = self.regs_mut().a();
        self.do_bit(val)
    }

    #[inline]
    fn bit_b(&mut self) -> CpuResult<()> {
        let val = self.regs_mut().a();
        self.do_bit(val)
    }

    #[inline]
    fn daa(&mut self) -> CpuResult<()> {
        panic!()
    }

    #[inline]
    fn tst(&mut self) -> CpuResult<()> {
        let val = self.fetch_operand()?;
        let regs = self.regs_mut();
        regs.set_nz_from_u8(val).clv().clc();
        Ok(())
    }
}
