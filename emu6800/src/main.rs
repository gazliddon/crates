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

fn main() {
    print_it();
    // use std::io::Write;


    // let mut out = vec![];
    
    // for op in 0..256 {
    //     if let Some(ins) = DBASE.get_instruction_info_from_opcode(op) {
    //         let mnem = format!("{:?}",ins.mnemonic).to_lowercase();
    //         let addr_mode = format!("{:?}",ins.addr_mode);
    //         let cycles = ins.opcode_data.cycles;
    //         let size = ins.opcode_data.size;
    //         let line = format!("\t\t0x{op:02x} => handle_op!({mnem}, {addr_mode}, {cycles}, {size}),");
    //         out.push(line)
    //     } 
    // }
    // let out = out.join("\n");

    // println!("{}", out);

    // use emucore::byteorder::BigEndian;

    // let mem: MemBlock<BigEndian> = MemBlock::new("test", false, &(0..65536));

    // let regs = RegisterFile::default();
    // let mut machine = Machine::new(mem, regs);

    // let data = [
    //     0x86, 0x3e, 0xb7, 0xe4, 0x1d, 0x86, 0x6d, 0xb7, 0xe4, 0x1e, 0x86, 0x79, 0xb7, 0xe4, 0x1f,
    //     0x86, 0x00, 0xb7, 0xe4, 0x20, 0x86, 0x5e, 0xb7, 0xe4, 0x21, 0x86, 0x6d, 0xb7, 0xe4, 0x22,
    //     0xce, 0xf0, 0xa2, 0xff, 0xe4, 0x19, 0x7e, 0xf0, 0xbb,
    // ];

    // machine.mem_mut().store_bytes(0, &data).unwrap();

    // let mut pc = 0;

    // loop {
    //     let d = diss(machine.mem(), pc, &DBASE);

    //     if let Ok(d) = d {
    //         let cycles = d.ins.opcode_data.cycles;

    //         println!("{pc:04x} {:19} [ {cycles} ]    {}", d.mem_string, d.text);
    //         pc = d.next_pc;
    //     } else {
    //         break;
    //     }
    // }
}
