//! Instruction execution — one cycle per instruction (MAME's model;
//! pipeline/wait states not modelled, except the +3 on taken branches).
//!
//! Semantics are a direct port of MAME's jaguar_cpu_device handlers
//! (`src/devices/cpu/jaguar/jaguar.cpp`), the empirical ground truth for
//! the JRISC cores. Notable quirks preserved:
//! - N flag = **bit 29** of the result,
//! - `convert_zero`: quick immediates use 0 -> 32,
//! - `shlq` amount = 32 - raw,
//! - `ror`/`rorq` carry from bit 30; `shrq`/`sharq`/`sh` carry from bit 1,
//! - `loadb`/`loadw`/`storeb`/`storew` on internal RAM hit the aligned long,
//! - `loadr14d`/`loadr15d` displacement is `4 * convert_zero(n)` (in longs),
//! - `div` by zero -> 0xFFFFFFFF (remainder 0xFFFFFFFF),
//! - `abs` of 0x80000000 sets N only (Z/C untouched).

use crate::cpu::{Cpu, DecodedInsn};
use crate::mem::JriscBus;

/// `convert_zero`: quick-immediate fields where raw 0 means 32.
fn convert_zero(n: usize) -> u32 {
    if n == 0 { 32 } else { n as u32 }
}

fn in_internal(cpu: &Cpu, a: u32) -> bool {
    let (base, len) = (cpu.chip.ram_base(), cpu.chip.ram_size());
    a >= base && a < base + len
}

/// Store semantics: byte/word stores on internal RAM write the aligned long.
fn store_value<B: JriscBus + ?Sized>(cpu: &mut Cpu, addr: u32, width: u8, v: u32, bus: &mut B) {
    if in_internal(cpu, addr) {
        bus.write_long(addr & !3, v);
    } else {
        match width {
            1 => bus.write_byte(addr, v),
            2 => bus.write_word(addr, v),
            _ => bus.write_long(addr, v),
        }
    }
}

