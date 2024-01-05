use emucore::mem::MemoryIO;

use super::{Bus, Flags};
use super::{CpuResult, Machine};

////////////////////////////////////////////////////////////////////////////////
// Helpers
#[inline]
fn branch_cond<A: Bus, M: MemoryIO>(mut bus: A, m: &mut Machine<M>, cond: bool) -> CpuResult<()> {
    let addr = bus.fetch_rel_addr(m)?;
    if cond {
        m.regs.pc = addr;
    }
    Ok(())
}

#[inline]
fn bool_as_u8(m: bool) -> u8 {
    if m {
        1u8
    } else {
        0u8
    }
}

////////////////////////////////////////////////////////////////////////////////
// Relative branches
#[inline]
fn bra<A: Bus, M: MemoryIO>(bus: A, m: &mut Machine<M>) -> CpuResult<()> {
    branch_cond(bus, m, true)
}

#[inline]
fn bpl<A: Bus, M: MemoryIO>(bus: A, m: &mut Machine<M>) -> CpuResult<()> {
    branch_cond(bus, m, !m.regs.flags.pl())
}

#[inline]
fn blt<A: Bus, M: MemoryIO>(bus: A, m: &mut Machine<M>) -> CpuResult<()> {
    branch_cond(bus, m, !m.regs.flags.lt())
}

#[inline]
fn bmi<A: Bus, M: MemoryIO>(bus: A, m: &mut Machine<M>) -> CpuResult<()> {
    branch_cond(bus, m, m.regs.flags.mi())
}

#[inline]
fn bne<A: Bus, M: MemoryIO>(bus: A, m: &mut Machine<M>) -> CpuResult<()> {
    branch_cond(bus, m, !m.regs.flags.ne())
}

#[inline]
fn beq<A: Bus, M: MemoryIO>(bus: A, m: &mut Machine<M>) -> CpuResult<()> {
    branch_cond(bus, m, m.regs.flags.eq())
}

#[inline]
fn bhi<A: Bus, M: MemoryIO>(bus: A, m: &mut Machine<M>) -> CpuResult<()> {
    branch_cond(bus, m, m.regs.flags.hi())
}

#[inline]
fn bgt<A: Bus, M: MemoryIO>(bus: A, m: &mut Machine<M>) -> CpuResult<()> {
    branch_cond(bus, m, m.regs.flags.gt())
}

#[inline]
fn ble<A: Bus, M: MemoryIO>(bus: A, m: &mut Machine<M>) -> CpuResult<()> {
    branch_cond(bus, m, m.regs.flags.le())
}

#[inline]
fn bls<A: Bus, M: MemoryIO>(bus: A, m: &mut Machine<M>) -> CpuResult<()> {
    branch_cond(bus, m, m.regs.flags.ls())
}

#[inline]
fn bge<A: Bus, M: MemoryIO>(bus: A, m: &mut Machine<M>) -> CpuResult<()> {
    branch_cond(bus, m, m.regs.flags.ge())
}

#[inline]
fn bcs<A: Bus, M: MemoryIO>(bus: A, m: &mut Machine<M>) -> CpuResult<()> {
    branch_cond(bus, m, m.regs.flags.c())
}

#[inline]
fn bcc<A: Bus, M: MemoryIO>(bus: A, m: &mut Machine<M>) -> CpuResult<()> {
    branch_cond(bus, m, !m.regs.flags.c())
}

#[inline]
fn bvs<A: Bus, M: MemoryIO>(bus: A, m: &mut Machine<M>) -> CpuResult<()> {
    branch_cond(bus, m, m.regs.flags.v())
}

#[inline]
fn bvc<A: Bus, M: MemoryIO>(bus: A, m: &mut Machine<M>) -> CpuResult<()> {
    branch_cond(bus, m, !m.regs.flags.v())
}

////////////////////////////////////////////////////////////////////////////////

