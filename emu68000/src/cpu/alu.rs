//! Instruction execution for the base M68000.
//!
//! Semantics follow the M68000 PRM as interpreted by MAME's Musashi core
//! (the same ground truth as the decode table). Notable rules preserved:
//! - byte/word ALU ops on data registers affect only the low 8/16 bits
//!   (upper bits unchanged); MOVE/MOVEA/MOVEM/MOVEQ write the whole reg
//! - MOVE to Dn zero-extends; MOVEA.W/ADDA.W/SUBA.W/CMPA.W sign-extend
//! - A7 steps by 2 for byte post/pre-increment
//! - MOVEM predecrement: A7 first decremented by 2*n; registers stored in
//!   reverse order; A7's own stored value is the new (decremented) SP
//! - DBcc: if the condition is true no branch; else decrement the word
//!   counter and branch while it is != $FFFF
//! - ADDX/SUBX/ABCD/SBCD use a "sticky" Z (cleared, never set)
//! - X is only modified by ASd/ROXd (and addx-family/negx/abcd/sbcd);
//!   LSL/LSR/ROL/ROR leave X alone
//! - register-count shifts with count 0 leave C/X alone
//!
//! Cycle accounting: `insn.cycles` (PRM base) plus an EA-access extra
//! (simple table). Precise PRM timing (bus cycles, flag-dependent +2s) is
//! a follow-up; state semantics are the priority here.

use crate::cpu::bus::M68kBus;
use crate::cpu::cpucore::{ea_addr, read_ea, size_bytes, write_ea, write_reg, Cpu};
use crate::cpu::decoder::{DecodedInsn, Ea};
use crate::isa::Form;
use crate::isa::UNKNOWN;

