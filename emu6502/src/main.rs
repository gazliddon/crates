use emu6502::*;
use cpu_core::Instructions;

fn main() {
    // load in a json file with the program
    let program = include_str!("../resources/opcodes6502.json");

    let x : Instructions = serde_json::from_str(program).unwrap();

    println!("{:#?}", x);
}
