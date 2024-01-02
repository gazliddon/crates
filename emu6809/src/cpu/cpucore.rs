#![deny(unused_imports)]
use bitflags::bitflags;
use alu::GazAlu;

bitflags! {
    pub struct StackFlags: u8 {

    const PC = 0x80;
    const STACK = 0x40;
    const Y = 0x20;
    const X = 0x10;
    const DP = 0x08;
    const B = 0x04;
    const A = 0x02;
    const CC = 0x01;
    const INT_FLAGS =
          Self::X.bits()
        | Self::Y.bits()
        | Self::A.bits()
        | Self::B.bits()
        | Self::DP.bits()
        | Self::STACK.bits() ;
    }
}

// 6809 vectors
pub const VEC_SWI3: usize = 0xfff2;
pub const VEC_SWI2: usize = 0xfff4;
pub const VEC_FIRQ: usize = 0xfff6;
pub const VEC_IRQ: usize = 0xfff8;
pub const VEC_SWI: usize = 0xfffa;
pub const VEC_NMI: usize = 0xfffc;
pub const VEC_RESET: usize = 0xfffe;

// Handles CPU emulation
use super::{
    alu,
    AddressLines, Direct, Extended, Flags, Immediate16, Immediate8, Indexed, Inherent,
    InstructionDecoder, RegEnum, RegisterPair, RegisterSet, Regs, Relative, Relative16,
};

use emucore::mem::{MemErrorTypes, MemoryIO};



// use std::cell::RefCell;
// use std::rc::Rc;

use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum CpuErr {
    #[error("Unknown instruction")]
    UnknownInstruction,
    #[error("Unimplemented instruction {0}")]
    Unimplemented(usize),
    #[error("Illegal addressing mode")]
    IllegalAddressingMode,
    #[error(transparent)]
    Memory(#[from] MemErrorTypes),
}

// use cpu::alu;
pub type CpuResult<T = ()> = std::result::Result<T, CpuErr>;

fn get_tfr_reg(op: u8) -> RegEnum {
    match op {
        0 => RegEnum::D,
        1 => RegEnum::X,
        2 => RegEnum::Y,
        3 => RegEnum::U,
        4 => RegEnum::S,
        5 => RegEnum::PC,
        8 => RegEnum::A,
        9 => RegEnum::B,
        10 => RegEnum::CC,
        11 => RegEnum::DP,
        _ => {
            println!("op of {op:02X}");
            panic!("illegal tfr regs")
        }
    }
}

pub fn get_tfr_regs(op: u8) -> (RegEnum, RegEnum) {
    (get_tfr_reg(op >> 4), get_tfr_reg(op & 0xf))
}

// Bool defaults to false
#[derive(Default)]
pub struct Pins {
    pub firq: bool,
    pub irq: bool,
    pub nmi: bool,
    pub reset: bool,
    pub waiting_for_irq: bool,
}

pub struct Context<'a> {
    pub regs: &'a mut Regs,
    pub pins: &'a mut Pins,
    pub ins: InstructionDecoder,
    pub cycles: usize,
    pub instructions: usize,
    // TODO This should generic with compile time dispatch
    pub mem: &'a mut dyn MemoryIO,
}

// use serde::Deserializer;
impl<'a> Context<'a> {
    fn set_next_pc(&mut self, v: usize) {
        self.ins.next_addr = v & 0xffff;
    }

    fn set_next_pc_rel(&mut self, v: i16) {
        let pc = self.ins.next_addr.wrapping_add(v as usize);
        self.set_next_pc(pc)
    }

    fn fetch_byte_as_i16<A: AddressLines>(&mut self) -> CpuResult<i16> {
        let ret = self.fetch_byte::<A>()?;
        Ok(ret as i8 as i16)
    }

    fn store_byte<A: AddressLines>(&mut self, v: u8) -> CpuResult<u16> {
        A::store_byte(self.mem, self.regs, &mut self.ins, v)
    }

    fn store_word<A: AddressLines>(&mut self, v: u16) -> CpuResult<u16> {
        A::store_word(self.mem, self.regs, &mut self.ins, v)
    }

    fn fetch_word_as_i16<A: AddressLines>(&mut self) -> CpuResult<i16> {
        let ret = self.fetch_word::<A>()?;
        Ok(ret as i16)
    }

    fn fetch_byte<A: AddressLines>(&mut self) -> CpuResult<u8> {
        A::fetch_byte(self.mem, self.regs, &mut self.ins)
    }

    fn fetch_word<A: AddressLines>(&mut self) -> CpuResult<u16> {
        A::fetch_word(self.mem, self.regs, &mut self.ins)
    }

    fn ea<A: AddressLines>(&mut self) -> CpuResult<u16> {
        A::ea(self.mem, self.regs, &mut self.ins)
    }

    fn op16_2<A: AddressLines>(
        &mut self,
        write_mask: u8,
        func: fn(&mut Flags, u8, u32, u32) -> u16,
        i0: u16,
    ) -> CpuResult<u16> {
        let i1 = self.fetch_word::<A>()?;
        Ok(func(
            &mut self.regs.flags,
            write_mask,
            u32::from(i0),
            u32::from(i1),
        ))
    }

    fn opd_2<A: AddressLines>(
        &mut self,
        write_mask: u8,
        func: fn(&mut Flags, u8, u32, u32) -> u16,
    ) -> CpuResult<u16> {
        let i0 = self.regs.get_d();
        self.op16_2::<A>(write_mask, func, i0)
    }

    fn modd_2<A: AddressLines>(
        &mut self,
        write_mask: u8,
        func: fn(&mut Flags, u8, u32, u32) -> u16,
    ) -> CpuResult<()> {
        let r = self.opd_2::<A>(write_mask, func)?;
        self.regs.set_d(r);
        Ok(())
    }

    fn opa_2<A: AddressLines>(
        &mut self,
        write_mask: u8,
        func: fn(&mut Flags, u8, u32, u32) -> u8,
    ) -> CpuResult<u8> {
        let i0 = self.regs.a;
        let i1 = self.fetch_byte::<A>()?;
        Ok(func(
            &mut self.regs.flags,
            write_mask,
            u32::from(i0),
            u32::from(i1),
        ))
    }

