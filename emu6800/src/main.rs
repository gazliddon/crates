#![allow(unused_imports)]
#![allow(dead_code)]

use cpu::{AddrModeEnum, Isa, Mnemonic, OpcodeData};
use emu6800::cpu::{self, RegisterFile, Machine, Instruction, Ins, StatusRegTrait, RegisterFileTrait};
use emucore::{mem::MemBlock, instructions::InstructionInfoTrait};
use std::{
    collections::{HashMap, HashSet},
    fmt::format,
    str::FromStr,
};

use itertools::Itertools;
use serde::{Deserialize, Serialize};
use serde_json::{Result, Value};

fn main() {
    use emucore::byteorder::BigEndian;
    use emucore::mem::MemoryIO;
    use emu6800::cpu::Immediate;

    let mut mem: MemBlock<BigEndian> = MemBlock::new("test", false, &(0..65536));

    let ldaa_imm = &[0x86,0x80];
    mem.store_bytes(0,ldaa_imm).unwrap();

    let mut regs = RegisterFile::default();
    regs.sec();
    regs.inc_pc();
    let mut machine = Machine::new(mem, regs);
    println!("regs: {:?}", machine.regs);

    let mut ins =  Ins::new(Immediate, &mut machine);
    ins.ldaa().unwrap();

    println!("regs: {:?}", machine.regs);
}
