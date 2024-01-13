#![allow(unused_imports)]
#![allow(dead_code)]

use emu6800::cpu_core::{AddrModeEnum, Mnemonic, Isa, IsaDatabase};

use emu6800::cpu::{
    self, diss, Bus, Ins, Machine, RegisterFile, RegisterFileTrait,
    StatusRegTrait, decoder::print_it,
};

use emucore::{
    instructions::InstructionInfoTrait,
    mem::{MemBlock, MemoryIO},
    byteorder::*,
};
use std::{
    collections::{HashMap, HashSet},
    fmt::format,
    str::FromStr, io::BufWriter,
};

use serde::{Deserialize, Serialize};
use serde_json::{Result, Value};

lazy_static::lazy_static! {
    static ref DBASE : IsaDatabase = {
        let txt = include_str!("../resources/opcodes6800.json");
        let isa: Isa = serde_json::from_str(txt).unwrap();
        IsaDatabase::new(&isa)
    };
}

static SND : &[u8;2048] = include_bytes!("../resources/sg.snd");

fn make_machine() -> Machine<MemBlock<BigEndian>, RegisterFile> {
    let regs = RegisterFile::default();
    let mut mem: MemBlock<BigEndian> = MemBlock::new("test", false, &(0..65536));
    mem.store_bytes(0xf800, SND).unwrap();
    let machine = Machine::new(mem, regs);
    machine
}

fn try_diss() {  
    let m = make_machine();
    let mut pc = 0xf800 + 1;
    loop {
        let d = diss(m.mem(), pc, &DBASE);

        if let Ok(d) = d {
            let cycles = d.ins.opcode_data.cycles;

            println!("{pc:04x} {:19} [ {cycles} ]    {}", d.mem_string, d.text);
            pc = d.next_pc;
        } else {
            println!("Uknown: {pc:04x} : {:02x}", m.mem().inspect_byte(pc as usize).unwrap());
            break;
        }
    }

}

fn try_step() { 
    let mut m  = make_machine();
    let data = [
        0x86, 0x3e, 0xb7, 0xe4, 0x1d, 0x86, 0x6d, 0xb7, 0xe4, 0x1e, 0x86, 0x79, 0xb7, 0xe4, 0x1f,
        0x86, 0x00, 0xb7, 0xe4, 0x20, 0x86, 0x5e, 0xb7, 0xe4, 0x21, 0x86, 0x6d, 0xb7, 0xe4, 0x22,
        0xce, 0xf0, 0xa2, 0xff, 0xe4, 0x19, 0x7e, 0xf0, 0xbb,
    ];

    m.mem_mut().store_bytes(0, &data).unwrap();
    m.regs.set_pc(0);
    m.regs.sev().sei().sec();
    println!("{}\n", m.regs);

    loop {
        let pc = m.regs.pc();

        m.step().unwrap();
        println!("{}", m.regs);
        let d = diss(m.mem(), pc as usize, &DBASE);
        if let Ok(d) = d {
            let cycles = d.ins.opcode_data.cycles;
            println!("{pc:04x} [ {cycles} ]  {}",  d.text);
        } else {
            break;
        }
        println!("");
    }
}


fn main() {
    try_diss();
}