/// Run one decoded instruction; returns cycle count.
pub fn execute<B: M68kBus + ?Sized>(
    cpu: &mut Cpu,
    d: &DecodedInsn,
    bus: &mut B,
) -> Result<u32, crate::cpu::ExecError> {
    let size = size_bytes(d.insn.size);
    let mn = d.insn.mnemonic;
    let cycles = d.insn.cycles as u32 + ea_extra(d);

    match d.insn.form {
        Form::EaReg => {
            // <ea> -> Dn: or/sub/cmp/and/add/mulu/muls/divu/divs/chk
            let e = d.ea.as_ref().unwrap();
            let src = read_ea(cpu, bus, d, e, size)?;
            let dst = cpu.regs.d[d.reg1 as usize];
            match mn {
                "add" => alu_add(cpu, size, src, dst, d.reg1),
                "sub" => alu_sub(cpu, size, src, dst, d.reg1),
                "cmp" => alu_cmp(cpu, size, src, dst),
                "or" => alu_logic(cpu, size, src, dst, d.reg1, 1),
                "and" => alu_logic(cpu, size, src, dst, d.reg1, 0),
                "mulu" | "muls" => {
                    let a = (dst & 0xFFFF) as u16;
                    let b = (src & 0xFFFF) as u16;
                    let r = if mn == "mulu" {
                        (a as u32) * (b as u32)
                    } else {
                        (a as i16 as i32 as u32).wrapping_mul(b as i16 as i32 as u32)
                    };
                    cpu.regs.d[d.reg1 as usize] = r;
                    cpu.regs.set_zn(4, r);
                    cpu.regs.set_v(false);
                    cpu.regs.set_c(false);
                }
                "divu" | "divs" => {
                    let a = (dst & 0xFFFF) as u16;
                    let b = (src & 0xFFFF) as u16;
                    if b == 0 {
                        return Err(crate::cpu::ExecError {
                            message: "divide by zero".into(),
                        });
                    }
                    let ovf = if mn == "divs" {
                        let aa = a as i16 as i32;
                        let bb = b as i16 as i32;
                        let q = aa / bb;
                        q > 0x7FFF || q < -0x8000
                    } else {
                        false
                    };
                    if ovf {
                        cpu.regs.set_v(true);
                    } else {
                        let (q, r) = if mn == "divu" {
                            ((a as u32) / (b as u32), (a as u32) % (b as u32))
                        } else {
                            let aa = a as i16 as i32;
                            let bb = b as i16 as i32;
                            let q = aa / bb;
                            let r = aa % bb;
                            (q as u16 as u32, r as u16 as u32)
                        };
                        cpu.regs.d[d.reg1 as usize] = (r << 16) | (q & 0xFFFF);
                        cpu.regs.set_zn(4, q & 0xFFFF);
                        cpu.regs.set_v(false);
                        cpu.regs.set_c(false);
                    }
                }
                "chk" => {
                    let a = (dst & 0xFFFF) as i16 as i32;
                    let bound = (src & 0xFFFF) as i16 as i32;
                    if a < 0 || a > bound {
                        cpu.regs.set_n(a < 0);
                        cpu.regs.set_v(false);
                        cpu.regs.set_c(false);
                        cpu.regs.set_z(false);
                        return Err(crate::cpu::ExecError {
                            message: "chk: out of range".into(),
                        });
                    }
                    cpu.regs.set_n(false);
                    cpu.regs.set_v(false);
                    cpu.regs.set_c(false);
                    cpu.regs.set_z(false);
                }
                "move" => {
                    // <ea>,CCR / <ea>,SR (form ea_reg, dst reg is not used)
                    if d.insn.dst == "M_AM_CCR" {
                        cpu.regs.sr = (cpu.regs.sr & 0xFF00) | (src as u16 & 0xFF);
                    } else if d.insn.dst == "M_AM_SR" {
                        cpu.regs.sr = src as u16;
                    }
                }
                _ => {
                    return Err(crate::cpu::ExecError {
                        message: format!("unimplemented: {mn}"),
                    })
                }
            }
        }
        Form::RegEa => {
            // Dn -> <ea>: or/sub/and/add/eor + bit ops + move SR/CCR
            let e = d.ea.as_ref().unwrap();
            let src = cpu.regs.d[d.reg1 as usize];
            match mn {
                "add" | "sub" | "or" | "and" | "eor" => {
                    let dst = read_ea(cpu, bus, d, e, size)?;
                    let r = match mn {
                        "add" => alu_add_ret(cpu, size, src, dst),
                        "sub" => alu_sub_ret(cpu, size, src, dst),
                        "or" => alu_logic_ret(cpu, size, src, dst, 1),
                        "and" => alu_logic_ret(cpu, size, src, dst, 0),
                        _ => alu_logic_ret(cpu, size, src, dst, 2),
                    };
                    write_ea(cpu, bus, d, e, size, r)?;
                }
                "btst" | "bchg" | "bclr" | "bset" => {
                    let bit = src & 31;
                    bit_op(cpu, bus, d, e, size, bit, mn)?;
                }
                "move" => {
                    // SR,<ea> / CCR,<ea>
                    let v = if d.insn.src == "M_AM_SR" {
                        cpu.regs.sr as u32
                    } else {
                        cpu.regs.sr as u32 & 0xFF
                    };
                    write_ea(cpu, bus, d, e, size, v)?;
                }
                _ => {
                    return Err(crate::cpu::ExecError {
                        message: format!("unimplemented: {mn}"),
                    })
                }
            }
        }
        Form::EaAddr => {
            let e = d.ea.as_ref().unwrap();
            let src = read_ea(cpu, bus, d, e, size)?;
            let an = cpu.regs.read_a(d.reg1 as usize);
            match mn {
                "adda" | "suba" => {
                    let s = if size == 2 {
                        src as i16 as i32 as u32
                    } else {
                        src
                    };
                    let r = if mn == "adda" {
                        an.wrapping_add(s)
                    } else {
                        an.wrapping_sub(s)
                    };
                    cpu.set_a(d.reg1 as usize, r);
                }
                "cmpa" => {
                    let s = if size == 2 {
                        src as i16 as i32 as u32
                    } else {
                        src
                    };
                    let r = an.wrapping_sub(s);
                    // compare flags like a 32-bit sub
                    let rr = r;
                    cpu.regs.set_zn(4, rr);
                    cpu.regs.set_v(false);
                    cpu.regs.set_c((an as i64 - s as i64) < 0);
                    let o = ((an ^ s) & (an ^ rr)) & 0x8000_0000 != 0;
                    cpu.regs.set_v(o);
                }
                "lea" => {
                    let a = ea_addr(cpu, bus, d, e)?;
                    cpu.set_a(d.reg1 as usize, a);
                }
                "movea" => {
                    let v = if size == 2 {
                        src as i16 as i32 as u32
                    } else {
                        src
                    };
                    cpu.set_a(d.reg1 as usize, v);
                }
                _ => {
                    return Err(crate::cpu::ExecError {
                        message: format!("unimplemented: {mn}"),
                    })
                }
            }
        }
        Form::RegReg => {
            let s = cpu.regs.d[d.reg2 as usize];
            let dst = cpu.regs.d[d.reg1 as usize];
            match mn {
                "addx" | "subx" => {
                    let x = cpu.regs.x() as u32;
                    let r = if mn == "addx" {
                        alu_addx_ret(cpu, size, s, dst, x)
                    } else {
                        alu_subx_ret(cpu, size, s, dst, x)
                    };
                    cpu.regs.d[d.reg1 as usize] = (dst & !mask_of(size)) | (r & mask_of(size));
                }
                "abcd" | "sbcd" => {
                    let x = cpu.regs.x() as u32;
                    let a = (s & 0x0F) + (s >> 4) * 10;
                    let b = (dst & 0x0F) + (dst >> 4) * 10;
                    let (r, c) = if mn == "abcd" {
                        let t = a + b + x;
                        (t, t > 99)
                    } else {
                        let t = (b as i32) - (a as i32) - (x as i32);
                        (t as u32 & 0xFF, t < 0)
                    };
                    let d0 = (r % 10) | ((r / 10) % 10) << 4;
                    cpu.regs.d[d.reg1 as usize] = (dst & 0xFFFFFF00) | d0;
                    cpu.regs.set_c(c);
                    cpu.regs.set_x(c);
                    if r != 0 {
                        cpu.regs.set_z(false); // sticky
                    }
                    cpu.regs.set_n(false);
                    cpu.regs.set_v(false);
                }
                "exg" => {
                    let r1 = cpu.r(d.reg1 as usize);
                    let r2 = cpu.r(d.reg2 as usize);
                    cpu.w(d.reg1 as usize, r2);
                    cpu.w(d.reg2 as usize, r1);
                }
                _ => {
                    return Err(crate::cpu::ExecError {
                        message: format!("unimplemented: {mn}"),
                    })
                }
            }
        }
        Form::PredecPredec => {
            // abcd/sbcd/addx/subx -(Ay),-(Ax)
            let size2 = size_bytes(d.insn.size);
            let ay = cpu.regs.read_a(d.reg2 as usize) - step(d.reg2, size2);
            cpu.set_a(d.reg2 as usize, ay);
            let ax = cpu.regs.read_a(d.reg1 as usize) - step(d.reg1, size2);
            cpu.set_a(d.reg1 as usize, ax);
            let s = read_mem(bus, ay, size2);
            let dst = read_mem(bus, ax, size2);
            match mn {
                "addx" | "subx" => {
                    let x = cpu.regs.x() as u32;
                    let r = if mn == "addx" {
                        alu_addx_ret(cpu, size2, s, dst, x)
                    } else {
                        alu_subx_ret(cpu, size2, s, dst, x)
                    };
                    write_mem(bus, ax, size2, r)?;
                }
                "abcd" | "sbcd" => {
                    let x = cpu.regs.x() as u32;
                    let a = (s & 0x0F) + (s >> 4) * 10;
                    let b = (dst & 0x0F) + (dst >> 4) * 10;
                    let (r, c) = if mn == "abcd" {
                        let t = a + b + x;
                        (t, t > 99)
                    } else {
                        let t = (b as i32) - (a as i32) - (x as i32);
                        (t as u32 & 0xFF, t < 0)
                    };
                    let d0 = (r % 10) | ((r / 10) % 10) << 4;
                    write_mem(bus, ax, 1, d0)?;
                    cpu.regs.set_c(c);
                    cpu.regs.set_x(c);
                    if r != 0 {
                        cpu.regs.set_z(false);
                    }
                    cpu.regs.set_n(false);
                    cpu.regs.set_v(false);
                }
                _ => {
                    return Err(crate::cpu::ExecError {
                        message: format!("unimplemented: {mn}"),
                    })
                }
            }
        }
        Form::PostincPostinc => {
            // cmpm (Ay)+,(Ax)+
            let size2 = size_bytes(d.insn.size);
            let ay = cpu.regs.read_a(d.reg2 as usize);
            cpu.set_a(d.reg2 as usize, ay + step(d.reg2, size2));
            let ax = cpu.regs.read_a(d.reg1 as usize);
            cpu.set_a(d.reg1 as usize, ax + step(d.reg1, size2));
            let s = read_mem(bus, ay, size2);
            let dst = read_mem(bus, ax, size2);
            alu_cmp(cpu, size2, s, dst);
        }
        Form::ImmEa => {
            let imm = d.imm.unwrap_or(0) & mask_of(size);
            let e = d.ea.as_ref().unwrap();
            match mn {
                "ori" | "andi" | "subi" | "addi" | "eori" | "cmpi" => {
                    let dst = read_ea(cpu, bus, d, e, size)?;
                    let r = match mn {
                        "addi" => alu_add_ret(cpu, size, imm, dst),
                        "subi" => alu_sub_ret(cpu, size, imm, dst),
                        "ori" => alu_logic_ret(cpu, size, imm, dst, 1),
                        "andi" => alu_logic_ret(cpu, size, imm, dst, 0),
                        "eori" => alu_logic_ret(cpu, size, imm, dst, 2),
                        _ => 0xFFFF_FFFF, // cmpi: no write
                    };
                    if r != 0xFFFF_FFFF {
                        write_ea(cpu, bus, d, e, size, r)?;
                    } else {
                        alu_cmp(cpu, size, imm, dst);
                    }
                }
                _ => {
                    return Err(crate::cpu::ExecError {
                        message: format!("unimplemented: {mn}"),
                    })
                }
            }
        }
        Form::BitImmEa => {
            let bit = d.imm.unwrap_or(0) & 31;
            let e = d.ea.as_ref().unwrap();
            bit_op(cpu, bus, d, e, size, bit, mn)?;
        }
        Form::ImmOnly => {
            let imm = d.imm.unwrap_or(0) as u16;
            match mn {
                "ori" | "andi" | "eori" => {
                    let (mask, which) = if d.insn.dst == "M_AM_CCR" {
                        (0x00FF, 0)
                    } else {
                        (0xFFFF, 1)
                    };
                    let v = match mn {
                        "ori" => cpu.regs.sr | imm,
                        "andi" => cpu.regs.sr & imm,
                        _ => cpu.regs.sr ^ imm,
                    };
                    if which == 0 {
                        cpu.regs.sr = (cpu.regs.sr & !0xFF) | (v & 0xFF);
                    } else {
                        cpu.regs.sr = v;
                    }
                    let _ = mask;
                }
                _ => {
                    return Err(crate::cpu::ExecError {
                        message: format!("unimplemented: {mn}"),
                    })
                }
            }
        }
        Form::Move => {
            let e = d.ea.as_ref().unwrap();
            let ed = d.ea_dst.as_ref().unwrap();
            let src = read_ea(cpu, bus, d, e, size)?;
            match ed.mode {
                0 => write_reg(cpu, ed.reg, size, src),
                1 => {
                    let v = if size == 2 {
                        src as i16 as i32 as u32
                    } else {
                        src
                    };
                    cpu.set_a(ed.reg as usize, v);
                }
                _ => {
                    // memory destination: postincrement handled in write_ea
                    write_ea(cpu, bus, d, ed, size, src)?;
                }
            }
            cpu.regs.set_zn(size, src);
            cpu.regs.set_v(false);
            cpu.regs.set_c(false);
        }
        Form::Moveq => {
            let v = (d.word & 0xFF) as u8 as i8 as i32 as u32;
            cpu.regs.d[d.reg1 as usize] = v;
            cpu.regs.set_zn(4, v);
            cpu.regs.set_v(false);
            cpu.regs.set_c(false);
        }
        Form::Addq => {
            let q = if d.reg1 == 0 { 8 } else { d.reg1 } as u32;
            let e = d.ea.as_ref().unwrap();
            if e.mode == 1 {
                // An destination: full 32-bit, no flags
                let an = cpu.regs.read_a(e.reg as usize);
                let r = if mn == "addq" {
                    an.wrapping_add(q)
                } else {
                    an.wrapping_sub(q)
                };
                cpu.set_a(e.reg as usize, r);
            } else {
                let dst = read_ea(cpu, bus, d, e, size)?;
                let r = if mn == "addq" {
                    alu_add_ret(cpu, size, q & mask_of(size), dst)
                } else {
                    alu_sub_ret(cpu, size, q & mask_of(size), dst)
                };
                write_ea(cpu, bus, d, e, size, r)?;
            }
        }
        Form::ShiftImm | Form::ShiftReg => {
            let count = if d.insn.form == Form::ShiftImm {
                (if d.reg1 == 0 { 8 } else { d.reg1 }) as u32
            } else {
                cpu.regs.d[d.reg1 as usize] & 0x3F
            };
            let dst = cpu.regs.d[d.reg2 as usize];
            let r = shift_op(cpu, mn, size, dst, count)?;
            cpu.regs.d[d.reg2 as usize] = (dst & !mask_of(size)) | (r & mask_of(size));
        }
        Form::ShiftEa => {
            let count = 1;
            let e = d.ea.as_ref().unwrap();
            let dst = read_ea(cpu, bus, d, e, 2)?;
            let r = shift_op(cpu, mn, 2, dst, count)?;
            write_ea(cpu, bus, d, e, 2, r)?;
        }
        Form::Bcc8 | Form::Bcc16 => {
            let cc = (d.word >> 8) & 0xF;
            let target = d.label.unwrap_or(0);
            match mn {
                "bsr" => {
                    cpu.push_long(bus, cpu.regs.pc);
                    cpu.regs.pc = target;
                }
                _ => {
                    if cpu.regs.cond(cc as u8) {
                        cpu.regs.pc = target;
                    }
                }
            }
        }
        Form::Dbcc => {
            let cc = (d.word >> 8) & 0xF;
            let target = d.label.unwrap_or(0);
            if !cpu.regs.cond(cc as u8) {
                let v = cpu.regs.d[d.reg2 as usize].wrapping_sub(1) & 0xFFFF;
                cpu.regs.d[d.reg2 as usize] = (cpu.regs.d[d.reg2 as usize] & 0xFFFF_0000) | v;
                if v != 0xFFFF {
                    cpu.regs.pc = target;
                }
            }
        }
        Form::Scc => {
            let cc = (d.word >> 8) & 0xF;
            let v: u32 = if cpu.regs.cond(cc as u8) { 0xFF } else { 0 };
            let e = d.ea.as_ref().unwrap();
            write_ea(cpu, bus, d, e, 1, v)?;
        }
        Form::MovemRe | Form::MovemEr | Form::MovemPd => {
            movem_op(cpu, bus, d, d.insn.form == Form::MovemPd)?;
        }
        Form::Link => {
            // push An; SP -> An; SP += disp (An keeps the pre-disp SP)
            let an = d.reg2 as usize;
            let v = cpu.regs.read_a(an);
            cpu.push_long(bus, v);
            let sp = cpu.regs.sp();
            cpu.set_a(an, sp);
            let disp = d.imm.unwrap_or(0) as i16 as i32 as u32;
            cpu.regs.set_sp(sp.wrapping_add(disp));
        }
        Form::Movep => {
            let an = cpu.regs.read_a(d.reg2 as usize) + (d.imm.unwrap_or(0) & 0xFFFF);
            let er = d.insn.src == "M_AIND";
            let bytes = size_bytes(d.insn.size);
            let dn = cpu.regs.d[d.reg1 as usize];
            for i in 0..bytes {
                let addr = an + 2 * i as u32;
                if er {
                    let b = read_mem(bus, addr, 1);
                    let shift = 8 * (bytes - 1 - i);
                    cpu.regs.d[d.reg1 as usize] =
                        (cpu.regs.d[d.reg1 as usize] & !(0xFF << shift)) | (b << shift);
                } else {
                    let b = (dn >> (8 * (bytes - 1 - i))) & 0xFF;
                    write_mem(bus, addr, 1, b)?;
                }
            }
        }
        Form::Trap => {
            return Err(crate::cpu::ExecError {
                message: format!("trap #{} unimplemented", d.word & 0xF),
            });
        }
        Form::Ea => {
            let e = d.ea.as_ref().unwrap();
            match mn {
                "clr" => {
                    write_ea(cpu, bus, d, e, size, 0)?;
                    cpu.regs.set_zn(size, 0);
                    cpu.regs.set_v(false);
                    cpu.regs.set_c(false);
                }
                "neg" | "negx" | "not" => {
                    let dst = read_ea(cpu, bus, d, e, size)?;
                    let r = match mn {
                        "neg" => alu_neg_ret(cpu, size, dst),
                        "negx" => alu_negx_ret(cpu, size, dst),
                        _ => {
                            let r = !dst & mask_of(size);
                            cpu.regs.set_znv_logic(size, r);
                            r
                        }
                    };
                    write_ea(cpu, bus, d, e, size, r)?;
                }
                "tst" => {
                    let v = read_ea(cpu, bus, d, e, size)?;
                    cpu.regs.set_zn(size, v);
                    cpu.regs.set_v(false);
                    cpu.regs.set_c(false);
                }
                "nbcd" => {
                    let dst = read_ea(cpu, bus, d, e, 1)?;
                    let r = 100 - (dst & 0xFF) - cpu.regs.x() as u32;
                    let d0 = (r % 10) | ((r / 10) % 10) << 4;
                    write_ea(cpu, bus, d, e, 1, d0)?;
                    cpu.regs.set_c(r > 99);
                    cpu.regs.set_x(r > 99);
                    if r != 0 {
                        cpu.regs.set_z(false);
                    }
                    cpu.regs.set_n(false);
                    cpu.regs.set_v(false);
                }
                "tas" => {
                    let dst = read_ea(cpu, bus, d, e, 1)?;
                    cpu.regs.set_zn(1, dst);
                    write_ea(cpu, bus, d, e, 1, dst | 0x80)?;
                }
                "pea" => {
                    let a = ea_addr(cpu, bus, d, e)?;
                    cpu.push_long(bus, a);
                }
                "jsr" => {
                    let a = ea_addr(cpu, bus, d, e)?;
                    cpu.push_long(bus, cpu.regs.pc);
                    cpu.regs.pc = a;
                }
                "jmp" => {
                    let a = ea_addr(cpu, bus, d, e)?;
                    cpu.regs.pc = a;
                }
                _ => {
                    return Err(crate::cpu::ExecError {
                        message: format!("unimplemented: {mn}"),
                    })
                }
            }
        }
        Form::Dreg => match mn {
            "swap" => {
                let v = cpu.regs.d[d.reg2 as usize];
                let r = v.rotate_left(16);
                cpu.regs.d[d.reg2 as usize] = r;
                cpu.regs.set_zn(4, r);
                cpu.regs.set_v(false);
                cpu.regs.set_c(false);
            }
            "ext" => {
                let v = cpu.regs.d[d.reg2 as usize];
                let r = if size == 2 {
                    (v & 0xFFFF_0000) | ((v & 0xFF) as u8 as i8 as i16 as u16 as u32)
                } else {
                    (v & 0xFFFF) as u16 as i16 as i32 as u32
                };
                cpu.regs.d[d.reg2 as usize] = r;
                cpu.regs.set_zn(size, r);
                cpu.regs.set_v(false);
                cpu.regs.set_c(false);
            }
            _ => {
                return Err(crate::cpu::ExecError {
                    message: format!("unimplemented: {mn}"),
                })
            }
        },
        Form::Areg => match mn {
            "unlk" => {
                let an = d.reg2 as usize;
                let sp = cpu.regs.read_a(an);
                cpu.regs.set_sp(sp);
                let v = cpu.pop_long(bus);
                cpu.set_a(an, v);
            }
            "move" => {
                // move An,USP / move USP,An (reg2 = An)
                let to_usp = d.insn.src == "M_AREG";
                let v = cpu.regs.read_a(d.reg2 as usize);
                if to_usp {
                    cpu.regs.usp = v;
                } else {
                    cpu.set_a(d.reg2 as usize, cpu.regs.usp);
                }
            }
            _ => {
                return Err(crate::cpu::ExecError {
                    message: format!("unimplemented: {mn}"),
                })
            }
        },
        Form::Imm16 => match mn {
            "stop" => {
                let _ = d.imm.unwrap_or(0);
                cpu.halted = true;
            }
            _ => {
                return Err(crate::cpu::ExecError {
                    message: format!("unimplemented: {mn}"),
                })
            }
        },
        Form::None => match mn {
            "nop" => {}
            "rts" => {
                cpu.regs.pc = cpu.pop_long(bus);
            }
            "rte" => {
                let sr = cpu.pop_long(bus);
                cpu.regs.sr = sr as u16;
                cpu.regs.pc = cpu.pop_long(bus);
            }
            "rtr" => {
                let ccr = cpu.pop_long(bus);
                cpu.regs.sr = (cpu.regs.sr & 0xFF00) | (ccr as u16 & 0xFF);
                cpu.regs.pc = cpu.pop_long(bus);
            }
            "reset" | "trapv" | "illegal" | "dc.w" => {
                return Err(crate::cpu::ExecError {
                    message: format!("{mn} unimplemented"),
                });
            }
            _ => {
                return Err(crate::cpu::ExecError {
                    message: format!("unimplemented: {mn}"),
                })
            }
        },
        _ => {
            return Err(crate::cpu::ExecError {
                message: format!("unimplemented form for {mn}"),
            })
        }
    }
    let _ = UNKNOWN;
    Ok(cycles)
}

