//! Throughput benchmark: decode, execute, and disassemble.
//!
//! Usage: `cargo run --release -p emujrisc --example bench -- [passes]`

use emujrisc::cpu::{Chip, Cpu};
use emujrisc::diss::{Diss, DissCtx};
use emujrisc::mem::SliceMem;
use emujrisc::mem::{JriscBus, RamBus};
use std::time::Instant;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let passes: usize = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(10_000);

    let dsp: &[u8] = include_bytes!("../tests/data/dsp.bin");
    let base = 0xF1B000usize;
    let end = base + dsp.len();

    // ---- decode (linear walk of the DSP program) ----
    let mut count = 0usize;
    {
        let mut mem = SliceMem::new(base, dsp);
        let mut pc = base;
        while pc < end {
            let d = emujrisc::cpu::decode(&mut mem, pc, Chip::Dsp).unwrap();
            pc += d.size;
            count += 1;
        }
    }
    let t0 = Instant::now();
    let mut n = 0usize;
    for _ in 0..passes {
        let mut mem = SliceMem::new(base, dsp);
        let mut pc = base;
        while pc < end {
            let d = emujrisc::cpu::decode(&mut mem, pc, Chip::Dsp).unwrap();
            pc += d.size;
            n += 1;
        }
    }
    let dt = t0.elapsed().as_secs_f64();
    println!(
        "decode: {} instr/pass, {} passes -> {:.0} instr/s",
        count,
        passes,
        n as f64 / dt
    );

    // ---- execute (the real T2K DSP program on RamBus) ----
    let steps = passes * 1000;
    let mut cpu = Cpu::new(Chip::Dsp);
    cpu.pc = 0xF1B000;
    let mut bus = RamBus::new(Chip::Dsp, dsp.to_vec());
    // warm-up + sanity: the smoke-test anchors
    for _ in 0..110 {
        cpu.step(&mut bus).expect("warm-up step");
    }
    let t1 = Instant::now();
    let mut done = 0u64;
    for _ in 0..steps {
        if cpu.step(&mut bus).is_err() {
            break;
        }
        done += 1;
    }
    let dt1 = t1.elapsed().as_secs_f64();
    println!(
        "exec: {} steps in {:.3}s -> {:.0} instr/s ({:.0} cycles/s)",
        done,
        dt1,
        done as f64 / dt1,
        cpu.stats.cycles as f64 / dt1
    );

    // ---- disassemble ----
    let ctx = DissCtx::from_slice(base, "dsp", dsp);
    let diss = Diss::new(Chip::Dsp);
    let t2 = Instant::now();
    let mut lines = 0usize;
    for _ in 0..passes {
        let mut pc = base;
        while pc < end {
            let d = ctx.diss(&diss, pc).unwrap();
            pc += d.size;
            lines += 1;
        }
    }
    let dt2 = t2.elapsed().as_secs_f64();
    println!(
        "diss: {} lines/pass, {} passes -> {:.0} lines/s",
        count,
        passes,
        lines as f64 / dt2
    );
}
