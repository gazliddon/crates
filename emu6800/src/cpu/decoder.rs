use std::env;

pub fn print_it() {
    let file_name = format!("{}/isa_macros_6800.rs", std::env::var("OUT_DIR").unwrap());
    println!("FILE IS {file_name}");
}
