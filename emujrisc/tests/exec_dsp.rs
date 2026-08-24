//! Execution tests: instruction semantics (ported from MAME's jaguar core)
//! and a smoke run of the real Tempest 2000 DSP program.

use emujrisc::cpu::{Chip, Cpu};
use emujrisc::mem::{JriscBus, RamBus};

/// Build a CPU + internal RAM with the given little program (assembled by
/// hand from the validated JRISC encoding).
fn cpu_ram(chip: Chip, code: &[u16]) -> (Cpu, RamBus) {
    let mut bytes = Vec::new();
    for w in code {
        bytes.extend_from_slice(&w.to_be_bytes());
    }
    // pad to a full RAM window
    while bytes.len() < chip.ram_size() as usize {
        bytes.push(0);
    }
    let cpu = Cpu::new(chip);
    let bus = RamBus::new(chip, bytes);
    (cpu, bus)
}

fn step_n(cpu: &mut Cpu, bus: &mut dyn JriscBus, n: usize) {
    for _ in 0..n {
        cpu.step(bus).expect("step");
    }
}

// ---- helpers to encode instructions ----
fn word(op: u8, src: u8, dst: u8) -> u16 {
    ((op as u16) << 10) | ((src as u16) << 5) | (dst as u16)
}
fn movei(reg: u8, imm: u32) -> [u16; 3] {
    [word(38, 0, reg), (imm & 0xffff) as u16, (imm >> 16) as u16] // lo word first
}

#[test]
fn add_sets_flags_including_bit29_negative() {
    let mut code = vec![];
    code.extend_from_slice(&movei(1, 5));
    code.extend_from_slice(&movei(2, 3));
    code.push(word(0, 1, 2)); // add r1,r2
    code.extend_from_slice(&movei(3, 0x1000_0000));
    code.extend_from_slice(&movei(4, 0x1000_0000));
    code.push(word(0, 3, 4)); // add r3,r4 -> 0x40000000, bit29 set
    let (mut cpu, bus) = cpu_ram(Chip::Dsp, &code);
    let mut bus = bus;
    step_n(&mut cpu, &mut bus, 3);
    assert_eq!(cpu.r(2), 8);
    assert!(!cpu.flags.z && !cpu.flags.n && !cpu.flags.c);
    step_n(&mut cpu, &mut bus, 3);
    assert_eq!(cpu.r(4), 0x2000_0000);
    assert!(cpu.flags.n, "N flag is bit 29");
    assert!(!cpu.flags.z && !cpu.flags.c);
}

#[test]
fn addc_subc_use_carry_flag() {
    let mut code = vec![];
    code.extend_from_slice(&movei(1, 0xffff_ffff));
    code.extend_from_slice(&movei(2, 1));
    code.push(word(0, 1, 2)); // add -> 0, carry set
    code.push(word(1, 1, 2)); // addc r1,r2 -> 0 + 1 + C = 1... r2=0
    let (mut cpu, bus) = cpu_ram(Chip::Dsp, &code);
    let mut bus = bus;
    step_n(&mut cpu, &mut bus, 3);
    assert_eq!(cpu.r(2), 0);
    assert!(cpu.flags.c && cpu.flags.z);
    cpu.step(&mut bus).expect("addc");
    // r2 = 0 + 0xffffffff + 1 = 0; MAME computes the addc carry as
    // b(=r1+c wrapped to 0) > ~a(=~0) -> false (documenting the quirk)
    assert_eq!(cpu.r(2), 0);
    assert!(!cpu.flags.c, "MAME addc carry formula");
}

#[test]
fn shlq_amount_is_32_minus_raw_and_sets_carry_from_bit30() {
    // shlq #4, r2  (raw src = 28)
    let mut code = vec![];
    code.extend_from_slice(&movei(2, 0x4000_0000)); // bit 30
    code.push(word(24, 28, 2)); // shlq #4 -> 0x40000000 << 4 = 0, carry from bit30
    let (mut cpu, bus) = cpu_ram(Chip::Dsp, &code);
    let mut bus = bus;
    step_n(&mut cpu, &mut bus, 2);
    assert_eq!(cpu.r(2), 0);
    assert!(cpu.flags.c, "shlq carry comes from bit 30");
}