////////////////////////////////////////////////////////////////////////////////
// Flags
#[inline]
fn clc<A: Bus, M: MemoryIO>(_bus: A, m: &mut Machine<M>) -> CpuResult<()> {
    m.regs.flags.remove(Flags::C);
    Ok(())
}

#[inline]
fn sec<A: Bus, M: MemoryIO>(_bus: A, m: &mut Machine<M>) -> CpuResult<()> {
    m.regs.flags.set(Flags::C, false);
    Ok(())
}

#[inline]
fn clv<A: Bus, M: MemoryIO>(_bus: A, m: &mut Machine<M>) -> CpuResult<()> {
    m.regs.flags.set(Flags::V, false);
    Ok(())
}

#[inline]
fn sev<A: Bus, M: MemoryIO>(_bus: A, m: &mut Machine<M>) -> CpuResult<()> {
    m.regs.flags.set(Flags::V, true);
    Ok(())
}

#[inline]
fn cli<A: Bus, M: MemoryIO>(_bus: A, m: &mut Machine<M>) -> CpuResult<()> {
    m.regs.flags.set(Flags::I, false);
    Ok(())
}
#[inline]
fn sei<A: Bus, M: MemoryIO>(_bus: A, m: &mut Machine<M>) -> CpuResult<()> {
    m.regs.flags.set(Flags::I, true);
    Ok(())
}

////////////////////////////////////////////////////////////////////////////////
// Stack stuff
#[inline]
fn pul<A: Bus, M: MemoryIO>(bus: A, m: &mut Machine<M>) -> CpuResult<()> {
    let val = m.pop_byte()?;
    bus.store_byte(m, val)?;
    Ok(())
}

#[inline]
fn psh<A: Bus, M: MemoryIO>(mut bus: A, m: &mut Machine<M>) -> CpuResult<()> {
    let val = bus.fetch_operand(m)?;
    m.push_byte(val)?;
    Ok(())
}

#[inline]
fn ins<A: Bus, M: MemoryIO>(_bus: A, m: &mut Machine<M>) -> CpuResult<()> {
    m.regs.sp = m.regs.sp.wrapping_add(1);
    Ok(())
}
#[inline]
fn des<A: Bus, M: MemoryIO>(_bus: A, m: &mut Machine<M>) -> CpuResult<()> {
    m.regs.sp = m.regs.sp.wrapping_sub(1);
    Ok(())
}

////////////////////////////////////////////////////////////////////////////////
#[inline]
fn nop<A: Bus, M: MemoryIO>(_bus: A, _machine: &mut Machine<M>) -> CpuResult<()> {
    Ok(())
}

////////////////////////////////////////////////////////////////////////////////
// jmp / sub / returns
#[inline]
fn rti<A: Bus, M: MemoryIO>(_bus: A, m: &mut Machine<M>) -> CpuResult<()> {
    m.regs.flags = Flags::from_bits(m.pop_byte()?).unwrap();
    m.regs.b = m.pop_byte()?;
    m.regs.a = m.pop_byte()?;
    m.regs.x = m.pop_word()?;
    m.regs.pc = m.pop_word()?;
    Ok(())
}

#[inline]
fn jmp<A: Bus, M: MemoryIO>(_bus: A, m: &mut Machine<M>) -> CpuResult<()> {
    let addr = m.fetch_word()?;
    m.regs.pc = addr;
    Ok(())
}

#[inline]
fn bsr<A: Bus, M: MemoryIO>(mut bus: A, m: &mut Machine<M>) -> CpuResult<()> {
    let addr = bus.fetch_rel_addr(m)?;
    m.push_word(m.regs.pc)?;
    m.regs.pc = addr;
    Ok(())
}

#[inline]
fn swi<A: Bus, M: MemoryIO>(_bus: A, m: &mut Machine<M>) -> CpuResult<()> {
    m.push_word(m.regs.pc)?;
    m.push_word(m.regs.x)?;
    m.push_byte(m.regs.a)?;
    m.push_byte(m.regs.b)?;
    m.push_byte(m.regs.flags.bits())?;

    let addr = m.mem_mut().load_word(0xfffa)?;

    m.regs.pc = addr;
    Ok(())
}