    fn opb_2<A: AddressLines>(
        &mut self,
        write_mask: u8,
        func: fn(&mut Flags, u8, u32, u32) -> u8,
    ) -> CpuResult<u8> {
        let i0 = self.regs.b;
        let i1 = self.fetch_byte::<A>()?;
        Ok(func(
            &mut self.regs.flags,
            write_mask,
            u32::from(i0),
            u32::from(i1),
        ))
    }

    fn moda_2<A: AddressLines>(
        &mut self,
        write_mask: u8,
        func: fn(&mut Flags, u8, u32, u32) -> u8,
    ) -> CpuResult<()> {
        let r = self.opa_2::<A>(write_mask, func)?;
        self.regs.a = r;
        Ok(())
    }

    fn modb_2<A: AddressLines>(
        &mut self,
        write_mask: u8,
        func: fn(&mut Flags, u8, u32, u32) -> u8,
    ) -> CpuResult<()> {
        let r = self.opb_2::<A>(write_mask, func)?;
        self.regs.b = r;
        Ok(())
    }

    fn opa(&mut self, write_mask: u8, func: fn(&mut Flags, u8, u32) -> u8) -> u8 {
        let i0 = self.regs.a;
        func(&mut self.regs.flags, write_mask, u32::from(i0))
    }

    fn opb(&mut self, write_mask: u8, func: fn(&mut Flags, u8, u32) -> u8) -> u8 {
        let i0 = self.regs.b;
        func(&mut self.regs.flags, write_mask, u32::from(i0))
    }

    fn moda(&mut self, write_mask: u8, func: fn(&mut Flags, u8, u32) -> u8) -> u8 {
        let r = self.opa(write_mask, func);
        self.regs.a = r;
        r
    }

    fn modb(&mut self, write_mask: u8, func: fn(&mut Flags, u8, u32) -> u8) -> u8 {
        let r = self.opb(write_mask, func);
        self.regs.b = r;
        r
    }

    fn rwmod8<A: AddressLines>(
        &mut self,
        write_mask: u8,
        func: fn(&mut Flags, u8, u32) -> u8,
    ) -> CpuResult<u8> {
        let ea = self.ea::<A>()?;
        let b = self.mem.load_byte(ea.into())?;

        let v = u32::from(b);
        let r = func(&mut self.regs.flags, write_mask, v);

        self.mem.store_byte(ea.into(), r)?;

        Ok(r)
    }

    fn branch<A: AddressLines>(&mut self, v: bool) -> CpuResult<()> {
        let offset = self.fetch_byte_as_i16::<A>()?;

        if v {
            self.set_next_pc_rel(offset)
        }
        Ok(())
    }

    fn lbranch<A: AddressLines>(&mut self, v: bool) -> CpuResult<()> {
        let offset = self.fetch_word_as_i16::<A>()?;

        if v {
            self.set_next_pc_rel(offset)
        }
        Ok(())
    }

    fn post_clear(&mut self) {
        self.regs.flags.set(Flags::Z, true);
        self.regs.flags.set(Flags::N | Flags::V | Flags::C, false);
    }

    fn st8<A: AddressLines>(&mut self, v: u8) -> CpuResult<()> {
        self.store_byte::<A>(v)?;
        alu::nz::<u8>(&mut self.regs.flags, Flags::NZ.bits(), u32::from(v));
        self.regs.flags.set(Flags::V, false);
        Ok(())
    }

