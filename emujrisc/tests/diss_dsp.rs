//! Validation of the JRISC decoder/disassembler against the real Tempest
//! 2000 DSP program.
//!
//! `data/dsp.bin` is the 868-byte Jerry DSP program extracted from
//! `MOOMOO.DAT` and cross-validated byte-identical against `TEST.TXT`'s
//! `GPUSTART` blob ($6B34, module base $4000) — two independent builds
//! agree, so this is ground truth.

use emujrisc::cpu::{decode, Chip};
use emujrisc::diss::{Diss, DissCtx};
use emujrisc::mem::SliceMem;

const DSP: &[u8] = include_bytes!("data/dsp.bin");
const DSP_BASE: usize = 0xF1B000;

fn ctx() -> DissCtx {
    DissCtx::from_slice(DSP_BASE, "dsp", DSP)
}

#[test]
fn decodes_the_dsp_program_entry_sequence() {
    let diss = Diss::new(Chip::Dsp);
    let ctx = ctx();
    let cases = [
        (0xF1B000, "movei #$00F1B022,r0", 6),
        (0xF1B006, "jump t,(r0)", 2),
        (0xF1B008, "nop", 2),
        (0xF1B010, "load (r9),r29", 2),
        (0xF1B012, "bclr #3,r29", 2),
        (0xF1B01A, "addq #4,r31", 2),
        (0xF1B01C, "moveq #1,r0", 2),
        (0xF1B01E, "jump t,(r30)", 2),
        (0xF1B020, "store r29,(r9)", 2),
        (0xF1B022, "movei #$00F1B010,r31", 6),
        (0xF1B02A, "movei #$00F1A100,r9", 6),
        (0xF1B030, "movei #$00F1B800,r27", 6),
        (0xF1B042, "movei #$000000FF,r11", 6),
        (0xF1B050, "movei #$FFFFFFFC,r12", 6),
        (0xF1B056, "movei #$00F1B358,r28", 6),
    ];
    for (addr, expected, size) in cases {
        let d = ctx.diss(&diss, addr).unwrap_or_else(|e| panic!("decode @${addr:X}: {e}"));
        assert_eq!(d.text, expected, "@${addr:X}");
        assert_eq!(d.size, size, "@${addr:X} size");
    }
}

#[test]
fn whole_program_walk_counts_match_python_decoder() {
    // The Python prototype (tools/jrisc_disasm.py) walked the same bytes to
    // 382 instructions / 20 NOPs; the Rust walk must agree.
    let diss = Diss::new(Chip::Dsp);
    let ctx = ctx();
    let mut addr = DSP_BASE;
    let mut count = 0usize;
    let mut nops = 0usize;
    while addr < DSP_BASE + DSP.len() {
        let d = ctx.diss(&diss, addr).expect("decode");
        if d.text == "nop" {
            nops += 1;
        }
        addr += d.size;
        count += 1;
    }
    assert_eq!(count, 382, "instruction count");
    assert_eq!(nops, 20, "nop count");
}

#[test]
fn shared_opcode_numbers_disambiguate_by_chip() {
    let ctx = ctx();
    // 0x8419 / 0x841A: opcode 33 — GPU sat16 vs DSP sat16s (dst 25/26)
    for (addr, reg) in [(0xF1B344, 25u8), (0xF1B346, 26u8)] {
        let mut view = SliceMem::new(ctx.base, &ctx.data);
        let dsp = decode(&mut view, addr, Chip::Dsp).expect("dsp decode");
        assert_eq!(dsp.insn.mnemonic, "sat16s", "@{addr:X} as DSP");
        assert_eq!(dsp.dst, reg);
        let mut view = SliceMem::new(ctx.base, &ctx.data);
        let gpu = decode(&mut view, addr, Chip::Gpu).expect("gpu decode");
        assert_eq!(gpu.insn.mnemonic, "sat16", "@{addr:X} as GPU");
    }
}

#[test]
fn illegal_opcode_for_chip_reports_unknown() {
    // op 62 = sat24 is GPU-only; decoding as DSP must yield the unknown insn
    let mut buf = Vec::new();
    buf.extend_from_slice(&0xF800u16.to_be_bytes()); // sat24 r0 as GPU
    let ctx = DissCtx::from_slice(0, "crafted", &buf);
    let diss = Diss::new(Chip::Dsp);
    let dsp = ctx.diss(&diss, 0).expect("decode as dsp");
    assert_eq!(dsp.decoded.insn.mnemonic, "??");
    let diss = Diss::new(Chip::Gpu);
    let gpu = ctx.diss(&diss, 0).expect("decode as gpu");
    assert_eq!(gpu.text, "sat24 r0");
}

