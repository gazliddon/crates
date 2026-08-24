// use serde::{Deserialize, Serialize};
// use serde_json::{Result, Value};

use super::{Isa, IsaDatabase};

lazy_static::lazy_static! {
    pub static ref DBASE : IsaDatabase = {
        let txt = include_str!("../../resources/opcodes6800.json");
        let isa: Isa = serde_json::from_str(txt).unwrap();
        IsaDatabase::new(&isa)
    };
}

pub fn gen_instruction_str() -> String {
    let mut out = vec![];

    for op in 0..256 {
        if let Some(ins) = DBASE.get_instruction_info_from_opcode(op) {
            let mnem = format!("{:?}", ins.mnemonic).to_lowercase();
            // Undocumented opcodes share a handful of executor actions:
            // the IllegalXX NOPs differ only in how many operand bytes
            // they skip, and the silicon quirks have their own actions.
            let action = match mnem.as_str() {
                m if m.starts_with("illegal") => match ins.opcode_data.size {
                    1 => "illegal_nop",
                    2 => "illegal_nop2",
                    _ => "illegal_nop3",
                },
                "brn" => "brn",
                "staim" => "sta_ea_pc",
                "stbim" => "stb_ea_pc",
                "stsim" => "sts_ea_pc",
                "stxim" => "stx_ea_pc",
                "jsrundoc" => "jsr_direct",
                other => other,
            };
            let addr_mode = format!("{:?}", ins.addr_mode);
            let cycles = ins.opcode_data.cycles;
            let size = ins.opcode_data.size;
            let line =
                format!("\t\t0x{op:02x} => handle_op!({action}, {addr_mode}, {cycles}, {size}),");
            out.push(line)
        }
    }

    out.join("\n")
}