#[test]
fn jr_taken_executes_delay_slot_then_branches() {
    // 0: jr t, +1   (target = 0 + 2 + 2*1 = 4)
    // 2: movei #$11111111, r0   <- delay slot, executes
    // 4: movei #$22222222, r0   <- branch target
    let mut code = vec![];
    code.push(word(53, 3, 0)); // jr t, +3 words (land at base+8)
    code.extend_from_slice(&movei(0, 0x1111_1111)); // delay slot at +2
    code.extend_from_slice(&movei(0, 0x2222_2222)); // target at +8
    let (mut cpu, bus) = cpu_ram(Chip::Dsp, &code);
    let mut bus = bus;
    cpu.step(&mut bus).expect("jr");
    assert_eq!(cpu.r(0), 0x1111_1111, "delay slot executed");
    assert_eq!(cpu.pc, Chip::Dsp.ram_base() + 8, "branched to target");
    cpu.step(&mut bus).expect("movei");
    assert_eq!(cpu.r(0), 0x2222_2222);
}

#[test]
fn jr_not_taken_continues() {
    // 0: jr eq, +1  (eq needs Z=1; Z is 0 -> not taken)
    // 2: movei #$12345678, r0
    let mut code = vec![];
    code.push(word(53, 1, 2)); // jr eq, +1
    code.extend_from_slice(&movei(0, 0x1234_5678));
    let (mut cpu, bus) = cpu_ram(Chip::Dsp, &code);
    let mut bus = bus;
    cpu.step(&mut bus).expect("jr");
    assert_eq!(cpu.pc, Chip::Dsp.ram_base() + 2);
    cpu.step(&mut bus).expect("movei");
    assert_eq!(cpu.r(0), 0x1234_5678);
}

#[test]
fn jump_indirect_via_register() {
    // 0: movei #ram_base+10, r1
    // 6: jump t, (r1)     (delay slot: the nop at 8)
    // 8: nop
    // 10: movei #$deadbeef, r0 (branch target; the delay slot at 8 must not re-run)
    let mut code = vec![];
    code.extend_from_slice(&movei(1, Chip::Dsp.ram_base() + 10));
    code.push(word(52, 1, 0)); // jump t,(r1)
    code.push(word(57, 0, 0)); // nop (delay slot)
    code.extend_from_slice(&movei(0, 0xdead_beef));
    let (mut cpu, bus) = cpu_ram(Chip::Dsp, &code);
    let mut bus = bus;
    cpu.step(&mut bus).expect("movei");
    cpu.step(&mut bus).expect("jump");
    assert_eq!(cpu.pc, Chip::Dsp.ram_base() + 10);
    cpu.step(&mut bus).expect("movei");
    assert_eq!(cpu.r(0), 0xdead_beef);
}

#[test]
fn movepc_returns_own_address() {
    // 0: movepc r0
    let mut code = vec![];
    code.push(word(51, 0, 0));
    let (mut cpu, bus) = cpu_ram(Chip::Dsp, &code);
    let mut bus = bus;
    cpu.step(&mut bus).expect("movepc");
    assert_eq!(cpu.r(0), Chip::Dsp.ram_base());
}

#[test]
fn load_store_long_and_internal_ram_byte_quirk() {
    // r14 = internal base + 0x20; store r0,(r14); loadb (r14),r1 -> returns
    // the aligned LONG (the quirk), not the byte.
    let base = Chip::Dsp.ram_base() + 0x20;
    let mut code = vec![];
    code.extend_from_slice(&movei(14, base));
    code.extend_from_slice(&movei(0, 0x1234_5678));
    code.push(word(47, 14, 0)); // store r0,(r14)
    code.push(word(39, 14, 1)); // loadb (r14),r1
    let (mut cpu, bus) = cpu_ram(Chip::Dsp, &code);
    let mut bus = bus;
    step_n(&mut cpu, &mut bus, 3);
    // stored via store (long path, aligned)...
    cpu.step(&mut bus).expect("loadb");
    assert_eq!(cpu.r(1), 0x1234_5678, "loadb on internal RAM returns the aligned long");
}

