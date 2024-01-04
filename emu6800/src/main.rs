#![allow(unused_imports)]
#![allow(dead_code)]

use std::{collections::{HashMap, HashSet}, fmt::format, str::FromStr};
use emu6800::cpu;
use cpu::{ AddrModeEnum, Mnemonic };

use serde::{Deserialize, Serialize};
use serde_json::{Result, Value};
use itertools::Itertools;

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct InstructionInfo {
    addr_modes: HashMap<AddrModeEnum, Instruction>,
}


#[derive(Default, Serialize, Deserialize, Debug)]
pub struct Instruction {
    pub opcode: usize,
    pub cycles: usize,
    pub size: usize,
}

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct Isa {
    instructions: HashMap<Mnemonic, InstructionInfo>,
}

impl Isa {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn add(&mut self, _ins: emu6800::cpu::InstructionData) {

        // let text = ins.text.to_case(Case::Title);
        // use convert_case::{Case, Casing};
        // let m = Mnemonic::from_str(&text).unwrap();

        // let key = self
        //     .instructions
        //     .entry(m)
        //     .or_insert_with(|| InstructionInfo {
        //         ..Default::default()
        //     });

        // key.addr_modes.insert(ins.addr_mode, ins);
    }
}

fn modify() {
    // let text = include_str!("../resources/opcodes6800.json");
    // let mut dest_isa = Isa::default();
    // let file: emu6800::cpu::Isa = serde_json::from_str(text).unwrap();

    // for i in file.instructions.iter() {
    //     dest_isa.add(i.clone())
    // }

    // let out = serde_json::to_string_pretty(&dest_isa).unwrap();
    // println!("{out}");
}

fn read_new() {
    let text = include_str!("../resources/opcodes6800.json");
    let file :Isa = serde_json::from_str(text).unwrap();
    println!("{file:#?}");
}

fn main() {
    read_new()
}

