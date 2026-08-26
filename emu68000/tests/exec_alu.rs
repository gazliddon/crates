//! Executor unit tests: hand-computed instruction semantics.
//! Words are big-endian bytes loaded at $1000; `run` steps them.

use emu68000::cpu::{bus::Ram68k, Cpu, M68kBus};

fn run_prog(words: &[u16], n: usize) -> (Cpu, Ram68k) {
    let mut bytes = Vec::new();
    for w in words {
        bytes.extend_from_slice(&w.to_be_bytes());
    }
    let mut bus = Ram68k::new();
    bus.map(0x1000, bytes);
    // one region covering scratch and the downward-growing stack
    bus.map(0x2000, vec![0u8; 0x1200]); // $2000..$31FF
    let mut cpu = Cpu::new();
    cpu.regs.pc = 0x1000;
    cpu.regs.usp = 0x3000;
    cpu.regs.sr = 0;
    for _ in 0..n {
        cpu.step(&mut bus).expect("step");
    }
    (cpu, bus)
}

#[test]
fn move_and_alu_flags() {
    let (cpu, _) = run_prog(
        &[
            0x203C, 0x0000, 0x1234, // move.l #$1234,D0
            0x2240, // movea.l D0,A1
            0x7401, // moveq #1,D2
            0xD449, // add.w A1,D2
        ],
        4,
    );
    assert_eq!(cpu.regs.d[0], 0x0000_1234);
    assert_eq!(cpu.regs.a[1], 0x0000_1234);
    // D2 = 0x1234 + 1 (word op keeps upper bits... D2 was 1, so 0x1235)
    assert_eq!(cpu.regs.d[2] & 0xFFFF, 0x1235);
    assert!(!cpu.regs.n());
    assert!(!cpu.regs.z());
}

#[test]
fn add_carry_and_overflow() {
    // addq.w #8,D0 from 0 -> 8
    let (cpu3, _) = run_prog(&[0x7000, 0x5040, 0x0000], 2);
    assert_eq!(cpu3.regs.d[0], 8);
    // move.w #$FFF8,D0 ; addq.w #8,D0 -> 0 with carry + zero
    let (cpu4, _) = run_prog(&[0x303C, 0xFFF8, 0x5040, 0x0000], 2);
    assert_eq!(cpu4.regs.d[0] & 0xFFFF, 0x0000);
    assert!(cpu4.regs.c());
    assert!(cpu4.regs.z());
    // move.w #$7FFF,D0 ; addq.w #1,D0 -> $8000 with overflow + N
    let (cpu5, _) = run_prog(&[0x303C, 0x7FFF, 0x5240, 0x0000], 2);
    assert_eq!(cpu5.regs.d[0] & 0xFFFF, 0x8000);
    assert!(cpu5.regs.v());
    assert!(cpu5.regs.n());
    // byte op keeps upper bits: move.l #$12345678,D0 ; addq.b #1,D0
    let (cpu6, _) = run_prog(&[0x203C, 0x1234, 0x5678, 0x5200, 0x0000], 2);
    assert_eq!(cpu6.regs.d[0], 0x1234_5679);
}

#[test]
fn branches_and_bsr() {
    // beq taken when Z set: moveq #0,D0 ; tst.l D0 ; beq +4
    let (cpu, _) = run_prog(&[0x7000, 0x4A80, 0x6702, 0x7001, 0x7002, 0x0000], 4);
    assert_eq!(cpu.regs.d[0], 2, "beq taken past the moveq #1");
    // bsr pushes return and jumps: bsr.w +2 -> lands at nop; rts returns
    let (cpu2, bus2) = run_prog(&[0x6100, 0x0004, 0x4E71, 0x4E75, 0x0000], 2);
    assert_eq!(
        cpu2.regs.pc, 0x1004,
        "after bsr to the rts, rts back to nop"
    );
    let _ = bus2;
}

#[test]
fn dbra_loop() {
    // D0 = 3; loop: dbra D0, loop  (3,2,1,0 then fall through)
    let (cpu, _) = run_prog(&[0x7003, 0x51C8, 0xFFFE, 0x5240, 0x0000], 6);
    // after dbra falls through: D0 = 0xFFFF (word), addq.w #1 -> 0
    assert_eq!(cpu.regs.d[0] & 0xFFFF, 0);
}