#[test]
fn movei_immediate_is_word_swapped() {
    let mut buf = Vec::new();
    // movei #$00F1A114,r1 -> word 0x9801, extra words lo-first: 0xA114, 0x00F1
    buf.extend_from_slice(&0x9801u16.to_be_bytes());
    buf.extend_from_slice(&0xA114u16.to_be_bytes());
    buf.extend_from_slice(&0x00F1u16.to_be_bytes());
    let ctx = DissCtx::from_slice(0, "crafted", &buf);
    let diss = Diss::new(Chip::Dsp);
    let d = ctx.diss(&diss, 0).expect("decode");
    assert_eq!(d.text, "movei #$00F1A114,r1");
    assert_eq!(d.size, 6);
}

#[test]
fn jr_target_is_pc_relative_in_words() {
    let diss = Diss::new(Chip::Dsp);
    // jr t, +0 (offset 0): word = (53<<10)|(0<<5)|0 = 0xD400; target = pc+2
    let mut buf = Vec::new();
    buf.extend_from_slice(&0xD400u16.to_be_bytes());
    let ctx = DissCtx::from_slice(0x1000, "crafted", &buf);
    let d = ctx.diss(&diss, 0x1000).expect("decode");
    assert_eq!(d.text, "jr t,$1002");
    // jr mi, +31 words (+62 bytes): word = (53<<10)|(31<<5)|24 = 0xD7F8;
    // target = 0x2000 + 2 + 62 = 0x203E
    let mut buf2 = Vec::new();
    buf2.extend_from_slice(&0xD7F8u16.to_be_bytes());
    let ctx2 = DissCtx::from_slice(0x2000, "crafted", &buf2);
    let d2 = ctx2.diss(&diss, 0x2000).expect("decode");
    assert_eq!(d2.text, "jr mi,$2040");
}

#[test]
fn register_bank_defaults_follow_chip() {
    use emujrisc::cpu::Cpu;
    let gpu = Cpu::new(Chip::Gpu);
    assert_eq!(gpu.regs.bank(), 1);
    assert_eq!(gpu.pc, 0xF03000);
    let dsp = Cpu::new(Chip::Dsp);
    assert_eq!(dsp.regs.bank(), 0);
    assert_eq!(dsp.pc, 0xF1B000);
}

#[test]
fn metadata_fields_are_loaded() {
    use emujrisc::isa::{Dbase, InsnClass, MemAccess, Reloc, Variant};
    let db = Dbase::get();
    let load = db.lookup(41, Variant::Dsp).unwrap();
    assert_eq!(load.class, InsnClass::Memory);
    assert_eq!(load.mem, MemAccess::Read32);
    assert_eq!(load.cycles, 1);
    assert_eq!(load.size, 2);
    let movei = db.lookup(38, Variant::Dsp).unwrap();
    assert_eq!(movei.size, 6);
    assert_eq!(movei.reloc, Reloc::Abs32Swap);
    let jr = db.lookup(53, Variant::Dsp).unwrap();
    assert_eq!(jr.reloc, Reloc::Pcrel5);
    assert!(jr.pad, "jr needs NOP padding after it");
    let imacn = db.lookup(20, Variant::Dsp).unwrap();
    assert!(imacn.mac, "imacn touches the MAC");
    assert_eq!(imacn.class, InsnClass::Mac);
    let add = db.lookup(0, Variant::Dsp).unwrap();
    assert!(add.flags.carry && add.flags.zero && add.flags.neg, "add sets zcn");
    let sat16s = db.lookup(33, Variant::Dsp).unwrap();
    assert!(sat16s.flags.zero && sat16s.flags.neg && !sat16s.flags.carry, "sat16s sets zn");
    let btst = db.lookup(13, Variant::Dsp).unwrap();
    assert!(btst.flags.zero && !btst.flags.carry && !btst.flags.neg, "btst sets z");
    let sat16 = db.lookup(33, Variant::Gpu).unwrap();
    assert_eq!(sat16.mnemonic, "sat16", "shared opcode resolves per chip");
    // JSON-only fields are ignored by the crate but must not break parsing:
    assert_eq!(db.unknown.mnemonic, "??");
}
