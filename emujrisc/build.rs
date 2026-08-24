use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;

#[path = "src/isa/db.rs"]
mod isa;

fn main() {
    let js_str = include_str!("resources/opcodes_jrisc.json");
    let dbase = isa::Dbase::from_text(js_str);
    let source = format!("{dbase}");
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("isa_macros_jrisc.rs");

    let mut f = File::create(dest_path).unwrap();
    f.write_all(source.as_bytes()).unwrap();
}
