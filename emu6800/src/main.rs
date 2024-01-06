#![allow(unused_imports)]
#![allow(dead_code)]

use cpu::{AddrModeEnum, Isa, Mnemonic, OpcodeData};
use emu6800::cpu;
use std::{
    collections::{HashMap, HashSet},
    fmt::format,
    str::FromStr,
};

use itertools::Itertools;
use serde::{Deserialize, Serialize};
use serde_json::{Result, Value};

fn read_new() {
    let text = include_str!("../resources/opcodes6800.json");
    let file: Isa = serde_json::from_str(text).unwrap();
    println!("{file:#?}");
}

fn main() {
    read_new()
}