// ---------------- ALU helpers ----------------

fn mask_of(size: u8) -> u32 {
    match size {
        1 => 0xFF,
        2 => 0xFFFF,
        _ => 0xFFFF_FFFF,
    }
}

fn alu_add(cpu: &mut Cpu, size: u8, s: u32, d: u32, reg: u8) {
    let m = mask_of(size);
    let r = (d & m).wrapping_add(s & m);
    cpu.regs.set_znv_c_add(size, d & m, s & m, r);
    cpu.regs.d[reg as usize] = (d & !m) | (r & m);
}

fn alu_add_ret(cpu: &mut Cpu, size: u8, s: u32, d: u32) -> u32 {
    let m = mask_of(size);
    let r = (d & m).wrapping_add(s & m);
    cpu.regs.set_znv_c_add(size, d & m, s & m, r);
    r & m
}

fn alu_sub(cpu: &mut Cpu, size: u8, s: u32, d: u32, reg: u8) {
    let m = mask_of(size);
    let r = (d & m).wrapping_sub(s & m);
    cpu.regs.set_znv_c_sub(size, d & m, s & m, r);
    cpu.regs.d[reg as usize] = (d & !m) | (r & m);
}

fn alu_sub_ret(cpu: &mut Cpu, size: u8, s: u32, d: u32) -> u32 {
    let m = mask_of(size);
    let r = (d & m).wrapping_sub(s & m);
    cpu.regs.set_znv_c_sub(size, d & m, s & m, r);
    r & m
}