#[test]
fn movem_push_pop() {
    // movem.l D0-D3/A0-A1,-(A7) ; movem.l (A7)+,D0-D3/A0-A1
    let (mut cpu, bus) = run_prog(
        &[
            0x203C, 0x1111, 0x2222, // move.l #$11112222,D0
            0x48E7, 0xF0C0, // movem.l D0-D3/A0-A1,-(A7) (pd mask: D0=bit15..A1=bit6)
            0x7000, // clobber D0
            0x4CDF, 0x030F, // movem.l (A7)+,D0-D3/A0-A1 (mask 0x030F)
        ],
        6,
    );
    assert_eq!(cpu.regs.d[0], 0x1111_2222, "restored");
    assert_eq!(cpu.regs.sp(), 0x3000, "stack balanced");
    let _ = bus;
}

#[test]
fn shifts_and_extends() {
    // moveq #1,D0 ; lsl.w #8,D0 ; ext.l D0
    let (cpu, _) = run_prog(&[0x7001, 0xE148, 0x48C0, 0x0000], 3);
    assert_eq!(cpu.regs.d[0], 0x0000_0100);
    let (cpu2, _) = run_prog(&[0x303C, 0xFFFF, 0x48C0, 0x0000], 2);
    assert_eq!(cpu2.regs.d[0], 0xFFFF_FFFF, "ext.l sign-extends");
    let (cpu3, _) = run_prog(&[0x7001, 0xE208, 0x0000], 2); // lsr.b #1,D0
    assert_eq!(cpu3.regs.d[0] & 0xFF, 0);
    assert!(cpu3.regs.c(), "lsr carry from bit 0");
}

#[test]
fn mulu_divu() {
    // mulu.w #$0100,D0 with D0 = $0003 -> $300
    let (cpu, _) = run_prog(&[0x7003, 0xC0FC, 0x0100, 0x0000], 2);
    assert_eq!(cpu.regs.d[0], 0x300);
    // divu.w #4,D0 with D0 = $000A -> quotient 2 in low, remainder 2 in high
    let (cpu2, _) = run_prog(&[0x700A, 0x80FC, 0x0004, 0x0000], 2);
    assert_eq!(cpu2.regs.d[0], (2 << 16) | 2);
}

#[test]
fn link_unlk() {
    // link A6,#-8 ; unlk A6
    let (mut cpu, mut bus) = run_prog(&[0x4E56, 0xFFF8, 0x4E5E, 0x0000], 1);
    assert_eq!(cpu.regs.a[6], 0x2FFC, "A6 = SP after the push");
    assert_eq!(cpu.regs.sp(), 0x2FF4, "frame allocated (sp - 8)");
    cpu.step(&mut bus).expect("unlk");
    assert_eq!(cpu.regs.sp(), 0x3000, "frame freed");
    assert_eq!(cpu.regs.a[6], 0, "A6 restored");
}

#[test]
fn jsr_rts_and_stack_ops() {
    // jsr (A0) where A0 = $1010 (a rts); then move.l D1,D0 at $1010
    let (mut cpu, mut bus) = run_prog(
        &[
            0x227C, 0x0000, 0x100A, // movea.l #$100A,A1 (the rts)
            0x4E91, // jsr (A1) at $1006 -> pushes $1008
            0x7001, // $1008: moveq #1,D0 (skipped while in the jsr)
            0x4E75, // $100A: rts
            0x0000, 0x0000, // pad
            0x0000,
        ],
        2,
    );
    assert_eq!(cpu.regs.pc, 0x100A, "jsr landed on the rts");
    cpu.step(&mut bus).expect("rts");
    assert_eq!(cpu.regs.pc, 0x1008, "rts returned to the caller");
    cpu.step(&mut bus).expect("moveq");
    assert_eq!(cpu.regs.d[0], 1, "moveq #1 executed");
}

#[test]
fn byte_a7_steps_by_two() {
    // move.b D0,-(A7) then move.b (A7)+,D1: A7 changes by 2 for bytes
    let (mut cpu, _) = run_prog(&[0x7001, 0x1F00, 0x121F, 0x0000], 3);
    assert_eq!(cpu.regs.d[1], 1);
    assert_eq!(cpu.regs.sp(), 0x3000);
}

#[test]
fn pc_relative_lea() {
    // lea.l $10(PC),A0 at $1000: target = $1000 + 4 + $10 = $1014
    let (cpu, _) = run_prog(&[0x41FA, 0x0010, 0x0000], 1);
    assert_eq!(cpu.regs.a[0], 0x1014);
}