#[inline]
fn jsr<A: Bus, M: MemoryIO>(mut bus: A, m: &mut Machine<M>) -> CpuResult<()> {
    let addr = bus.fetch_operand_16(m)?;
    m.push_word(m.regs.pc)?;
    m.regs.pc = addr;
    Ok(())
}

#[inline]
fn rts<A: Bus, M: MemoryIO>(_bus: A, m: &mut Machine<M>) -> CpuResult<()> {
    let addr = m.pop_word()?;
    m.regs.pc = addr;
    Ok(())
}

////////////////////////////////////////////////////////////////////////////////
// bool logic
#[inline]
fn eor_val<A: Bus, M: MemoryIO>(_bus: A, m: &mut Machine<M>, val: u8) -> CpuResult<u8> {
    let operand = m.fetch_byte()?;
    let res = val ^ operand;
    m.regs.flags.set_nz(res);
    m.regs.flags.clv();
    Ok(res)
}

fn and_val<A: Bus, M: MemoryIO>(_bus: A, m: &mut Machine<M>, val: u8) -> CpuResult<u8> {
    let operand = m.fetch_byte()?;
    let res = val & operand;
    m.regs.flags.set_nz(res);
    m.regs.flags.clv();
    Ok(res)
}

fn or_val<A: Bus, M: MemoryIO>(_bus: A, m: &mut Machine<M>, val: u8) -> CpuResult<u8> {
    let res = val | m.fetch_byte()?;
    m.regs.flags.set_nz(res);
    m.regs.flags.clv();
    Ok(res)
}

#[inline]
fn com<A: Bus, M: MemoryIO>(mut bus: A, m: &mut Machine<M>) -> CpuResult<()> {
    let val = !bus.fetch_operand(m)?;
    bus.store_byte(m, val)?;
    m.regs.flags.sec();
    m.regs.flags.clv();
    m.regs.flags.set_nz(val);
    Ok(())
}

#[inline]
fn eor_a<A: Bus, M: MemoryIO>(bus: A, m: &mut Machine<M>) -> CpuResult<()> {
    m.regs.a = eor_val(bus, m, m.regs.a)?;
    Ok(())
}

#[inline]
fn eor_b<A: Bus, M: MemoryIO>(bus: A, m: &mut Machine<M>) -> CpuResult<()> {
    m.regs.b = eor_val(bus, m, m.regs.b)?;
    Ok(())
}

#[inline]
fn and_a<A: Bus, M: MemoryIO>(bus: A, m: &mut Machine<M>) -> CpuResult<()> {
    m.regs.a = and_val(bus, m, m.regs.a)?;
    Ok(())
}

#[inline]
fn and_b<A: Bus, M: MemoryIO>(bus: A, m: &mut Machine<M>) -> CpuResult<()> {
    m.regs.b = and_val(bus, m, m.regs.b)?;
    Ok(())
}

#[inline]
fn ora_a<A: Bus, M: MemoryIO>(bus: A, m: &mut Machine<M>) -> CpuResult<()> {
    m.regs.a = or_val(bus, m, m.regs.a)?;
    Ok(())
}

#[inline]
fn ora_b<A: Bus, M: MemoryIO>(bus: A, m: &mut Machine<M>) -> CpuResult<()> {
    m.regs.b = or_val(bus, m, m.regs.b)?;
    Ok(())
}

////////////////////////////////////////////////////////////////////////////////
// Adds, subs

fn do_sub<M: MemoryIO>(m: &mut Machine<M>, c: bool, val: u8, op: u8) -> CpuResult<u8> {
    let new_val = val.wrapping_sub(op).wrapping_sub(bool_as_u8(c));

    let n = new_val & 0x80 == 0x80;
    let z = new_val == 0;
    let v = new_val & 0x80 != val & 0x80;
    let c = new_val > val;

    m.regs.flags.set_n(n);
    m.regs.flags.set_z(z);
    m.regs.flags.set_z(v);
    m.regs.flags.set_c(c);

    Ok(val)
}

