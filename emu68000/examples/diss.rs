//! Disassemble a 68000 binary image with an optional ALN symbol table.
//!
//! Usage: `cargo run -p emu68000 --example diss -- <image> <base> [<sym>]`
//!
//! e.g. (from the crate directory):
//!   cargo run --example diss -- tests/data/TEST.TXT 0x4000 tests/data/TEST.SYM

use emu68000::cpu::decode;
use emu68000::diss::{Diss, SliceMem};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("usage: diss <image> <base-hex> [<aln-sym>]");
        std::process::exit(1);
    }
    let image = std::fs::read(&args[1]).expect("read image");
    let base = usize::from_str_radix(args[2].trim_start_matches("0x"), 16).expect("base");

    let mut labels: Vec<(u32, String)> = Vec::new();
    if let Some(sym) = args.get(3) {
        let data = std::fs::read(sym).expect("read symbol file");
        if data.len() >= 2 && data[0] == 0x60 && data[1] == 0x1B {
            let mut off = 8;
            while off + 14 <= data.len() {
                let name = String::from_utf8_lossy(&data[off..off + 8])
                    .trim_end_matches('\0')
                    .to_string();
                let typ = data[off + 8];
                let b = &data[off + 9..off + 14];
                let val = ((b[0] as u64) << 32
                    | (b[1] as u64) << 24
                    | (b[2] as u64) << 16
                    | (b[3] as u64) << 8
                    | b[4] as u64) as u32;
                if matches!(typ, 0x82 | 0xA2 | 0x84 | 0xA4) {
                    labels.push((val, name));
                }
                off += 14;
            }
        }
    }
    let diss = Diss::with_labels(labels);

    let mut mem = SliceMem::new(base, &image);
    let mut pc = base;
    while pc < base + image.len() {
        match decode(&mut mem, pc) {
            Ok(d) => {
                if let Some(name) = diss.label_at(pc as u32) {
                    println!("{name}:");
                }
                println!("{pc:04X}  {}", diss.render_line(&d));
                pc += d.size;
            }
            Err(_) => break,
        }
    }
}
