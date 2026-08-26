//! 68000 executor + decoder throughput benchmark.
//!
//! Usage: `cargo run --release -p emu68000 --example bench -- [passes]`

use emu68000::cpu::decode;
use emu68000::cpu::{bus::Ram68k, Cpu, M68kBus};
use emu68000::diss::SliceMem;
use std::time::Instant;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let passes: usize = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(10_000);

    let module = std::fs::read("emu68000/tests/data/TEST.TXT").expect("TEST.TXT");

    // ---- decode (linear walk of the module) ----
    let mut count = 0usize;
    {
        let mut mem = SliceMem::new(0x4000, &module);
        let mut pc = 0x4000usize;
        while pc < 0x70CE {
            let d = decode(&mut mem, pc).unwrap();
            pc += d.size;
            count += 1;
        }
    }
    let t0 = Instant::now();
    let mut n = 0usize;
    for _ in 0..passes {
        let mut mem = SliceMem::new(0x4000, &module);
        let mut pc = 0x4000usize;
        while pc < 0x70CE {
            let d = decode(&mut mem, pc).unwrap();
            pc += d.size;
            n += 1;
        }
    }
    let dt = t0.elapsed().as_secs_f64();
    println!(
        "decode: {} instr/pass, {passes} passes -> {:.0} instr/s",
        count,
        n as f64 / dt
    );

    // ---- execute: the real INITDSP routine (~530 instructions) ----
    let t1 = Instant::now();
    let mut instrs = 0u64;
    for _ in 0..passes {
        let mut bus = Ram68k::new();
        bus.map(0x4000, module.clone());
        bus.map(0x8000, vec![0u8; 0x800]);
        bus.map(0xF1A100, vec![0u8; 0x400]);
        bus.map(0xF1B000, vec![0u8; 0x1000]);
        let mut cpu = Cpu::new();
        cpu.regs.pc = 0x4D46;
        cpu.regs.usp = 0x8800;
        cpu.regs.sr = 0;
        cpu.push_long(&mut bus, 0x1234); // fake return address
        for _ in 0..600 {
            if cpu.regs.pc == 0x1234 {
                break;
            }
            if cpu.step(&mut bus).is_err() {
                break;
            }
        }
        instrs += cpu.stats.instructions;
    }
    let dt1 = t1.elapsed().as_secs_f64();
    println!(
        "exec INITDSP: {} instr/pass, {passes} passes -> {:.0} instr/s",
        instrs / passes as u64,
        instrs as f64 / dt1
    );
}