fn do_add<M: MemoryIO>(m: &mut Machine<M>, c: bool, val: u8, op: u8) -> CpuResult<u8> {
    let new_val = val.wrapping_add(op).wrapping_add(bool_as_u8(c));

    let n = new_val & 0x80 == 0x80;
    let z = new_val == 0;
    let v = new_val & 0x80 != val & 0x80;
    let c = new_val > val;

    m.regs.flags.set_n(n);
    m.regs.flags.set_z(z);
    m.regs.flags.set_z(v);
    m.regs.flags.set_c(c);

    Ok(val)
}

#[inline]
fn neg<A: Bus, M: MemoryIO>(bus: A, m: &mut Machine<M>) -> CpuResult<()> {
    let val = m.fetch_byte()? as i8;
    let new_val = -val;
    bus.store_byte(m, new_val as u8)?;
    panic!()
}

#[inline]
fn cmp_a<A: Bus, M: MemoryIO>(mut bus: A, m: &mut Machine<M>) -> CpuResult<()> {
    let op = bus.fetch_operand(m)?;
    let _ = do_sub(m, false, m.regs.a, op)?;
    Ok(())
}

#[inline]
fn cmp_b<A: Bus, M: MemoryIO>(mut bus: A, m: &mut Machine<M>) -> CpuResult<()> {
    let op = bus.fetch_operand(m)?;
    let _ = do_sub(m, false, m.regs.a, op)?;
    Ok(())
}

#[inline]
fn add_b<A: Bus, M: MemoryIO>(mut bus: A, m: &mut Machine<M>) -> CpuResult<()> {
    let op = bus.fetch_operand(m)?;
    m.regs.b = do_add(m, false, m.regs.b, op)?;
    Ok(())
}

#[inline]
fn adc_b<A: Bus, M: MemoryIO>(mut bus: A, m: &mut Machine<M>) -> CpuResult<()> {
    let op = bus.fetch_operand(m)?;
    m.regs.b = do_add(m, m.regs.flags.c(), m.regs.b, op)?;
    Ok(())
}

#[inline]
fn add_a<A: Bus, M: MemoryIO>(mut bus: A, m: &mut Machine<M>) -> CpuResult<()> {
    let op = bus.fetch_operand(m)?;
    m.regs.a = do_add(m, false, m.regs.a, op)?;
    Ok(())
}
#[inline]
fn adc_a<A: Bus, M: MemoryIO>(mut bus: A, m: &mut Machine<M>) -> CpuResult<()> {
    let op = bus.fetch_operand(m)?;
    m.regs.a = do_add(m, m.regs.flags.c(), m.regs.a, op)?;
    Ok(())
}

#[inline]
fn aba<A: Bus, M: MemoryIO>(_bus: A, m: &mut Machine<M>) -> CpuResult<()> {
    m.regs.a = do_add(m, false, m.regs.a, m.regs.b)?;
    Ok(())
}

#[inline]
fn sba<A: Bus, M: MemoryIO>(_bus: A, m: &mut Machine<M>) -> CpuResult<()> {
    m.regs.a = do_sub(m, false, m.regs.a, m.regs.b)?;
    Ok(())
}

#[inline]
fn sub_a<A: Bus, M: MemoryIO>(mut bus: A, m: &mut Machine<M>) -> CpuResult<()> {
    let op = bus.fetch_operand(m)?;
    m.regs.a = do_sub(m, false, m.regs.a, op)?;
    Ok(())
}
#[inline]
fn sub_b<A: Bus, M: MemoryIO>(mut bus: A, m: &mut Machine<M>) -> CpuResult<()> {
    let op = bus.fetch_operand(m)?;
    m.regs.b = do_sub(m, false, m.regs.b, op)?;
    Ok(())
}