fn alu_cmp(cpu: &mut Cpu, size: u8, s: u32, d: u32) {
    let m = mask_of(size);
    let r = (d & m).wrapping_sub(s & m);
    cpu.regs.set_znv_c_sub(size, d & m, s & m, r);
}

fn alu_logic(cpu: &mut Cpu, size: u8, s: u32, d: u32, reg: u8, op: u8) {
    let r = alu_logic_ret(cpu, size, s, d, op);
    let m = mask_of(size);
    cpu.regs.d[reg as usize] = (d & !m) | (r & m);
}

fn alu_logic_ret(cpu: &mut Cpu, size: u8, s: u32, d: u32, op: u8) -> u32 {
    let m = mask_of(size);
    let r = match op {
        0 => d & s,
        1 => d | s,
        _ => d ^ s,
    } & m;
    cpu.regs.set_znv_logic(size, r);
    r
}

fn alu_neg_ret(cpu: &mut Cpu, size: u8, d: u32) -> u32 {
    let m = mask_of(size);
    let r = (0u32).wrapping_sub(d & m) & m;
    cpu.regs.set_znv_c_sub(size, 0, d & m, r);
    r
}

fn alu_negx_ret(cpu: &mut Cpu, size: u8, d: u32) -> u32 {
    let m = mask_of(size);
    let x = cpu.regs.x() as u32;
    let r = (0u32).wrapping_sub((d & m).wrapping_add(x)) & m;
    cpu.regs.set_znv_c_sub(size, 0, (d & m).wrapping_add(x), r);
    cpu.regs.set_x(cpu.regs.c());
    r
}

