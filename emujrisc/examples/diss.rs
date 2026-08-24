//! Dump a labeled disassembly of a JRISC image.
//! Usage: diss <chip: gpu|dsp> <base-hex> <image>
use emujrisc::cpu::Chip;
use emujrisc::diss::{Diss, DissCtx};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let chip = match args.get(1).map(|s| s.as_str()) {
        Some("gpu") => Chip::Gpu,
        Some("dsp") => Chip::Dsp,
        _ => {
            eprintln!("usage: diss <gpu|dsp> <base-hex> <image>");
            std::process::exit(1);
        }
    };
    let base = usize::from_str_radix(args[2].trim_start_matches("0x"), 16).unwrap();
    let data = std::fs::read(&args[3]).unwrap();
    let ctx = DissCtx::from_slice(base, "image", &data);
    let diss = Diss::new(chip);
    let mut pc = base;
    while pc < base + data.len() {
        match ctx.diss(&diss, pc) {
            Ok(d) => {
                println!("{:06X}  {}", d.addr, d.text);
                pc += d.size;
            }
            Err(_) => {
                println!("{:06X}  (unterminated)", pc);
                break;
            }
        }
    }
}