#[inline]
fn sbc_a<A: Bus, M: MemoryIO>(mut bus: A, m: &mut Machine<M>) -> CpuResult<()> {
    let op = bus.fetch_operand(m)?;
    m.regs.a = do_sub(m, m.regs.flags.c(), m.regs.a, op)?;
    Ok(())
}

fn sbc_b<A: Bus, M: MemoryIO>(mut bus: A, m: &mut Machine<M>) -> CpuResult<()> {
    let op = bus.fetch_operand(m)?;
    m.regs.b = do_sub(m, m.regs.flags.c(), m.regs.b, op)?;
    Ok(())
}

#[inline]
fn cpx<A: Bus, M: MemoryIO>(mut bus: A, m: &mut Machine<M>) -> CpuResult<()> {
    let op = bus.fetch_operand_16(m)?;
    let x = m.regs.x;
    let new_x = m.regs.x.wrapping_sub(op);
    m.regs.x = new_x;

    let n = new_x & 0x8000 == 0x8000;
    let z = new_x == 0;
    let v = new_x & 0x8000 != x & 0x8000;

    m.regs.flags.set_n(n);
    m.regs.flags.set_z(z);
    m.regs.flags.set_z(v);

    Ok(())
}

#[inline]
fn cba<A: Bus, M: MemoryIO>(_bus: A, m: &mut Machine<M>) -> CpuResult<()> {
    let _ = do_sub(m, false, m.regs.a, m.regs.b)?;
    Ok(())
}

////////////////////////////////////////////////////////////////////////////////
// inc / dec
#[inline]
fn inx<A: Bus, M: MemoryIO>(_bus: A, m: &mut Machine<M>) -> CpuResult<()> {
    let x = m.regs.x.wrapping_add(1);
    // dex only sets zero flag - no bpl/bmu x loops on 6800
    m.regs.flags.set_z(x == 0);
    m.regs.x = x;
    Ok(())
}

#[inline]
fn dex<A: Bus, M: MemoryIO>(_bus: A, m: &mut Machine<M>) -> CpuResult<()> {
    let x = m.regs.x.wrapping_add(0xff);
    // dex only sets zero flag - no bpl/bmi x loops on 6800
    m.regs.flags.set_z(x == 0);
    m.regs.x = x;
    Ok(())
}

#[inline]
fn inc<A: Bus, M: MemoryIO>(mut bus: A, m: &mut Machine<M>) -> CpuResult<()> {
    let val = bus.fetch_operand(m)?;
    let new_val = val.wrapping_add(1);
    bus.store_byte(m, new_val)?;
    m.regs.flags.set_nz(new_val);
    m.regs.flags.set_v(new_val < val);
    Ok(())
}

#[inline]
fn dec<A: Bus, M: MemoryIO>(mut bus: A, m: &mut Machine<M>) -> CpuResult<()> {
    let val = bus.fetch_operand(m)?;
    let new_val = val.wrapping_sub(1);
    bus.store_byte(m, new_val)?;
    m.regs.flags.set_nz(new_val);
    m.regs.flags.set_v(new_val > val);
    Ok(())
}

////////////////////////////////////////////////////////////////////////////////
// Shifts
//
fn finish_a_shift<A: Bus, M: MemoryIO>(
    bus: A,
    m: &mut Machine<M>,
    c: bool,
    val: u8,
    new_val: u8,
) -> CpuResult<()> {
    bus.store_byte(m, new_val)?;
    let v = val & 0x80 != new_val & 0x80;
    m.regs.flags.set(Flags::C, c);
    m.regs.flags.set_nz(new_val);
    m.regs.flags.set_v(v);
    Ok(())
}

#[inline]
fn asr<A: Bus, M: MemoryIO>(mut bus: A, m: &mut Machine<M>) -> CpuResult<()> {
    let val = bus.fetch_operand(m)?;
    let new_val = val.wrapping_shr(1);
    let new_val = new_val | (val & 0x80);
    let c = (val & 0x80) == 0x80;
    finish_a_shift(bus, m, c, val, new_val)
}