fn alu_addx_ret(cpu: &mut Cpu, size: u8, s: u32, d: u32, x: u32) -> u32 {
    let m = mask_of(size);
    let r = (d & m).wrapping_add(s & m).wrapping_add(x) & m;
    cpu.regs
        .set_znv_c_add(size, (d & m).wrapping_add(x), s & m, r);
    cpu.regs.set_x(cpu.regs.c());
    if r != 0 {
        cpu.regs.set_z(false);
    }
    r
}

fn alu_subx_ret(cpu: &mut Cpu, size: u8, s: u32, d: u32, x: u32) -> u32 {
    let m = mask_of(size);
    let r = (d & m).wrapping_sub(s & m).wrapping_sub(x) & m;
    cpu.regs
        .set_znv_c_sub(size, d & m, (s & m).wrapping_add(x), r);
    cpu.regs.set_x(cpu.regs.c());
    if r != 0 {
        cpu.regs.set_z(false);
    }
    r
}

/// Shift/rotate semantics. `count` 0 (register form) leaves C/X alone.
fn shift_op(
    cpu: &mut Cpu,
    mn: &str,
    size: u8,
    d: u32,
    count: u32,
) -> Result<u32, crate::cpu::ExecError> {
    let m = mask_of(size);
    let bits = size as u32 * 8;
    let mut v = d & m;
    let mut c = cpu.regs.c();
    let (is_left, is_rot, use_x) = match mn {
        "asl" | "lsl" | "rol" => (true, mn == "rol", false),
        "asr" | "lsr" | "ror" => (false, mn == "ror", false),
        "roxl" => (true, true, true),
        _ => (false, true, true), // roxr
    };
    if count == 0 {
        cpu.regs.set_zn(size, v);
        cpu.regs.set_v(false);
        return Ok(v);
    }
    let n = count.min(bits);
    for _ in 0..n {
        if is_rot {
            let out = if is_left { v >> (bits - 1) } else { v & 1 };
            if use_x {
                let x = cpu.regs.x() as u32;
                v = if is_left {
                    (v << 1) | x
                } else {
                    (v >> 1) | (x << (bits - 1))
                };
                c = out != 0;
            } else {
                v = if is_left {
                    (v << 1) | out
                } else {
                    (v >> 1) | (out << (bits - 1))
                };
                c = out != 0;
            }
        } else if is_left {
            let out = v >> (bits - 1);
            v = (v << 1) & m;
            c = out != 0;
        } else {
            let out = v & 1;
            if mn == "asr" {
                v = (v >> 1) | (v & (1 << (bits - 1))); // arithmetic
            } else {
                v >>= 1;
            }
            c = out != 0;
        }
    }
    if use_x || mn.starts_with("as") {
        cpu.regs.set_x(c);
    }
    cpu.regs.set_c(c);
    cpu.regs.set_zn(size, v);
    cpu.regs.set_v(false);
    Ok(v)
}

