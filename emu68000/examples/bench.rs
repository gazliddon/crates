//! Decode-throughput benchmark: walk the TEST.TXT module repeatedly.
//!
//! Usage: cargo run --release -p emu68000 --example bench -- [iterations]

use emu68000::cpu::decode;
use emu68000::diss::{Diss, SliceMem};
use std::time::Instant;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let iters: usize = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(10_000);

    let module = std::fs::read("emu68000/tests/data/TEST.TXT").expect("TEST.TXT");
    let base = 0x4000usize;
    let end = base + module.len() - 2; // stop before the padding word

    // warm up + count instructions per pass
    let mut count = 0usize;
    {
        let mut mem = SliceMem::new(base, &module);
        let mut pc = base;
        while pc < end {
            let d = decode(&mut mem, pc).unwrap();
            pc += d.size;
            count += 1;
        }
    }
    println!("module: {} bytes, {} instructions per pass", module.len(), count);

    // full pass, decode only (no rendering)
    let t0 = Instant::now();
    let mut total = 0usize;
    for _ in 0..iters {
        let mut mem = SliceMem::new(base, &module);
        let mut pc = base;
        while pc < end {
            let d = decode(&mut mem, pc).unwrap();
            pc += d.size;
        }
        total += count;
    }
    let dt = t0.elapsed();
    let ip = total as f64 / dt.as_secs_f64();
    println!(
        "decode-only: {} passes in {:?}  ->  {:.0} instr/s  ({:.1} MB/s)",
        iters,
        dt,
        ip,
        ip as f64 * 4.0 / 1e6
    );

    // full pass with rendering + label resolution
    let labels: Vec<(u32, String)> = vec![(0x4D46, "INITDSP".into()), (0x4E1A, "INIT_SOU".into())];
    let diss = Diss::with_labels(labels);
    let t1 = Instant::now();
    let mut chars = 0usize;
    for _ in 0..iters {
        let mut mem = SliceMem::new(base, &module);
        let mut pc = base;
        while pc < end {
            let d = decode(&mut mem, pc).unwrap();
            let line = diss.render_line(&d);
            chars += line.len();
            pc += d.size;
        }
    }
    let dt1 = t1.elapsed();
    let ip1 = total as f64 / dt1.as_secs_f64();
    println!(
        "decode+render: {} passes in {:?}  ->  {:.0} instr/s  ({:.1} MB/s listing)",
        iters,
        dt1,
        ip1,
        chars as f64 / dt1.as_secs_f64() / 1e6
    );
}
