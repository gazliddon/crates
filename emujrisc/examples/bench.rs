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

    // ---- execute, idle: the real T2K DSP program (its F1B330 spin) ----
    let steps = passes * 1000;
    let (i1, cy1) = run(&dsp.to_vec(), steps, None);
    println!(
        "exec idle: {} steps -> {:.0} instr/s ({:.0} cycles/s)",
        i1, i1, cy1
    );

    // ---- execute, busy: the spin patched to nop + the 68000 signal
    // (tick/command mailboxes + a channel record) — full pipeline:
    // record walk, command dispatch, mixing, stereo output ----
    let mut img = dsp.to_vec();
    img[0x332] = 0xE4; // patch `jr eq,$F1B330` -> nop
    img[0x333] = 0x00;
    let (i2, cy2) = run(&img, steps, Some(0x100));
    println!(
        "exec busy: {} steps -> {:.0} instr/s ({:.0} cycles/s)",
        i2, i2, cy2
    );

/// Run `steps` instructions of the DSP program; `samples` (if set) writes
/// the 68000-side signal: a channel record requesting that many samples.
fn run(img: &[u8], steps: usize, samples: Option<u32>) -> (f64, f64) {
    let mut cpu = Cpu::new(Chip::Dsp);
    cpu.pc = 0xF1B000;
    let mut bus = RamBus::new(Chip::Dsp, img.to_vec());
    for _ in 0..500 {
        cpu.step(&mut bus).expect("warm-up step");
    }
    if let Some(n) = samples {
        bus.write_long(0xF1B35C, 0); // tick mailbox: zero -> mixing path
        bus.write_long(0xF1B358, 1); // command mailbox: bit0 gate
        bus.write_long(0xF1B800, 0); // record[0]: process
        bus.write_long(0xF1B80C, n); // field +6 -> sample count
        bus.write_long(0xF1B820, 0xFFFF_FFFC); // -4 terminator
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
    (done as f64 / dt1, cpu.stats.cycles as f64 / dt1)
}

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