fn bit_op<B: M68kBus + ?Sized>(
    cpu: &mut Cpu,
    bus: &mut B,
    d: &DecodedInsn,
    e: &Ea,
    size: u8,
    bit: u32,
    mn: &str,
) -> Result<(), crate::cpu::ExecError> {
    let bit_in_reg = bit < 32 && e.mode == 0;
    let v = if bit_in_reg {
        cpu.regs.d[e.reg as usize]
    } else {
        read_ea(cpu, bus, d, e, size)?
    };
    let test = (v >> bit) & 1 != 0;
    cpu.regs.set_z(!test);
    let write_back = mn != "btst";
    if write_back {
        let new = match mn {
            "bset" => v | (1 << bit),
            "bclr" => v & !(1 << bit),
            _ => v ^ (1 << bit),
        };
        if bit_in_reg {
            cpu.regs.d[e.reg as usize] = new;
        } else {
            write_ea(cpu, bus, d, e, size, new)?;
        }
    }
    Ok(())
}

fn movem_op<B: M68kBus + ?Sized>(
    cpu: &mut Cpu,
    bus: &mut B,
    d: &DecodedInsn,
    predec: bool,
) -> Result<(), crate::cpu::ExecError> {
    let mask = d.regmask.unwrap_or(0);
    let size = size_bytes(d.insn.size);
    let count = mask.count_ones() as u32;
    let width = size as u32;
    let e = d.ea.as_ref().unwrap();

    if predec {
        // -(An): decrement An by 2*n (A7 for the stack), store reversed
        // (A7..D0), A7's own stored value = the new SP
        let an = cpu.regs.read_a(e.reg as usize);
        let new_an = an.wrapping_sub(2 * count);
        cpu.set_a(e.reg as usize, new_an);
        let mut addr = new_an;
        // pd mask is bit-reversed: bit 15 = D0 .. bit 0 = A7; stored in
        // reverse list order, so D0 lands at the lowest address (matching
        // the er pop). A7's own stored value is the new SP.
        for bit in (0..16).rev() {
            if mask & (1 << bit) != 0 {
                let reg = 15 - bit;
                let v = if reg == 15 {
                    new_an
                } else {
                    cpu.r(reg as usize)
                };
                write_mem(bus, addr, size, v)?;
                addr += width;
            }
        }
    } else {
        // (An)+ postincrements by 2 per register transferred, regardless
        // of operand size (PRM); plain (An) / d16 / abs do not modify An
        let post_an: Option<usize> = match e.mode {
            3 => Some(e.reg as usize),
            1 => None,
            _ => None,
        };
        let mut addr = match e.mode {
            1 | 3 => cpu.regs.read_a(e.reg as usize),
            _ => ea_addr(cpu, bus, d, e)?,
        };
        if let Some(an) = post_an {
            cpu.set_a(an, cpu.regs.read_a(an).wrapping_add(2 * count));
        }
        let to_regs = d.insn.form == Form::MovemEr;
        for reg in 0..16 {
            if mask & (1 << reg) != 0 {
                if to_regs {
                    let v = read_mem(bus, addr, size);
                    cpu.w(reg as usize, v);
                } else {
                    let v = cpu.r(reg as usize);
                    write_mem(bus, addr, size, v)?;
                }
                addr += width;
            }
        }
    }
    Ok(())
}

fn step(reg: u8, size: u8) -> u32 {
    if size == 1 && reg == 7 {
        2
    } else {
        size as u32
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
) -> Result<(), crate::cpu::ExecError> {
    match size {
        1 => bus.write_byte(addr, v as u8),
        2 => bus.write_word(addr, v as u16),
        _ => bus.write_long(addr, v),
    }
    Ok(())
}

/// Best-effort EA cycle extra (PRM operand-fetch costs, base values).
fn ea_extra(d: &DecodedInsn) -> u32 {
    let e = match (&d.ea, &d.ea_dst) {
        (Some(e), _) => e,
        (None, Some(e)) => e,
        _ => return 0,
    };
    match e.mode {
        0 | 1 => 0,
        2 => 4,
        3 => 4,
        4 => 6,
        5 => 8,
        6 => 10,
        7 => match e.reg {
            0 => 8,
            1 => 12,
            2 | 3 => 10,
            _ => 0,
        },
        _ => 0,
    }
}