#[test]
fn mult_imacn_resmac_chain() {
    // mult r0,r1 (16x16 unsigned); imacn r2,r3 (signed 16x16 accumulate x2);
    // resmac r4
    let mut code = vec![];
    code.extend_from_slice(&movei(0, 0x0002_0002)); // 2, 2
    code.extend_from_slice(&movei(1, 0x0003_0003)); // 3, 3
    code.extend_from_slice(&movei(2, 0xffff_0004)); // -1, 4
    code.extend_from_slice(&movei(3, 0x0005_0006)); // 5, 6
    code.push(word(16, 0, 1)); // mult -> (0x0002*0x0003) = 6
    code.push(word(20, 2, 3)); // imacn -> (-1*5) + (4*6) = 19
    code.push(word(20, 2, 3)); // imacn again -> 38
    code.push(word(19, 0, 4)); // resmac r4
    let (mut cpu, bus) = cpu_ram(Chip::Dsp, &code);
    let mut bus = bus;
    step_n(&mut cpu, &mut bus, 5);
    assert_eq!(cpu.r(1), 6);
    assert!(cpu.flags.z == false && cpu.flags.n == false);
    step_n(&mut cpu, &mut bus, 3);
    // imacn truncates the full register to i16 (low halves):
    // r2=0xffff0004 -> 4, r3=0x00050006 -> 6, so 2 * (4*6) = 48
    assert_eq!(cpu.r(4) as i32, 48, "accumulator = 2 * (4*6)");
}

#[test]
fn cmpq_sign_extends_immediate() {
    // cmpq #-1, r0  (src raw 31): r0=0 -> res = 0 - (-1) = 1 -> no Z, C set
    let mut code = vec![];
    code.extend_from_slice(&movei(0, 0));
    code.push(word(31, 31, 0)); // cmpq #-1, r0
    let (mut cpu, bus) = cpu_ram(Chip::Dsp, &code);
    let mut bus = bus;
    step_n(&mut cpu, &mut bus, 2);
    // 0 - (-1) = 1: no Z; borrow out -> C = r1 > r2 = 1
    assert!(!cpu.flags.z && cpu.flags.c);
}

#[test]
fn sat16s_clamps() {
    let mut code = vec![];
    code.extend_from_slice(&movei(0, 0x0001_0000)); // 65536
    code.push(word(33, 0, 0)); // sat16s r0 (DSP variant)
    code.extend_from_slice(&movei(1, 0xffff_8001)); // -32767
    code.push(word(33, 0, 1));
    let (mut cpu, bus) = cpu_ram(Chip::Dsp, &code);
    let mut bus = bus;
    step_n(&mut cpu, &mut bus, 2);
    assert_eq!(cpu.r(0), 0x7fff, "sat16s clamps high");
    step_n(&mut cpu, &mut bus, 2);
    assert_eq!(cpu.r(1), 0xffff_8001, "within range unchanged");
}

#[test]
fn dsp_program_smoke_run_reaches_init() {
    // Run the real T2K DSP program: the entry trampoline
    // (movei #$F1B022,r0; jump t,(r0)) lands at $F1B022 which sets the
    // stack pointer: movei #$F1B010,r31, then loads the TEST.SYM constants.
    let dsp: &[u8] = include_bytes!("data/dsp.bin");
    let mut cpu = Cpu::new(Chip::Dsp);
    cpu.pc = 0xF1B000;
    let mut bus = RamBus::new(Chip::Dsp, dsp.to_vec());
    step_n(&mut cpu, &mut bus, 10);
    assert_eq!(cpu.r(31), 0x00F1_B010, "stack pointer set by the real program");
    assert_eq!(cpu.r(9), 0x00F1_A100, "D_FLAGS address loaded");
    assert_eq!(cpu.r(27), 0x00F1_B800, "TABLESTA address loaded");
    // keep going through the channel setup without faults
    step_n(&mut cpu, &mut bus, 100);
    assert!(cpu.r(31) == 0x00F1_B010, "sp stays set");
}