#[inline]
fn asl<A: Bus, M: MemoryIO>(mut bus: A, m: &mut Machine<M>) -> CpuResult<()> {
    let val = bus.fetch_operand(m)?;
    let new_val = val.wrapping_shl(1);
    let c = (val & 0x80) == 0x80;
    finish_a_shift(bus, m, c, val, new_val)
}

#[inline]
fn lsr<A: Bus, M: MemoryIO>(mut bus: A, m: &mut Machine<M>) -> CpuResult<()> {
    let val = bus.fetch_operand(m)?;
    let new_val = val.wrapping_shr(1);
    let c = (val & 1) == 1;
    finish_a_shift(bus, m, c, val, new_val)
}

#[inline]
fn ror<A: Bus, M: MemoryIO>(mut bus: A, m: &mut Machine<M>) -> CpuResult<()> {
    let val = bus.fetch_operand(m)?;

    let new_val = if m.regs.flags.c() {
        val.wrapping_shr(1) | 0x80
    } else {
        val.wrapping_shr(1)
    };

    let c = (val & 1) == 1;

    finish_a_shift(bus, m, c, val, new_val)
}

#[inline]
fn rol<A: Bus, M: MemoryIO>(mut bus: A, m: &mut Machine<M>) -> CpuResult<()> {
    let val = bus.fetch_operand(m)?;

    let new_val = if m.regs.flags.c() {
        val.wrapping_shl(1) | 1
    } else {
        val.wrapping_shr(1)
    };

    let c = (val & 0x80) == 0x80;

    finish_a_shift(bus, m, c, val, new_val)
}

////////////////////////////////////////////////////////////////////////////////
// Transfers
#[inline]
fn tpa<A: Bus, M: MemoryIO>(_bus: A, m: &mut Machine<M>) -> CpuResult<()> {
    m.regs.a = m.regs.flags.bits();
    Ok(())
}
#[inline]
fn tap<A: Bus, M: MemoryIO>(_bus: A, m: &mut Machine<M>) -> CpuResult<()> {
    m.regs.flags = Flags::from_bits(m.regs.a).unwrap();
    Ok(())
}
#[inline]
fn tba<A: Bus, M: MemoryIO>(_bus: A, m: &mut Machine<M>) -> CpuResult<()> {
    m.regs.a = m.regs.b;
    m.regs.flags.set_nz(m.regs.a);
    m.regs.flags.clv();
    Ok(())
}

#[inline]
fn tab<A: Bus, M: MemoryIO>(_bus: A, m: &mut Machine<M>) -> CpuResult<()> {
    m.regs.b = m.regs.a;
    m.regs.flags.set_nz(m.regs.a);
    m.regs.flags.clv();
    Ok(())
}

#[inline]
fn tsx<A: Bus, M: MemoryIO>(_bus: A, m: &mut Machine<M>) -> CpuResult<()> {
    m.regs.x = m.regs.sp;
    Ok(())
}
#[inline]
fn txs<A: Bus, M: MemoryIO>(_bus: A, m: &mut Machine<M>) -> CpuResult<()> {
    m.regs.sp = m.regs.x;
    Ok(())
}

////////////////////////////////////////////////////////////////////////////////
// Load / stores
fn ld8<A: Bus, M: MemoryIO>(mut bus: A, m: &mut Machine<M>) -> CpuResult<u8> {
    let val = bus.fetch_operand(m)?;
    m.regs.flags.set_nz(val);
    m.regs.flags.clc();
    Ok(val)
}
fn ld16<A: Bus, M: MemoryIO>(mut bus: A, m: &mut Machine<M>) -> CpuResult<u16> {
    let val = bus.fetch_operand_16(m)?;
    m.regs.flags.set_nz_16(val);
    m.regs.flags.clc();
    Ok(val)
}

