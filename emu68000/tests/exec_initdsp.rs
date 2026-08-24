//! Execute the real Imagitec sound-module code: INITDSP.
//!
//! INITDSP ($4D46) is the routine that boots the DSP: it copies the DSP
//! program (dsp.bin, embedded at $6B34) into the DSP's program RAM at
//! $F1B000, fills the channel table via six calls to $4DD4, programs the
//! DSP control registers (SCLK/SMODE/D_PC/D_CTRL) and returns. Running it
//! end to end exercises the bulk of the executor: movem predecrement,
//! long immediates, absolute-long addressing, postincrement copies, dbra
//! loops, jsr/rts, and the whole ALU flag path of the module's helpers.

use emu68000::cpu::{bus::Ram68k, Cpu, M68kBus};

const TEXT_BASE: u32 = 0x4000;

fn module() -> Vec<u8> {
    std::fs::read("tests/data/TEST.TXT").expect("TEST.TXT")
}

#[test]
fn initdsp_boots_the_dsp() {
    let dsp: &[u8] = include_bytes!("data/dsp.bin");
    let module = module();

    let mut bus = Ram68k::new();
    bus.map(TEXT_BASE, module); // module text/data ($4000-$70CF)
    bus.map(0x8000, vec![0u8; 0x800]); // BSS
    bus.map(0xF1A100, vec![0u8; 0x400]); // DSP register window
    bus.map(0xF1B000, vec![0u8; 0x1000]); // DSP program RAM

    let mut cpu = Cpu::new();
    // INITDSP is called from the module; fake a return address on the stack
    cpu.regs.pc = 0x4D46;
    cpu.regs.usp = 0x8800;
    cpu.regs.sr = 0; // user mode: A7 = USP
    cpu.push_long(&mut bus, 0x0000_1234); // return address

    // run INITDSP to completion (its own rts pops the fake address)
    for _ in 0..600 {
        match cpu.step(&mut bus) {
            Ok(()) => {
                if cpu.regs.pc == 0x0000_1234 {
                    break; // returned
                }
            }
            Err(e) => panic!("executor error at pc=${:06X}: {e}", cpu.regs.pc),
        }
    }
    assert_eq!(cpu.regs.pc, 0x0000_1234, "INITDSP must return via rts");

    // the DSP program was copied to $F1B000
    let mut copied = vec![0u8; dsp.len()];
    for (i, b) in copied.iter_mut().enumerate() {
        *b = bus.read_byte(0xF1B000 + i as u32);
    }
    assert_eq!(&copied, dsp, "DSP program copied verbatim");

    // the DSP control registers were programmed
    assert_eq!(bus.read_long(0xF1A114), 1, "D_CTRL = 1 (run)");
    assert_eq!(bus.read_long(0xF1A110), 0x00F1_B000, "D_PC = $F1B000");
    assert_eq!(bus.read_long(0xF1A150), 0x1B, "SCLK = $1B");
    assert_eq!(bus.read_long(0xF1A154), 0x15, "SMODE = $15");
    assert_eq!(bus.read_long(0xF1A100), 0, "D_FLAGS cleared");

    // the channel table at $F1B800 got its -4 terminators (six $4DD4
    // calls write (-4, $4000, 0) per entry — this is the DSP's TABLESTA)
    let mut terms = 0;
    let mut addr = 0xF1B800;
    while bus.read_long(addr) == 0xFFFF_FFFC && addr < 0xF1B900 {
        terms += 1;
        addr += 32;
    }
    assert_eq!(terms, 6, "expected six channel-table entries");

    // the register file was restored (movem.l (A7)+ at the end)
    assert_eq!(cpu.regs.sp(), 0x8800, "stack balanced");
}

#[test]
fn module_entry_boots_the_dsp() {
    // Run from the module's entry ($4000): INIT_SOUND -> INITDSP. The DSP
    // control registers and the program RAM must end up programmed even
    // though the module eventually runs off into uninitialized dispatch
    // (the real game provides that environment).
    let module = module();
    let mut bus = Ram68k::new();
    bus.map(TEXT_BASE, module);
    bus.map(0x8000, vec![0u8; 0x800]); // BSS (stack at $8800)
    bus.map(0xF00000, vec![0u8; 0x20000]); // TOM window
    bus.map(0xF10000, vec![0u8; 0x20000]); // JERRY window
    let mut cpu = Cpu::new();
    cpu.regs.pc = 0x4000;
    cpu.regs.usp = 0x8800;
    cpu.regs.sr = 0;
    let mut dsp_seen = false;
    for _ in 0..600 {
        if cpu.regs.pc == 0x4D46 {
            dsp_seen = true;
        }
        if cpu.step(&mut bus).is_err() {
            break;
        }
    }
    assert!(dsp_seen, "the entry must reach INITDSP");
    assert_eq!(bus.read_long(0xF1A114), 1, "D_CTRL = 1 (run)");
    assert_eq!(bus.read_long(0xF1A110), 0x00F1_B000, "D_PC = $F1B000");
    assert_eq!(bus.read_long(0xF1A150), 0x1B, "SCLK = $1B");
    assert_eq!(bus.read_long(0xF1A154), 0x15, "SMODE = $15");
    let dsp: &[u8] = include_bytes!("data/dsp.bin");
    let same = (0..dsp.len()).all(|i| bus.read_byte(0xF1B000 + i as u32) == dsp[i]);
    assert!(same, "DSP program copied verbatim");
}

#[test]
fn initdsp_cycle_count_is_sane() {
    let module = module();
    let mut bus = Ram68k::new();
    bus.map(TEXT_BASE, module);
    bus.map(0x8000, vec![0u8; 0x800]);
    bus.map(0xF1A100, vec![0u8; 0x400]);
    bus.map(0xF1B000, vec![0u8; 0x1000]);
    let mut cpu = Cpu::new();
    cpu.regs.pc = 0x4D46;
    cpu.regs.usp = 0x8800;
    cpu.regs.sr = 0;
    cpu.push_long(&mut bus, 0x1234);
    for _ in 0..600 {
        match cpu.step(&mut bus) {
            Ok(()) => {
                if cpu.regs.pc == 0x1234 {
                    break;
                }
            }
            Err(e) => panic!("executor error at pc=${:06X}: {e}", cpu.regs.pc),
        }
    }
    assert!(cpu.stats.instructions > 80, "INITDSP is ~90 instructions");
    // the real 68000 at 13.3 MHz would take ~10-20 us; base-cycle estimate
    assert!(cpu.stats.cycles > 400, "cycle count at least 400");
}