/// Execute a non-branch instruction (`jump`/`jr` are handled by
/// [`Cpu::step`] because of the branch delay slot).
pub fn execute<B: JriscBus + ?Sized>(cpu: &mut Cpu, d: &DecodedInsn, bus: &mut B) -> Result<(), String> {
    let src = d.src as usize;
    let dst = d.dst as usize;
    let mn = d.insn.mnemonic;
    match mn {
        // ---- arithmetic ----
        // cmpq first: the T2K DSP main loop is a cmpq/jr spin (measured
        // ~50% of the executed stream), so it must win in one compare.
        "cmpq" => {
            // 5-bit signed immediate, sign-extended (MAME: (s8)(op>>2)>>3)
            let r1 = ((src << 27) as i32 >> 27) as u32;
            let r2 = cpu.r(dst);
            let r = r2.wrapping_sub(r1);
            cpu.flags.set_znc_sub(r2, r1, r);
        }
        "add" => {
            let r2 = cpu.r(dst);
            let r1 = cpu.r(src);
            let r = r2.wrapping_add(r1);
            cpu.w(dst, r);
            cpu.flags.set_znc_add(r2, r1, r);
        }
        "addc" => {
            let r2 = cpu.r(dst);
            let r1 = cpu.r(src);
            let c = cpu.flags.c as u32;
            let r = r2.wrapping_add(r1).wrapping_add(c);
            cpu.w(dst, r);
            cpu.flags.set_znc_add(r2, r1.wrapping_add(c), r);
        }
        "addq" => {
            let r2 = cpu.r(dst);
            let r1 = convert_zero(src);
            let r = r2.wrapping_add(r1);
            cpu.w(dst, r);
            cpu.flags.set_znc_add(r2, r1, r);
        }
        "addqmod" => {
            let r2 = cpu.r(dst);
            let r1 = convert_zero(src);
            let mut r = r2.wrapping_add(r1);
            r = (r & !cpu.modulo) | (r2 & cpu.modulo);
            cpu.w(dst, r);
            cpu.flags.set_znc_add(r2, r1, r);
        }
        "addqt" => {
            let r = cpu.r(dst).wrapping_add(convert_zero(src));
            cpu.w(dst, r);
        }
        "sub" => {
            let r2 = cpu.r(dst);
            let r1 = cpu.r(src);
            let r = r2.wrapping_sub(r1);
            cpu.w(dst, r);
            cpu.flags.set_znc_sub(r2, r1, r);
        }
        "subc" => {
            let r2 = cpu.r(dst);
            let r1 = cpu.r(src);
            let c = cpu.flags.c as u32;
            let r = r2.wrapping_sub(r1).wrapping_sub(c);
            cpu.w(dst, r);
            cpu.flags.set_znc_sub(r2, r1.wrapping_add(c), r);
        }
        "subq" => {
            let r2 = cpu.r(dst);
            let r1 = convert_zero(src);
            let r = r2.wrapping_sub(r1);
            cpu.w(dst, r);
            cpu.flags.set_znc_sub(r2, r1, r);
        }
        "subqmod" => {
            let r2 = cpu.r(dst);
            let r1 = convert_zero(src);
            let mut r = r2.wrapping_sub(r1);
            r = (r & !cpu.modulo) | (r2 & cpu.modulo);
            cpu.w(dst, r);
            cpu.flags.set_znc_sub(r2, r1, r);
        }
        "subqt" => {
            let r = cpu.r(dst).wrapping_sub(convert_zero(src));
            cpu.w(dst, r);
        }
        "neg" => {
            let r2 = cpu.r(dst);
            let r = r2.wrapping_neg();
            cpu.w(dst, r);
            cpu.flags.set_znc_sub(0, r2, r);
        }
        "abs" => {
            let r2 = cpu.r(dst) as i32;
            if r2 == i32::MIN {
                // quirk: does not work for 0x80000000 — N only
                cpu.flags.n = true;
            } else {
                cpu.flags.z = false;
                cpu.flags.c = false;
                cpu.flags.n = false;
                cpu.flags.c = (r2 as u32 >> 31) & 1 != 0;
                cpu.w(dst, r2.unsigned_abs());
                cpu.flags.z = cpu.r(dst) == 0;
            }
        }
        "cmp" => {
            let r1 = cpu.r(src);
            let r2 = cpu.r(dst);
            let r = r2.wrapping_sub(r1);
            cpu.flags.set_znc_sub(r2, r1, r);
        }
        "div" => {
            let r1 = cpu.r(src);
            let r2 = cpu.r(dst);
            if r1 != 0 {
                if cpu.div_offset {
                    let num = (r2 as u64) << 16;
                    cpu.w(dst, (num / r1 as u64) as u32);
                    cpu.div_remainder = (num % r1 as u64) as u32;
                } else {
                    cpu.w(dst, r2 / r1);
                    cpu.div_remainder = r2 % r1;
                }
            } else {
                cpu.w(dst, 0xffff_ffff);
                cpu.div_remainder = 0xffff_ffff;
            }
        }
        // ---- logic / bit ----
        "and" => {
            let r = cpu.r(dst) & cpu.r(src);
            cpu.w(dst, r);
            cpu.flags.set_zn(r);
        }
        "or" => {
            let r = cpu.r(dst) | cpu.r(src);
            cpu.w(dst, r);
            cpu.flags.set_zn(r);
        }
        "xor" => {
            let r = cpu.r(dst) ^ cpu.r(src);
            cpu.w(dst, r);
            cpu.flags.set_zn(r);
        }
        "not" => {
            let r = !cpu.r(dst);
            cpu.w(dst, r);
            cpu.flags.set_zn(r);
        }
        "btst" => {
            let bit = src & 31;
            cpu.flags.z = (cpu.r(dst) >> bit) & 1 == 0;
        }
        "bset" => {
            let r = cpu.r(dst) | (1 << (src & 31));
            cpu.w(dst, r);
            cpu.flags.set_zn(r);
        }
        "bclr" => {
            let r = cpu.r(dst) & !(1 << (src & 31));
            cpu.w(dst, r);
            cpu.flags.set_zn(r);
        }
        "mirror" => {
            let r = cpu.r(dst);
            let res = (mirror16((r & 0xffff) as u16) as u32) << 16 | mirror16((r >> 16) as u16) as u32;
            cpu.w(dst, res);
            cpu.flags.set_zn(res);
        }
        // ---- shifts / rotates ----
        "sh" | "sha" => {
            let r1 = cpu.r(src) as i32;
            let r2 = cpu.r(dst);
            cpu.flags.z = false;
            cpu.flags.c = false;
            cpu.flags.n = false;
            let res = if r1 < 0 {
                if r1 <= -32 {
                    0
                } else {
                    cpu.flags.c = (r2 >> 30) & 1 != 0;
                    r2.wrapping_shl((-r1) as u32 & 31)
                }
            } else {
                if r1 >= 32 {
                    if mn == "sha" {
                        ((r2 as i32) >> 31) as u32
                    } else {
                        0
                    }
                } else {
                    cpu.flags.c = (r2 << 1) & 2 != 0;
                    if mn == "sha" {
                        ((r2 as i32) >> r1) as u32
                    } else {
                        r2 >> r1
                    }
                }
            };
            cpu.w(dst, res);
            cpu.flags.set_zn(res);
        }
        "shlq" => {
            // amount = 32 - raw; raw 0 -> 32 (MAME note: convert_zero not used)
            let amt = (32 - src as u32) & 63;
            let r2 = cpu.r(dst);
            cpu.flags.z = false;
            cpu.flags.c = false;
            cpu.flags.n = false;
            let res = if amt >= 32 { 0 } else { r2 << amt };
            cpu.flags.c = (r2 >> 30) & 1 != 0;
            cpu.w(dst, res);
            cpu.flags.set_zn(res);
        }
        "shrq" => {
            let amt = convert_zero(src);
            let r2 = cpu.r(dst);
            cpu.flags.z = false;
            cpu.flags.c = false;
            cpu.flags.n = false;
            let res = if amt >= 32 { 0 } else { r2 >> amt };
            cpu.flags.c = (r2 << 1) & 2 != 0;
            cpu.w(dst, res);
            cpu.flags.set_zn(res);
        }
        "sharq" => {
            let amt = convert_zero(src);
            let r2 = cpu.r(dst);
            cpu.flags.z = false;
            cpu.flags.c = false;
            cpu.flags.n = false;
            let res = if amt >= 32 {
                ((r2 as i32) >> 31) as u32
            } else {
                ((r2 as i32) >> amt) as u32
            };
            cpu.flags.c = (r2 << 1) & 2 != 0;
            cpu.w(dst, res);
            cpu.flags.set_zn(res);
        }
        "ror" => {
            let r1 = cpu.r(src) & 31;
            let r2 = cpu.r(dst);
            let res = r2.rotate_right(r1);
            cpu.flags.z = false;
            cpu.flags.c = false;
            cpu.flags.n = false;
            cpu.flags.c = (r2 >> 30) & 1 != 0;
            cpu.w(dst, res);
            cpu.flags.set_zn(res);
        }
        "rorq" => {
            let r1 = convert_zero(src);
            let r2 = cpu.r(dst);
            let res = r2.rotate_right(r1);
            cpu.flags.z = false;
            cpu.flags.c = false;
            cpu.flags.n = false;
            cpu.flags.c = (r2 >> 30) & 1 != 0;
            cpu.w(dst, res);
            cpu.flags.set_zn(res);
        }
        // ---- multiply / MAC ----
        "mult" => {
            let r = (cpu.r(src) as u16 as u32) * (cpu.r(dst) as u16 as u32);
            cpu.w(dst, r);
            cpu.flags.set_zn(r);
        }
        "imult" => {
            let r = (cpu.r(src) as i16 as i32) * (cpu.r(dst) as i16 as i32);
            cpu.w(dst, r as u32);
            cpu.flags.set_zn(r as u32);
        }
        "imultn" => {
            let r = (cpu.r(src) as i16 as i32) * (cpu.r(dst) as i16 as i32);
            cpu.w(dst, r as u32);
            cpu.accum = r as i64;
            cpu.flags.set_zn(r as u32);
        }
        "imacn" => {
            cpu.accum += (cpu.r(src) as i16 as i64) * (cpu.r(dst) as i16 as i64);
        }
        "resmac" => {
            cpu.w(dst, cpu.accum as u32);
        }
        "mtoi" => {
            let r1 = cpu.r(src);
            cpu.w(dst, (((r1 as i32) >> 8) as u32 & 0xff80_0000) | (r1 & 0x007f_ffff));
        }
        "normi" => {
            let mut r1 = cpu.r(src);
            let mut res = 0i32;
            if r1 != 0 {
                while (r1 & 0xffc0_0000) == 0 {
                    r1 <<= 1;
                    res -= 1;
                }
                while (r1 & 0xff80_0000) != 0 {
                    r1 >>= 1;
                    res += 1;
                }
            }
            cpu.w(dst, res as u32);
            cpu.flags.set_zn(res as u32);
        }
        "mmult" => {
            let count = cpu.mtx_width & 0xf;
            let sreg = src;
            let mut addr = cpu.mtx_addr;
            let mut accum = 0i64;
            let step = if cpu.mtx_addw { 4 * count as u32 } else { 4 };
            for i in 0..count as usize {
                let a = if i & 1 != 0 {
                    (cpu.regs.bank_get(1, (sreg + (i >> 1)) & 31) >> 16) as i16
                } else {
                    cpu.regs.bank_get(1, (sreg + (i >> 1)) & 31) as u16 as i16
                };
                let b = bus.read_word(addr + 2) as u16 as i16;
                accum += a as i64 * b as i64;
                addr = addr.wrapping_add(step);
            }
            let res = accum as u32;
            cpu.w(dst, res);
            cpu.flags.set_zn(res);
        }
        // ---- saturation / pack ----
        "sat8" => {
            let r = cpu.r(dst) as i32;
            let res = if r < 0 { 0 } else if r > 255 { 255 } else { r };
            cpu.w(dst, res as u32);
            cpu.flags.set_zn(res as u32);
        }
        "sat16" => {
            let r = cpu.r(dst) as i32;
            let res = if r < 0 { 0 } else if r > 65535 { 65535 } else { r };
            cpu.w(dst, res as u32);
            cpu.flags.set_zn(res as u32);
        }
        "sat16s" => {
            let r = cpu.r(dst) as i32;
            let res = if r < -32768 { -32768 } else if r > 32767 { 32767 } else { r };
            cpu.w(dst, res as u32);
            cpu.flags.set_zn(res as u32);
        }
        "sat24" => {
            let r = cpu.r(dst) as i32;
            let res = if r < 0 { 0 } else if r > 16_777_215 { 16_777_215 } else { r };
            cpu.w(dst, res as u32);
            cpu.flags.set_zn(res as u32);
        }
        "sat32s" => {
            let r2 = cpu.r(dst);
            let temp = (cpu.accum >> 32) as i32;
            let res = if temp < -1 {
                0x8000_0000u32
            } else if temp > 0 {
                0x7fff_ffffu32
            } else {
                r2
            };
            cpu.w(dst, res);
            cpu.flags.set_zn(res);
        }
        "pack" | "unpack" => {
            // mode in the src field: 0 = PACK, else UNPACK (MAME pack_rn)
            let r2 = cpu.r(dst);
            let res = if src == 0 {
                ((r2 >> 10) & 0xf000) | ((r2 >> 5) & 0x0f00) | (r2 & 0xff)
            } else {
                ((r2 & 0xf000) << 10) | ((r2 & 0x0f00) << 5) | (r2 & 0xff)
            };
            cpu.w(dst, res);
        }
        // ---- moves ----
        "move" => {
            let v = cpu.r(src);
            cpu.w(dst, v);
        }
        "moveq" => {
            cpu.w(dst, src as u32);
        }
        "movei" => {
            cpu.w(dst, d.extra.unwrap_or(0));
        }
        "moveta" => {
            let v = cpu.r(src);
            cpu.regs.alt_set(dst, v);
        }
        "movefa" => {
            let v = cpu.regs.alt_get(src);
            cpu.w(dst, v);
        }
        "movepc" => {
            cpu.w(dst, cpu.ppc);
        }
        "nop" => {}
        // ---- loads / stores ----
        "load" | "loadp" => {
            let addr = cpu.r(src);
            let v = if in_internal(cpu, addr) {
                bus.read_long(addr & !3)
            } else {
                if mn == "loadp" {
                    cpu.hidata = bus.read_long(addr);
                    bus.read_long(addr + 4)
                } else {
                    bus.read_long(addr)
                }
            };
            cpu.w(dst, v);
        }
        "loadb" | "loadw" => {
            let addr = cpu.r(src);
            let v = if in_internal(cpu, addr) {
                bus.read_long(addr & !3)
            } else if mn == "loadb" {
                bus.read_byte(addr)
            } else {
                bus.read_word(addr)
            };
            cpu.w(dst, v);
        }
        "loadr14d" | "loadr15d" => {
            let base = cpu.r(if mn == "loadr14d" { 14 } else { 15 });
            let addr = base.wrapping_add(4 * convert_zero(src));
            let v = bus.read_long(addr);
            cpu.w(dst, v);
        }
        "loadr14r" | "loadr15r" => {
            let base = cpu.r(if mn == "loadr14r" { 14 } else { 15 });
            let addr = base.wrapping_add(cpu.r(src));
            let v = bus.read_long(addr);
            cpu.w(dst, v);
        }
        "store" | "storep" => {
            let addr = cpu.r(src);
            let v = cpu.r(dst);
            if in_internal(cpu, addr) {
                bus.write_long(addr & !3, v);
            } else if mn == "storep" {
                bus.write_long(addr, cpu.hidata);
                bus.write_long(addr + 4, v);
            } else {
                bus.write_long(addr, v);
            }
        }
        "storeb" | "storew" => {
            let addr = cpu.r(src);
            let v = cpu.r(dst);
            store_value(cpu, addr, if mn == "storeb" { 1 } else { 2 }, v, bus);
        }
        "storer14d" | "storer15d" => {
            let base = cpu.r(if mn == "storer14d" { 14 } else { 15 });
            let addr = base.wrapping_add(4 * convert_zero(src));
            bus.write_long(addr, cpu.r(dst));
        }
        "storer14r" | "storer15r" => {
            let base = cpu.r(if mn == "storer14r" { 14 } else { 15 });
            let addr = base.wrapping_add(cpu.r(src));
            bus.write_long(addr, cpu.r(dst));
        }
        other => return Err(format!("unimplemented instruction: {other}")),
    }
    Ok(())
}

/// 16-bit bit reversal (MAME mirror_table).
fn mirror16(v: u16) -> u16 {
    v.reverse_bits()
}