#[inline]
fn lds<A: Bus, M: MemoryIO>(bus: A, m: &mut Machine<M>) -> CpuResult<()> {
    m.regs.sp = ld16(bus, m)?;
    Ok(())
}

#[inline]
fn lda_a<A: Bus, M: MemoryIO>(bus: A, m: &mut Machine<M>) -> CpuResult<()> {
    m.regs.a = ld8(bus, m)?;
    Ok(())
}

#[inline]
fn lda_b<A: Bus, M: MemoryIO>(bus: A, m: &mut Machine<M>) -> CpuResult<()> {
    m.regs.b = ld8(bus, m)?;
    Ok(())
}

#[inline]
fn ldx<A: Bus, M: MemoryIO>(bus: A, m: &mut Machine<M>) -> CpuResult<()> {
    m.regs.x = ld16(bus, m)?;
    Ok(())
}

#[inline]
fn st8<A: Bus, M: MemoryIO>(bus: A, m: &mut Machine<M>, val: u8) -> CpuResult<()> {
    m.regs.flags.set_nz(val);
    m.regs.flags.clv();
    bus.store_byte(m, val)?;
    Ok(())
}
#[inline]
fn st16<A: Bus, M: MemoryIO>(bus: A, m: &mut Machine<M>, val: u16) -> CpuResult<()> {
    m.regs.flags.set_nz_16(val);
    m.regs.flags.clv();
    bus.store_word(m, val)?;
    Ok(())
}

#[inline]
fn sta_a<A: Bus, M: MemoryIO>(bus: A, m: &mut Machine<M>) -> CpuResult<()> {
    st8(bus, m, m.regs.a)
}

#[inline]
fn sta_b<A: Bus, M: MemoryIO>(bus: A, m: &mut Machine<M>) -> CpuResult<()> {
    st8(bus, m, m.regs.b)
}

#[inline]
fn sts<A: Bus, M: MemoryIO>(bus: A, m: &mut Machine<M>) -> CpuResult<()> {
    st16(bus, m, m.regs.sp)
}
#[inline]
fn stx<A: Bus, M: MemoryIO>(bus: A, m: &mut Machine<M>) -> CpuResult<()> {
    st16(bus, m, m.regs.x)
}

////////////////////////////////////////////////////////////////////////////////
#[inline]
fn clr<A: Bus, M: MemoryIO>(bus: A, m: &mut Machine<M>) -> CpuResult<()> {
    m.regs.flags.set(Flags::N | Flags::C | Flags::V, false);
    m.regs.flags.set(Flags::Z, true);
    bus.store_byte(m, 0)?;
    Ok(())
}
#[inline]
fn do_bit<A: Bus, M: MemoryIO>(mut bus: A, m: &mut Machine<M>, val: u8) -> CpuResult<u8> {
    let operand = bus.fetch_operand(m)?;
    let new_val = val & operand;
    m.regs.flags.set_nz(new_val);
    m.regs.flags.clc();
    Ok(new_val)
}

#[inline]
fn bit_a<A: Bus, M: MemoryIO>(bus: A, m: &mut Machine<M>) -> CpuResult<()> {
    m.regs.a = do_bit(bus, m, m.regs.a)?;
    Ok(())
}

#[inline]
fn bit_b<A: Bus, M: MemoryIO>(bus: A, m: &mut Machine<M>) -> CpuResult<()> {
    m.regs.b = do_bit(bus, m, m.regs.b)?;
    Ok(())
}

#[inline]
fn daa<A: Bus, M: MemoryIO>(_bus: A, _machine: &mut Machine<M>) -> CpuResult<()> {
    panic!()
}

#[inline]
fn tst<A: Bus, M: MemoryIO>(mut bus: A, m: &mut Machine<M>) -> CpuResult<()> {
    let val = bus.fetch_operand(m)?;
    m.regs.flags.set_nz(val);
    m.regs.flags.set(Flags::V | Flags::C, false);
    Ok(())
}
