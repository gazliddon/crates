#![allow(unused_imports)]
#![allow(dead_code)]

use emu6800::cpu_core::{AddrModeEnum, Mnemonic, Isa, IsaDatabase};

use emu6800::cpu::{
    self, Bus, Ins, Machine, RegisterFile, RegisterFileTrait,
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

static SND : &[u8;2048] = include_bytes!("../resources/sg.snd");

fn make_machine() -> Machine<MemBlock<BigEndian>, RegisterFile> {
    let regs = RegisterFile::default();
    let mut mem: MemBlock<BigEndian> = MemBlock::new("test", false, &(0..65536));
    mem.store_bytes(0xf800, SND).unwrap();
    Machine::new(mem, regs)
}

fn try_diss() {

    let m = make_machine();

    let mut pc = 0xf800 + 1;
    loop {
        let d = m.diss(pc);

        if let Ok(d) = d {
            let cycles = d.ins.opcode_data.cycles;

            println!("{pc:04x} {:19} [ {cycles} ]    {}", d.mem_string, d.text);
            pc = d.next_pc;
        } else {
            println!("Uknown: {pc:04x} : {:02x}", m.mem().inspect_byte(pc ).unwrap());
            break;
        }
    }

}


fn main() {
    try_diss();
}
