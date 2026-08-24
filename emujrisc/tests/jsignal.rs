//! Driving the real T2K DSP program into its busy pipeline.
//!
//! The program's idle state is a two-instruction spin at $F1B330
//! (`cmpq #0,r0; jr eq,$F1B330`) — r0 is a sample counter that the
//! channel-record walk sets into the *alternate* register bank
//! (`moveta r8,r0`), and the mixing path selects REGPAGE before the
//! sample loop. To keep the DSP in the full pipeline for a sustained
//! run (benchmarking), this test patches the spin to `nop` in a copy of
//! the image and signals it like the 68000 does: the tick mailbox
//! ($F1B35C) zeroed, the command mailbox ($F1B358) gate set, and a
//! channel record (+6 = sample count) with a -4 terminator at TABLESTA
//! ($F1B800). The DSP then loops: record walk -> command dispatch ->
//! mixing (imultn/imacn chains) -> stereo output (sharq/sat16s/store to
//! $F1A148/$F1A14C).

use emujrisc::cpu::{Chip, Cpu};
use emujrisc::mem::{JriscBus, RamBus};
use std::collections::BTreeMap;

/// RamBus wrapper that counts writes to the DSP sound-output registers.
struct CountBus {
    inner: RamBus,
    pub out_writes: u64,
}

impl CountBus {
    fn new(chip: Chip, data: Vec<u8>) -> Self {
        Self { inner: RamBus::new(chip, data), out_writes: 0 }
    }
}

impl JriscBus for CountBus {
    fn read_byte(&self, a: u32) -> u32 {
        self.inner.read_byte(a)
    }
    fn read_word(&self, a: u32) -> u32 {
        self.inner.read_word(a)
    }
    fn read_long(&self, a: u32) -> u32 {
        self.inner.read_long(a)
    }
    fn write_byte(&mut self, a: u32, v: u32) {
        self.inner.write_byte(a, v)
    }
    fn write_word(&mut self, a: u32, v: u32) {
        if (0xF1A148..0xF1A150).contains(&a) {
            self.out_writes += 1;
        }
        self.inner.write_word(a, v)
    }
    fn write_long(&mut self, a: u32, v: u32) {
        if (0xF1A148..0xF1A150).contains(&a) {
            self.out_writes += 1;
        }
        self.inner.write_long(a, v)
    }
}

#[test]
fn dsp_program_runs_busy_pipeline_when_signaled() {
    let dsp: &[u8] = include_bytes!("../tests/data/dsp.bin");
    // busy image: patch the sample-counter spin (jr eq at $F1B332) to nop
    let mut img = dsp.to_vec();
    img[0x332] = 0xE4;
    img[0x333] = 0x00;

    let mut cpu = Cpu::new(Chip::Dsp);
    cpu.pc = 0xF1B000;
    let mut bus = CountBus::new(Chip::Dsp, img);
    for _ in 0..500 {
        cpu.step(&mut bus).unwrap();
    }

    // the 68000-side signal
    bus.write_long(0xF1B35C, 0); // tick mailbox: zero -> F1B2CE mixing path
    bus.write_long(0xF1B358, 1); // command mailbox: bit0 gate set
    bus.write_long(0xF1B800, 0); // record[0]: r13 != -4 -> process
    bus.write_long(0xF1B80C, 0x100); // field +6 -> sample count
    bus.write_long(0xF1B820, 0xFFFF_FFFC); // record[1]: -4 terminator

    let mut counts: BTreeMap<u32, u64> = BTreeMap::new();
    for _ in 0..1_000_000 {
        let pc = cpu.pc;
        cpu.step(&mut bus).unwrap();
        *counts.entry(pc).or_insert(0) += 1;
    }

    // the pipeline must be doing real work: continuous stereo output
    // writes and a rich instruction mix spanning the whole driver
    assert!(
        bus.out_writes > 10_000,
        "expected sustained sample output, got {} writes",
        bus.out_writes
    );
    assert!(counts.len() > 50, "expected a rich pc mix, got {}", counts.len());
    // the four pipeline stages all execute
    let in_range = |lo: u32, hi: u32| counts.keys().any(|&p| p >= lo && p <= hi);
    assert!(in_range(0xF1B066, 0xF1B0B4), "record walk");
    assert!(in_range(0xF1B0B6, 0xF1B2BE), "command dispatch");
    assert!(in_range(0xF1B2C0, 0xF1B32E), "mixing");
    assert!(in_range(0xF1B330, 0xF1B356), "output");
}
