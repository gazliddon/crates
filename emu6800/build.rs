#![allow(dead_code)]
#[path="src/cpu_core/mod.rs"]
mod cpu_core;

use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;

fn make_string(body: String) -> String {
        let header = r#"#[macro_export]
macro_rules! op_table {
    ($op:expr, $fail:block) => {
        match $op {"#;

        let footer = r#"
            _ => $fail
        }
    }
}"#;
    format!("{header}\n{body}\n{footer}\n")
}

fn main() {
    let body = cpu_core::gen_instruction_str();
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("isa_macros_6800.rs");
    let out = make_string(body.clone());
    let mut f = File::create(dest_path).unwrap();
    f.write_all(out.as_bytes()).unwrap();
}