    fn st16<A: AddressLines>(&mut self, v: u16) -> CpuResult<()> {
        self.store_word::<A>(v)?;
        alu::nz::<u16>(&mut self.regs.flags, Flags::NZ.bits(), u32::from(v));
        self.regs.flags.set(Flags::V, false);
        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////
// Stakc functions

impl<'a> Context<'a> {
    fn get_stack(&self, is_system: bool) -> u16 {
        if is_system {
            self.regs.s
        } else {
            self.regs.u
        }
    }

    fn set_stack(&mut self, v: u16, is_system: bool) {
        if is_system {
            self.regs.s = v
        } else {
            self.regs.u = v
        }
    }

    fn push_byte(&mut self, v: u8, is_system: bool) -> CpuResult<()> {
        let sp = self.get_stack(is_system);
        let sp = sp.wrapping_sub(1);
        self.mem.store_byte(sp.into(), v)?;
        self.set_stack(sp, is_system);
        Ok(())
    }

    fn push_word(&mut self, v: u16, is_system: bool) -> CpuResult<()> {
        let sp = self.get_stack(is_system);
        let sp = sp.wrapping_sub(2);
        self.mem.store_word(sp.into(), v)?;
        self.set_stack(sp, is_system);
        Ok(())
    }

    fn pop_byte(&mut self, is_system: bool) -> CpuResult<u8> {
        let sp = self.get_stack(is_system);
        let r = self.mem.load_byte(sp as usize)?;
        self.set_stack(sp.wrapping_add(1), is_system);
        Ok(r)
    }

    fn pop_word(&mut self, is_system: bool) -> CpuResult<u16> {
        let sp = self.get_stack(is_system);

        let r = self.mem.load_word(sp as usize)?;

        self.set_stack(sp.wrapping_add(2), is_system);

        Ok(r)
    }
}

////////////////////////////////////////////////////////////////////////////////

impl<'a> Context<'a> {
    fn orcc<A: AddressLines>(&mut self) -> CpuResult<()> {
        let v = self.fetch_byte::<A>()?;
        let cc = self.regs.flags.bits();
        self.regs.flags.set_flags(v | cc);
        Ok(())
    }

    fn stx<A: AddressLines>(&mut self) -> CpuResult<()> {
        let x = self.regs.x;
        self.st16::<A>(x)
    }

    fn sta<A: AddressLines>(&mut self) -> CpuResult<()> {
        let r = self.regs.a;
        self.st8::<A>(r)
    }

    fn stb<A: AddressLines>(&mut self) -> CpuResult<()> {
        let r = self.regs.b;
        self.st8::<A>(r)
    }

    fn std<A: AddressLines>(&mut self) -> CpuResult<()> {
        let r = self.regs.get_d();
        self.st16::<A>(r)
    }

    fn stu<A: AddressLines>(&mut self) -> CpuResult<()> {
        let r = self.regs.u;
        self.st16::<A>(r)
    }

    fn sty<A: AddressLines>(&mut self) -> CpuResult<()> {
        let r = self.regs.y;
        self.st16::<A>(r)
    }

    fn sts<A: AddressLines>(&mut self) -> CpuResult<()> {
        let r = self.regs.s;
        self.st16::<A>(r)
    }

    #[allow(clippy::extra_unused_type_parameters)]
    fn lsla_asla<A: AddressLines>(&mut self) -> CpuResult<()> {
        self.moda(Flags::NZVC.bits(), u8::asl);
        Ok(())
    }

    #[allow(clippy::extra_unused_type_parameters)]
    fn lslb_aslb<A: AddressLines>(&mut self) -> CpuResult<()> {
        self.modb(Flags::NZVC.bits(), u8::asl);
        Ok(())
    }

    #[allow(clippy::extra_unused_type_parameters)]
    fn asra<A: AddressLines>(&mut self) -> CpuResult<()> {
        self.moda(Flags::NZVC.bits(), u8::asr);
        Ok(())
    }

    #[allow(clippy::extra_unused_type_parameters)]
    fn asrb<A: AddressLines>(&mut self) -> CpuResult<()> {
        self.modb(Flags::NZVC.bits(), u8::asr);
        Ok(())
    }

    fn asr<A: AddressLines>(&mut self) -> CpuResult<()> {
        self.rwmod8::<A>(Flags::NZVC.bits(), u8::asr)?;
        Ok(())
    }

    fn tfr<A: AddressLines>(&mut self) -> CpuResult<()> {
        let operand = self.fetch_byte::<A>()?;
        let (a, b) = get_tfr_regs(operand );
        let av = self.regs.get(&a);
        self.regs.set(&b, av);
        Ok(())
    }

    #[allow(clippy::extra_unused_type_parameters)]
    fn abx<A: AddressLines>(&mut self) -> CpuResult<()> {
        let x = self.regs.x;
        self.regs.x = x.wrapping_add(u16::from(self.regs.b));
        Ok(())
    }

    fn beq<A: AddressLines>(&mut self) -> CpuResult<()> {
        let z = self.regs.flags.contains(Flags::Z);
        self.branch::<A>(z)
    }

    fn bge<A: AddressLines>(&mut self) -> CpuResult<()> {
        let cond = self.regs.flags.ge();
        self.branch::<A>(cond)
    }

    fn bgt<A: AddressLines>(&mut self) -> CpuResult<()> {
        let cond = self.regs.flags.gt();
        self.branch::<A>(cond)
    }

    fn blo_bcs<A: AddressLines>(&mut self) -> CpuResult<()> {
        let cond = self.regs.flags.contains(Flags::C);
        self.branch::<A>(cond)
    }

    fn brn<A: AddressLines>(&mut self) -> CpuResult<()> {
        let cond = self.regs.flags.contains(Flags::N);
        self.branch::<A>(cond)
    }

    fn bhs_bcc<A: AddressLines>(&mut self) -> CpuResult<()> {
        let cond = self.regs.flags.contains(Flags::C);
        self.branch::<A>(!cond)
    }

    fn bhi<A: AddressLines>(&mut self) -> CpuResult<()> {
        let cond = self.regs.flags.hi();
        self.branch::<A>(cond)
    }

    fn ble<A: AddressLines>(&mut self) -> CpuResult<()> {
        let cond = self.regs.flags.le();
        self.branch::<A>(cond)
    }

    fn bls<A: AddressLines>(&mut self) -> CpuResult<()> {
        let cond = self.regs.flags.ls();
        self.branch::<A>(cond)
    }

    fn blt<A: AddressLines>(&mut self) -> CpuResult<()> {
        let cond = self.regs.flags.lt();
        self.branch::<A>(cond)
    }

    #[allow(clippy::extra_unused_type_parameters)]
    fn rts<A: AddressLines>(&mut self) -> CpuResult<()> {
        let pc = self.pop_word(true)?;
        self.set_next_pc(pc as usize);
        Ok(())
    }

    fn bsr<A: AddressLines>(&mut self) -> CpuResult<()> {
        let offset = self.fetch_byte_as_i16::<A>()?;
        let next_op = self.get_pc();
        self.push_word((next_op & 0xfff) as u16, true)?;
        self.set_next_pc_rel(offset);
        Ok(())
    }

    fn bvc<A: AddressLines>(&mut self) -> CpuResult<()> {
        let cond = !self.regs.flags.contains(Flags::V);
        self.branch::<A>(cond)
    }

    fn bne<A: AddressLines>(&mut self) -> CpuResult<()> {
        let cond = !self.regs.flags.contains(Flags::Z);
        self.branch::<A>(cond)
    }

    fn bvs<A: AddressLines>(&mut self) -> CpuResult<()> {
        let cond = self.regs.flags.contains(Flags::V);
        self.branch::<A>(cond)
    }

    fn bmi<A: AddressLines>(&mut self) -> CpuResult<()> {
        let cond = self.regs.flags.contains(Flags::N);
        self.branch::<A>(cond)
    }

    fn bra<A: AddressLines>(&mut self) -> CpuResult<()> {
        self.branch::<A>(true)
    }

    fn bpl<A: AddressLines>(&mut self) -> CpuResult<()> {
        let cond = !self.regs.flags.contains(Flags::N);
        self.branch::<A>(cond)
    }

    fn andcc<A: AddressLines>(&mut self) -> CpuResult<()> {
        let i0 = self.regs.flags.bits();
        let i1 = self.fetch_byte::<A>()?;
        let new_f = u8::and(&mut self.regs.flags, 0, u32::from(i0), u32::from(i1));
        self.regs.flags.set_flags(new_f);
        Ok(())
    }

    fn lsl_asl<A: AddressLines>(&mut self) -> CpuResult<()> {
        self.rwmod8::<A>(Flags::NZVC.bits(), u8::asl)?;
        Ok(())
    }

    ////////////////////////////////////////////////////////////////////////////////

    fn adda<A: AddressLines>(&mut self) -> CpuResult<()> {
        self.moda_2::<A>(Flags::NZVCH.bits(), u8::add)
    }

    fn adca<A: AddressLines>(&mut self) -> CpuResult<()> {
        self.moda_2::<A>(Flags::NZVCH.bits(), u8::adc)
    }

    fn adcb<A: AddressLines>(&mut self) -> CpuResult<()> {
        self.modb_2::<A>(Flags::NZVCH.bits(), u8::adc)
    }

    fn addb<A: AddressLines>(&mut self) -> CpuResult<()> {
        self.modb_2::<A>(Flags::NZVCH.bits(), u8::add)
    }

    fn addd<A: AddressLines>(&mut self) -> CpuResult<()> {
        self.modd_2::<A>(Flags::NZVC.bits(), u16::add)
    }

    ////////////////////////////////////////////////////////////////////////////////
    fn anda<A: AddressLines>(&mut self) -> CpuResult<()> {
        self.moda_2::<A>(Flags::NZV.bits(), u8::and)?;
        Ok(())
    }

    fn andb<A: AddressLines>(&mut self) -> CpuResult<()> {
        self.modb_2::<A>(Flags::NZV.bits(), u8::and)
    }

    fn bita<A: AddressLines>(&mut self) -> CpuResult<()> {
        self.opa_2::<A>(Flags::NZ.bits(), u8::and)?;
        Ok(())
    }

    fn bitb<A: AddressLines>(&mut self) -> CpuResult<()> {
        self.opb_2::<A>(Flags::NZ.bits(), u8::and)?;
        Ok(())
    }

    ////////////////////////////////////////////////////////////////////////////////

    #[allow(clippy::extra_unused_type_parameters)]
    fn clra<A: AddressLines>(&mut self) -> CpuResult<()> {
        self.regs.a = 0;
        self.post_clear();
        Ok(())
    }

    #[allow(clippy::extra_unused_type_parameters)]
    fn clrb<A: AddressLines>(&mut self) -> CpuResult<()> {
        self.regs.b = 0;
        self.post_clear();
        Ok(())
    }

    fn clr<A: AddressLines>(&mut self) -> CpuResult<()> {
        self.store_byte::<A>(0)?;
        self.post_clear();
        Ok(())
    }

    ////////////////////////////////////////////////////////////////////////////////
    fn cmpa<A: AddressLines>(&mut self) -> CpuResult<()> {
        self.opa_2::<A>(Flags::NZVC.bits(), u8::sub)?;
        Ok(())
    }

    fn cmpb<A: AddressLines>(&mut self) -> CpuResult<()> {
        self.opb_2::<A>(Flags::NZVC.bits(), u8::sub)?;
        Ok(())
    }

    fn cmpd<A: AddressLines>(&mut self) -> CpuResult<()> {
        let i0 = self.regs.get_d();
        self.op16_2::<A>(Flags::NZVC.bits(), u16::sub, i0)?;
        Ok(())
    }

    fn cmpu<A: AddressLines>(&mut self) -> CpuResult<()> {
        let i0 = self.regs.u;
        self.op16_2::<A>(Flags::NZVC.bits(), u16::sub, i0)?;
        Ok(())
    }

    fn cmps<A: AddressLines>(&mut self) -> CpuResult<()> {
        let i0 = self.regs.s;
        self.op16_2::<A>(Flags::NZVC.bits(), u16::sub, i0)?;
        Ok(())
    }

    fn cmpx<A: AddressLines>(&mut self) -> CpuResult<()> {
        let i0 = self.regs.x;
        self.op16_2::<A>(Flags::NZVC.bits(), u16::sub, i0)?;
        Ok(())
    }

    fn cmpy<A: AddressLines>(&mut self) -> CpuResult<()> {
        let i0 = self.regs.y;
        self.op16_2::<A>(Flags::NZVC.bits(), u16::sub, i0)?;
        Ok(())
    }

    ////////////////////////////////////////////////////////////////////////////////
    #[allow(clippy::extra_unused_type_parameters)]
    fn coma<A: AddressLines>(&mut self) -> CpuResult<()> {
        self.moda(Flags::NZVC.bits(), u8::com);
        Ok(())
    }

    #[allow(clippy::extra_unused_type_parameters)]
    fn comb<A: AddressLines>(&mut self) -> CpuResult<()> {
        self.modb(Flags::NZVC.bits(), u8::com);
        Ok(())
    }

    fn com<A: AddressLines>(&mut self) -> CpuResult<()> {
        self.rwmod8::<A>(Flags::NZVC.bits(), u8::com)?;
        Ok(())
    }

    ////////////////////////////////////////////////////////////////////////////////
    #[allow(clippy::extra_unused_type_parameters)]
    fn deca<A: AddressLines>(&mut self) -> CpuResult<()> {
        self.moda(Flags::NZV.bits(), u8::dec);
        Ok(())
    }

    #[allow(clippy::extra_unused_type_parameters)]
    fn decb<A: AddressLines>(&mut self) -> CpuResult<()> {
        self.modb(Flags::NZV.bits(), u8::dec);
        Ok(())
    }

    fn dec<A: AddressLines>(&mut self) -> CpuResult<()> {
        self.rwmod8::<A>(Flags::NZV.bits(), u8::dec)?;
        Ok(())
    }

    ////////////////////////////////////////////////////////////////////////////////
    #[allow(clippy::extra_unused_type_parameters)]
    fn inca<A: AddressLines>(&mut self) -> CpuResult<()> {
        self.moda(Flags::NZV.bits(), u8::inc);
        Ok(())
    }
    #[allow(clippy::extra_unused_type_parameters)]
    fn incb<A: AddressLines>(&mut self) -> CpuResult<()> {
        self.modb(Flags::NZV.bits(), u8::inc);
        Ok(())
    }
    fn inc<A: AddressLines>(&mut self) -> CpuResult<()> {
        self.rwmod8::<A>(Flags::NZV.bits(), u8::inc)?;
        Ok(())
    }

    ////////////////////////////////////////////////////////////////////////////////

    #[allow(clippy::extra_unused_type_parameters)]
    fn lsra<A: AddressLines>(&mut self) -> CpuResult<()> {
        self.moda(Flags::NZC.bits(), u8::lsr);
        Ok(())
    }

    #[allow(clippy::extra_unused_type_parameters)]
    fn lsrb<A: AddressLines>(&mut self) -> CpuResult<()> {
        self.modb(Flags::NZC.bits(), u8::lsr);
        Ok(())
    }

    fn lsr<A: AddressLines>(&mut self) -> CpuResult<()> {
        self.rwmod8::<A>(Flags::NZC.bits(), u8::lsr)?;
        Ok(())
    }

    ////////////////////////////////////////////////////////////////////////////////
    fn eora<A: AddressLines>(&mut self) -> CpuResult<()> {
        self.moda_2::<A>(Flags::NZV.bits(), u8::eor)?;
        Ok(())
    }
    fn eorb<A: AddressLines>(&mut self) -> CpuResult<()> {
        self.modb_2::<A>(Flags::NZV.bits(), u8::eor)?;
        Ok(())
    }

    ////////////////////////////////////////////////////////////////////////////////
    fn ora<A: AddressLines>(&mut self) -> CpuResult<()> {
        self.moda_2::<A>(Flags::NZV.bits(), u8::or)?;
        Ok(())
    }

    fn orb<A: AddressLines>(&mut self) -> CpuResult<()> {
        self.modb_2::<A>(Flags::NZV.bits(), u8::or)?;
        Ok(())
    }

    ////////////////////////////////////////////////////////////////////////////////
    #[allow(clippy::extra_unused_type_parameters)]
    fn daa<A: AddressLines>(&mut self) -> CpuResult<()> {
        // fuck sakes
        let a = u32::from(self.regs.a);

        let msn = a & 0xf0;
        let lsn = a & 0xf0;

        let mut cf = 0u32;

        if lsn > 0x09 || self.regs.flags.contains(Flags::H) {
            cf |= 0x06;
        }

        if msn > 0x80 && lsn > 0x09 {
            cf |= 0x60;
        }

        if msn > 0x90 || self.regs.flags.contains(Flags::C) {
            cf |= 0x60;
        }

        let temp = cf.wrapping_add(a);

        self.regs.flags.set(Flags::C, temp & 0x100 != 0);
        self.regs.flags.set(Flags::V | Flags::N, false);

        let new_a = alu::nz::<u8>(&mut self.regs.flags, Flags::NZ.bits(), temp);

        self.regs.a = new_a;

        Ok(())
    }

    ////////////////////////////////////////////////////////////////////////////////
    fn exg<A: AddressLines>(&mut self) -> CpuResult<()> {
        let operand = self.fetch_byte::<A>()?;
        let (a, b) = get_tfr_regs(operand );
        let av = self.regs.get(&a);
        let bv = self.regs.get(&b);
        self.regs.set(&b, av);
        self.regs.set(&a, bv);
        Ok(())
    }

    fn jsr<A: AddressLines>(&mut self) -> CpuResult<()> {
        let dest = self.ea::<A>()?;
        let next_op = (self.get_pc() & 0xffff) as u16;
        self.push_word(next_op, true)?;
        self.set_next_pc(dest as usize);
        Ok(())
    }

    // {{{ Long Branches

    fn lbsr<A: AddressLines>(&mut self) -> CpuResult<()> {
        let offset = self.fetch_word_as_i16::<A>()?;
        let next_op = (self.get_pc() & 0xffff) as u16;
        self.push_word(next_op, true)?;
        self.set_next_pc_rel(offset);
        Ok(())
    }

    fn lbrn<A: AddressLines>(&mut self) -> CpuResult<()> {
        let cond = self.regs.flags.contains(Flags::N);
        self.lbranch::<A>(cond)
    }
    fn lbhi<A: AddressLines>(&mut self) -> CpuResult<()> {
        let cond = self.regs.flags.hi();
        self.lbranch::<A>(cond)
    }

    fn lbra<A: AddressLines>(&mut self) -> CpuResult<()> {
        self.lbranch::<A>(true)
    }

    fn lbls<A: AddressLines>(&mut self) -> CpuResult<()> {
        let cond = self.regs.flags.ls();
        self.lbranch::<A>(cond)
    }

    fn lble<A: AddressLines>(&mut self) -> CpuResult<()> {
        let cond = self.regs.flags.le();
        self.lbranch::<A>(cond)
    }

    fn lbge<A: AddressLines>(&mut self) -> CpuResult<()> {
        let cond = self.regs.flags.ge();
        self.lbranch::<A>(cond)
    }
    fn lblt<A: AddressLines>(&mut self) -> CpuResult<()> {
        let cond = self.regs.flags.lt();
        self.lbranch::<A>(cond)
    }
    fn lbgt<A: AddressLines>(&mut self) -> CpuResult<()> {
        let cond = self.regs.flags.gt();
        self.lbranch::<A>(cond)
    }

    fn lbvc<A: AddressLines>(&mut self) -> CpuResult<()> {
        let cond = !self.regs.flags.contains(Flags::V);
        self.lbranch::<A>(cond)
    }

    fn lbvs<A: AddressLines>(&mut self) -> CpuResult<()> {
        let cond = self.regs.flags.contains(Flags::V);
        self.lbranch::<A>(cond)
    }
    fn lbpl<A: AddressLines>(&mut self) -> CpuResult<()> {
        let cond = !self.regs.flags.contains(Flags::N);
        self.lbranch::<A>(cond)
    }
    fn lbmi<A: AddressLines>(&mut self) -> CpuResult<()> {
        let cond = self.regs.flags.contains(Flags::N);
        self.lbranch::<A>(cond)
    }

    fn lbhs_lbcc<A: AddressLines>(&mut self) -> CpuResult<()> {
        let cond = self.regs.flags.contains(Flags::C);
        self.lbranch::<A>(!cond)
    }

    fn lblo_lbcs<A: AddressLines>(&mut self) -> CpuResult<()> {
        let cond = self.regs.flags.contains(Flags::C);
        self.lbranch::<A>(cond)
    }

    fn lbne<A: AddressLines>(&mut self) -> CpuResult<()> {
        let cond = !self.regs.flags.contains(Flags::Z);
        self.lbranch::<A>(cond)
    }
    fn lbeq<A: AddressLines>(&mut self) -> CpuResult<()> {
        let z = self.regs.flags.contains(Flags::Z);
        self.lbranch::<A>(z)
    }
    // }}}

    ////////////////////////////////////////////////////////////////////////////////
    // {{{ Register loads
    fn load_reg_byte<A: AddressLines>(&mut self) -> CpuResult<u8> {
        let v = self.fetch_byte::<A>()?;
        alu::nz::<u8>(&mut self.regs.flags, Flags::NZ.bits(), u32::from(v));
        self.regs.flags.set(Flags::V, false);
        Ok(v)
    }
    fn load_reg_word<A: AddressLines>(&mut self) -> CpuResult<u16> {
        let v = self.fetch_word::<A>()?;
        alu::nz::<u16>(&mut self.regs.flags, Flags::NZ.bits(), u32::from(v));
        self.regs.flags.set(Flags::V, false);
        Ok(v)
    }

    fn lda<A: AddressLines>(&mut self) -> CpuResult<()> {
        let i0 = self.load_reg_byte::<A>()?;
        self.regs.a = i0;
        Ok(())
    }

    fn ldb<A: AddressLines>(&mut self) -> CpuResult<()> {
        let i0 = self.load_reg_byte::<A>()?;
        self.regs.b = i0;
        Ok(())
    }

    fn ldd<A: AddressLines>(&mut self) -> CpuResult<()> {
        let i0 = self.load_reg_word::<A>()?;
        self.regs.set_d(i0);
        Ok(())
    }

    fn ldx<A: AddressLines>(&mut self) -> CpuResult<()> {
        let i0 = self.load_reg_word::<A>()?;
        self.regs.x = i0;
        Ok(())
    }

    fn ldy<A: AddressLines>(&mut self) -> CpuResult<()> {
        let i0 = self.load_reg_word::<A>()?;
        self.regs.y = i0;
        Ok(())
    }

    fn lds<A: AddressLines>(&mut self) -> CpuResult<()> {
        let i0 = self.load_reg_word::<A>()?;
        self.regs.s = i0;
        Ok(())
    }

    fn ldu<A: AddressLines>(&mut self) -> CpuResult<()> {
        let i0 = self.load_reg_word::<A>()?;
        self.regs.u = i0;
        Ok(())
    }
    // }}}

    ////////////////////////////////////////////////////////////////////////////////

    fn pop_regs(&mut self, sf: StackFlags, is_system: bool) -> CpuResult<()> {
        if sf.contains(StackFlags::CC) {
            let i0 = self.pop_byte(is_system)?;
            self.regs.flags.set_flags(i0);
            self.cycles += 1;
        }

        if sf.contains(StackFlags::A) {
            let i0 = self.pop_byte(is_system)?;
            self.regs.a = i0;
            self.cycles += 1;
        }

        if sf.contains(StackFlags::B) {
            let i0 = self.pop_byte(is_system)?;
            self.regs.b = i0;
            self.cycles += 1;
        }

        if sf.contains(StackFlags::DP) {
            let i0 = self.pop_byte(is_system)?;
            self.regs.dp = i0;
            self.cycles += 1;
        }

        if sf.contains(StackFlags::X) {
            let i0 = self.pop_word(is_system)?;
            self.regs.x = i0;
            self.cycles += 2;
        }

        if sf.contains(StackFlags::Y) {
            let i0 = self.pop_word(is_system)?;
            self.regs.y = i0;
            self.cycles += 2;
        }

        if sf.contains(StackFlags::STACK) {
            let v = self.pop_word(is_system)?;
            // Set the other stack pointer
            self.set_stack(v, !is_system);
            self.cycles += 2;
        }

        if sf.contains(StackFlags::PC) {
            let i0 = self.pop_word(is_system)?;
            self.set_next_pc(i0 as usize);
            self.cycles += 2;
        }

        Ok(())
    }

    fn push_regs(&mut self, sf: StackFlags, is_system: bool) -> CpuResult<()> {
        if sf.contains(StackFlags::PC) {
            let i0 = self.get_pc() as u16;
            self.push_word(i0, is_system)?;
            self.cycles += 2;
        }

        if sf.contains(StackFlags::STACK) {
            // Get the other stack pointer
            let sp = self.get_stack(!is_system);
            self.push_word(sp, is_system)?;
            self.cycles += 2;
        }

        if sf.contains(StackFlags::Y) {
            let i0 = self.regs.y;
            self.push_word(i0, is_system)?;
            self.cycles += 2;
        }

        if sf.contains(StackFlags::X) {
            let i0 = self.regs.x;
            self.push_word(i0, is_system)?;
            self.cycles += 2;
        }

        if sf.contains(StackFlags::DP) {
            let i0 = self.regs.dp;
            self.push_byte(i0, is_system)?;
            self.cycles += 1;
        }

        if sf.contains(StackFlags::B) {
            let i0 = self.regs.b;
            self.push_byte(i0, is_system)?;
            self.cycles += 1;
        }

        if sf.contains(StackFlags::A) {
            let i0 = self.regs.a;
            self.push_byte(i0, is_system)?;
            self.cycles += 1;
        }

        if sf.contains(StackFlags::CC) {
            let i0 = self.regs.flags.bits();
            self.push_byte(i0, is_system)?;
            self.cycles += 1;
        }
        Ok(())
    }

    fn pshs<A: AddressLines>(&mut self) -> CpuResult<()> {
        let op = self.fetch_byte::<A>()?;
        let sf = StackFlags::from_bits(op).unwrap();
        self.push_regs(sf, true)
    }

    fn pshu<A: AddressLines>(&mut self) -> CpuResult<()> {
        let op = self.fetch_byte::<A>()?;
        let sf = StackFlags::from_bits(op).unwrap();
        self.push_regs(sf, false)
    }

    fn puls<A: AddressLines>(&mut self) -> CpuResult<()> {
        let op = self.fetch_byte::<A>()?;
        let sf = StackFlags::from_bits(op).unwrap();
        self.pop_regs(sf, true)?;
        Ok(())
    }

    fn pulu<A: AddressLines>(&mut self) -> CpuResult<()> {
        let op = self.fetch_byte::<A>()?;
        let sf = StackFlags::from_bits(op).unwrap();
        self.pop_regs(sf, false)?;
        Ok(())
    }

    #[allow(clippy::extra_unused_type_parameters)]
    fn mul<A: AddressLines>(&mut self) -> CpuResult<()> {
        let i0 = self.regs.a;
        let i1 = self.regs.b;

        let r = u16::mul(
            &mut self.regs.flags,
            Flags::NZC.bits(),
            u32::from(i0),
            u32::from(i1),
        );

        self.regs.set_d(r);
        Ok(())
    }

    ////////////////////////////////////////////////////////////////////////////////

    fn leax<A: AddressLines>(&mut self) -> CpuResult<()> {
        let ea = self.ea::<A>()?;
        self.regs.flags.set(Flags::Z, ea == 0);
        self.regs.x = ea;
        Ok(())
    }

    fn leay<A: AddressLines>(&mut self) -> CpuResult<()> {
        let ea = self.ea::<A>()?;
        self.regs.flags.set(Flags::Z, ea == 0);
        self.regs.y = ea;
        Ok(())
    }

    fn leas<A: AddressLines>(&mut self) -> CpuResult<()> {
        let ea = self.ea::<A>()?;
        self.regs.s = ea;
        Ok(())
    }

    fn leau<A: AddressLines>(&mut self) -> CpuResult<()> {
        let ea = self.ea::<A>()?;
        self.regs.u = ea;
        Ok(())
    }

    ////////////////////////////////////////////////////////////////////////////////

    fn neg<A: AddressLines>(&mut self) -> CpuResult<()> {
        self.rwmod8::<A>(Flags::NZVC.bits(), u8::neg)?;
        Ok(())
    }

    #[allow(clippy::extra_unused_type_parameters)]
    fn nega<A: AddressLines>(&mut self) -> CpuResult<()> {
        self.moda(Flags::NZVC.bits(), u8::neg);
        Ok(())
    }

    #[allow(clippy::extra_unused_type_parameters)]
    fn negb<A: AddressLines>(&mut self) -> CpuResult<()> {
        self.modb(Flags::NZVC.bits(), u8::neg);
        Ok(())
    }

    ////////////////////////////////////////////////////////////////////////////////

    #[allow(clippy::extra_unused_type_parameters)]
    fn nop<A: AddressLines>(&mut self) -> CpuResult<()> {
        Ok(())
    }

    ////////////////////////////////////////////////////////////////////////////////
    fn rol<A: AddressLines>(&mut self) -> CpuResult<()> {
        self.rwmod8::<A>(Flags::NZVC.bits(), u8::rol)?;
        Ok(())
    }

    #[allow(clippy::extra_unused_type_parameters)]
    fn rola<A: AddressLines>(&mut self) -> CpuResult<()> {
        self.moda(Flags::NZVC.bits(), u8::rol);
        Ok(())
    }

    #[allow(clippy::extra_unused_type_parameters)]
    fn rolb<A: AddressLines>(&mut self) -> CpuResult<()> {
        self.modb(Flags::NZVC.bits(), u8::rol);
        Ok(())
    }

    ////////////////////////////////////////////////////////////////////////////////
    fn ror<A: AddressLines>(&mut self) -> CpuResult<()> {
        self.rwmod8::<A>(Flags::NZC.bits(), u8::ror)?;
        Ok(())
    }

    #[allow(clippy::extra_unused_type_parameters)]
    fn rora<A: AddressLines>(&mut self) -> CpuResult<()> {
        self.moda(Flags::NZC.bits(), u8::ror);
        Ok(())
    }
    #[allow(clippy::extra_unused_type_parameters)]
    fn rorb<A: AddressLines>(&mut self) -> CpuResult<()> {
        self.modb(Flags::NZC.bits(), u8::ror);
        Ok(())
    }

    ////////////////////////////////////////////////////////////////////////////////
    fn sbca<A: AddressLines>(&mut self) -> CpuResult<()> {
        self.moda_2::<A>(Flags::NZVC.bits(), u8::sbc)?;
        Ok(())
    }

    fn sbcb<A: AddressLines>(&mut self) -> CpuResult<()> {
        self.modb_2::<A>(Flags::NZVC.bits(), u8::sbc)?;
        Ok(())
    }
    fn suba<A: AddressLines>(&mut self) -> CpuResult<()> {
        self.moda_2::<A>(Flags::NZVC.bits(), u8::sub)?;
        Ok(())
    }
    fn subb<A: AddressLines>(&mut self) -> CpuResult<()> {
        self.modb_2::<A>(Flags::NZVC.bits(), u8::sub).map(|_| ())
    }

    #[allow(clippy::extra_unused_type_parameters)]
    fn tsta<A: AddressLines>(&mut self) -> CpuResult<()> {
        self.moda(Flags::NZV.bits(), u8::tst);
        Ok(())
    }

    #[allow(clippy::extra_unused_type_parameters)]
    fn tstb<A: AddressLines>(&mut self) -> CpuResult<()> {
        self.modb(Flags::NZV.bits(), u8::tst);
        Ok(())
    }

    fn tst<A: AddressLines>(&mut self) -> CpuResult<()> {
        self.rwmod8::<A>(Flags::NZV.bits(), u8::tst)?;
        Ok(())
    }

    ////////////////////////////////////////////////////////////////////////////////
    #[allow(clippy::extra_unused_type_parameters)]
    fn sex<A: AddressLines>(&mut self) -> CpuResult<()> {
        if self.regs.b & 0x80 == 0x80 {
            self.regs.a = 0xff;
        } else {
            self.regs.a = 0;
        }

        let d = self.regs.get_d();

        alu::nz::<u16>(&mut self.regs.flags, Flags::NZ.bits(), u32::from(d));
        Ok(())
    }

    #[allow(clippy::extra_unused_type_parameters)]
    fn swi_base<A: AddressLines>(&mut self, vec: usize, flags: Flags) -> CpuResult<()> {
        macro_rules! push8 {
            ($val:expr) => {{
                let i0 = $val;
                self.push_byte(i0, true)?
            }};
        }

        macro_rules! push16 {
            ($val:expr) => {{
                let i0 = $val;
                self.push_word(i0 as u16, true)?
            }};
        }

        self.regs.flags |= flags;

        push16!(self.get_pc());
        push16!(self.regs.u);
        push16!(self.regs.y);
        push16!(self.regs.x);

        push8!(self.regs.dp);
        push8!(self.regs.b);
        push8!(self.regs.a);

        push8!(self.regs.flags.bits());

        let pc = self.mem.load_word(vec)?;
        self.set_next_pc(pc as usize);
        Ok(())
    }

    fn swi<A: AddressLines>(&mut self) -> CpuResult<()> {
        self.swi_base::<A>(VEC_SWI, Flags::E | Flags::F)
    }

    fn swi2<A: AddressLines>(&mut self) -> CpuResult<()> {
        self.swi_base::<A>(VEC_SWI2, Flags::E)
    }

    fn swi3<A: AddressLines>(&mut self) -> CpuResult<()> {
        self.swi_base::<A>(VEC_SWI2, Flags::E)
    }

    fn subd<A: AddressLines>(&mut self) -> CpuResult<()> {
        let i0 = self.regs.get_d();
        let r = self.op16_2::<A>(Flags::NZVC.bits(), u16::sub, i0)?;
        self.regs.set_d(r);
        Ok(())
    }

    fn jmp<A: AddressLines>(&mut self) -> CpuResult<()> {
        let a = self.ea::<A>()?;
        self.set_next_pc(a as usize);
        Ok(())
    }

    #[allow(clippy::extra_unused_type_parameters)]
    fn rti<A: AddressLines>(&mut self) -> CpuResult<()> {
        // pop CC
        // regs if E flag is set
        // PC
        let cc = self.pop_byte(true)?;
        self.regs.flags.set_flags(cc);

        // pop saved regs if E flag is set
        if self.regs.flags.contains(Flags::E) {
            self.pop_regs(StackFlags::INT_FLAGS, true)?;
            self.cycles += 9;
        }

        let pc = self.pop_word(true)? as usize;

        self.set_next_pc(pc);

        Ok(())
    }

    #[allow(clippy::extra_unused_type_parameters)]
    fn cwai<A: AddressLines>(&mut self) -> CpuResult<()> {
        self.unimplemented()
    }

    #[allow(clippy::extra_unused_type_parameters)]
    fn sync<A: AddressLines>(&mut self) -> CpuResult<()> {
        // are any of the interrupts disabled or any interrupts asserting?
        if self.regs.flags.intersects(Flags::I | Flags::F) || !self.pins.waiting_for_irq {
            Ok(())
        } else {
            self.pins.waiting_for_irq = true;
            // No interrupts pending and interrrupts are enabled
            // so set the next addr to be this addr
            self.ins.next_addr = self.ins.addr;
            Ok(())
        }
    }

    fn unimplemented(&mut self) -> CpuResult<()> {
        Err(CpuErr::Unimplemented(self.get_pc()))
    }
}

// Interrupts / reset
#[allow(unused_variables)]
impl<'a> Context<'a> {
    fn gen_irq(&mut self, irq_flag: Flags, save_regs: bool, vector: usize) -> CpuResult<()> {
        // Is this IRQ disabled?
        // TODO: should be moved into the IRQ line sensing code
        // https://github.com/elmerucr/MC6809/blob/master/src/mc6809.cpp

        if !self.regs.flags.contains(irq_flag) {
            // Save PC
            // Regs if needed
            // CC
            // Save pc
            self.push_word(self.regs.pc, true)?;

            if save_regs {
                self.regs.flags.set(Flags::E, true);
                self.push_regs(StackFlags::INT_FLAGS, true)?;
            }

            // Save cc
            self.push_byte(self.regs.flags.bits(), true)?;

            // Disable this IRQ
            self.regs.flags.set(irq_flag, true);
            // Get the irq vector
            let pc = self.mem.load_word(vector)? as usize;
            // set the PC
            self.set_next_pc(pc);
        }

        Ok(())
    }

    fn irq(&mut self) -> CpuResult<()> {
        let r = self.gen_irq(Flags::I, true, 0xfff8);
        self.ins.cycles += 19;
        r
    }

    fn firq(&mut self) -> CpuResult<()> {
        let r = self.gen_irq(Flags::F, false, 0xfff6);
        self.ins.cycles += 10;
        r
    }

    fn nmi(&mut self) -> CpuResult<()> {
        self.push_regs(StackFlags::PC | StackFlags::STACK, true)?;
        self.push_regs(
            StackFlags::Y | StackFlags::X | StackFlags::DP | StackFlags::B | StackFlags::A,
            true,
        )?;
        self.regs.flags.set(Flags::E, true);
        self.push_regs(StackFlags::CC, true)?;
        self.regs.flags.set(Flags::I | Flags::F, true);
        self.ins.cycles += 19;
        Ok(())
    }
}

// Public interface
#[allow(unused_variables)]
impl<'a> Context<'a> {
    pub fn cycles(&self) -> usize {
        self.cycles
    }

    pub fn instructions(&self) -> usize {
        self.instructions
    }

    pub fn get_pc(&self) -> usize {
        self.ins.next_addr & 0xffff
    }

    pub fn new(
        mem: &'a mut dyn MemoryIO,
        regs: &'a mut Regs,
        pins: &'a mut Pins,
    ) -> CpuResult<Context<'a>> {
        let ins = InstructionDecoder::new_from_read_mem(regs.pc as usize, mem)?;
        let ret = Context {
            regs,
            mem,
            ins,
            pins,
            cycles: 0,
            instructions: 0,
        };
        Ok(ret)
    }

    fn clear_pending_irq(&mut self) {
        self.pins.waiting_for_irq = false;
    }

    fn is_pending_irq(&self) -> bool {
        self.pins.waiting_for_irq
    }

    pub fn step(&mut self) -> CpuResult<()> {
        if self.pins.irq {
            self.irq()?;
            self.clear_pending_irq();
        } else if self.pins.firq {
            self.firq()?;
            self.clear_pending_irq();
        } else if self.pins.nmi {
            self.nmi()?;
            self.clear_pending_irq();
        } else {
            self.ins = InstructionDecoder::new_from_read_mem(self.regs.pc as usize, self.mem)?;

            macro_rules! handle_op {
                ($addr:ident, $action:ident, $opcode:expr, $cycles:expr, $size:expr) => {{
                    self.$action::<$addr>()
                }};
            }

            let opcode = self.ins.instruction_info.opcode;

            op_table!(opcode, { self.unimplemented() })?;
        }

        self.regs.pc = self.ins.next_addr as u16;
        self.cycles += self.ins.cycles;
        self.instructions += 1;

        Ok(())
    }

    pub fn reset(&mut self) -> CpuResult<()> {
        let pc = self.mem.load_word(VEC_RESET)?;

        log::info!("PC IS {:04x}", pc);

        *self.regs = Regs {
            pc,
            dp: 0,
            flags: Flags::I | Flags::F,
            ..Default::default()
        };
        Ok(())
    }
}

//
// }}}
