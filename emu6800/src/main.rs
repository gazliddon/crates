#![allow(unused_imports)]
#![allow(dead_code)]

use cpu::{AddrModeEnum, Isa, Mnemonic, OpcodeData};
use emu6800::cpu::{
    self, diss, Bus, Ins, Instruction, IsaDatabase, Machine, RegisterFile, RegisterFileTrait,
    StatusRegTrait,
};
use emucore::{
    instructions::InstructionInfoTrait,
    mem::{MemBlock, MemoryIO},
};
use std::{
    collections::{HashMap, HashSet},
    fmt::format,
    str::FromStr,
};

use itertools::Itertools;
use serde::{Deserialize, Serialize};
use serde_json::{Result, Value};

lazy_static::lazy_static! {
    static ref DBASE : IsaDatabase = {
        let txt = include_str!("../resources/opcodes6800.json");
        let isa: Isa = serde_json::from_str(txt).unwrap();
        IsaDatabase::new(&isa)
    };

}

fn main() {
    use emucore::byteorder::BigEndian;

    let mem: MemBlock<BigEndian> = MemBlock::new("test", false, &(0..65536));

    let regs = RegisterFile::default();
    let mut machine = Machine::new(mem, regs);


    let data = [
        0x86, 0x3e, 0xb7, 0xe4, 0x1d, 0x86, 0x6d, 0xb7, 0xe4, 0x1e, 0x86, 0x79, 0xb7, 0xe4, 0x1f,
        0x86, 0x00, 0xb7, 0xe4, 0x20, 0x86, 0x5e, 0xb7, 0xe4, 0x21, 0x86, 0x6d, 0xb7, 0xe4, 0x22,
        0xce, 0xf0, 0xa2, 0xff, 0xe4, 0x19, 0x7e, 0xf0, 0xbb,
    ];

    machine.mem_mut().store_bytes(0, &data).unwrap();

    let mut pc = 0;

    for _i in 0..15 {
        let d = diss(machine.mem(), pc, &DBASE).unwrap();
        let mem_txt = &d.mem_string;
        let txt = &d.text;

        println!("{pc:04x} {mem_txt:10} {txt}");
        pc = d.next_pc;
    }
}
