// use serde::{Deserialize, Serialize};
// use serde_json::{Result, Value};

use super::{ IsaDatabase, Isa };

lazy_static::lazy_static! {
    static ref DBASE : IsaDatabase = {
        let txt = include_str!("../../resources/opcodes6800.json");
        let isa: Isa = serde_json::from_str(txt).unwrap();
        IsaDatabase::new(&isa)
    };
}

pub fn gen_instruction_str() -> String{

    let mut out = vec![];
    
    for op in 0..256 {
        if let Some(ins) = DBASE.get_instruction_info_from_opcode(op) {
            let mnem = format!("{:?}",ins.mnemonic).to_lowercase();
            let addr_mode = format!("{:?}",ins.addr_mode);
            let cycles = ins.opcode_data.cycles;
            let size = ins.opcode_data.size;
            let line = format!("\t\t0x{op:02x} => handle_op!({mnem}, {addr_mode}, {cycles}, {size}),");
            out.push(line)
        } 
    }

    out.join("\n")
}

