use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;

#[path = "src/isa/mod.rs"]
mod isa;

fn main() {
    println!("ABOUT TO DO IT");
    let js_str = include_str!("resources/opcodes6809.json");
    let dbase = isa::Dbase::from_text(js_str);
    let source = format!("{dbase}");
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("isa_macros_6809.rs");

    let mut f = File::create(dest_path).unwrap();
    f.write_all(source.as_bytes()).unwrap();
}
