//! Execution tests for the **Tom GPU** variant (and the shared-opcode
//! numbers where GPU and DSP diverge: 32, 33, 42, 48, 63).
//!
//! The execution core is shared; these tests pin the GPU-specific behavior:
//! sat8/sat16/loadp/storep/pack/unpack, the bank-1 default, the `$F03000`
//! RAM base, and moveta/movefa cross-bank moves under REGPAGE.

use emujrisc::cpu::{Chip, Cpu};
use emujrisc::mem::{JriscBus, RamBus};

fn cpu_ram(chip: Chip, code: &[u16]) -> (Cpu, RamBus) {
    let mut bytes = Vec::new();
    for w in code {
        bytes.extend_from_slice(&w.to_be_bytes());
    }
    while bytes.len() < chip.ram_size() as usize {
        bytes.push(0);
    }
    (Cpu::new(chip), RamBus::new(chip, bytes))
}

fn step_n(cpu: &mut Cpu, bus: &mut dyn JriscBus, n: usize) {
    for _ in 0..n {
        cpu.step(bus).expect("step");
    }
}

fn word(op: u8, src: u8, dst: u8) -> u16 {
    ((op as u16) << 10) | ((src as u16) << 5) | (dst as u16)
}
fn movei(reg: u8, imm: u32) -> [u16; 3] {
    [word(38, 0, reg), (imm & 0xffff) as u16, (imm >> 16) as u16]
}

#[test]
fn gpu_ram_base_and_pc() {
    let mut cpu = Cpu::new(Chip::Gpu);
    assert_eq!(cpu.pc, 0xF03000);
    assert_eq!(cpu.regs.bank(), 1, "GPU code is assembled for bank 1 (GASM -R1)");
    let mut bus = RamBus::new(Chip::Gpu, vec![0; Chip::Gpu.ram_size() as usize]);
    cpu.step(&mut bus).expect("nop from zeroed ram");
}

#[test]
fn gpu_sat8_clamps_unsigned() {
    // op 32 as GPU = sat8 (DSP = subqmod)
    let mut code = vec![];
    code.extend_from_slice(&movei(2, 0x0000_0100)); // 256
    code.push(word(32, 0, 2));
    code.extend_from_slice(&movei(3, 0xffff_ffff)); // -1
    code.push(word(32, 0, 3));
    let (mut cpu, bus) = cpu_ram(Chip::Gpu, &code);
    let mut bus = bus;
    step_n(&mut cpu, &mut bus, 2);
    assert_eq!(cpu.r(2), 255, "sat8 clamps high");
    step_n(&mut cpu, &mut bus, 2);
    assert_eq!(cpu.r(3), 0, "sat8 clamps negatives to 0");
    assert!(cpu.flags.z);
}

#[test]
fn gpu_sat16_vs_dsp_sat16s_differ_on_negatives() {
    // 0xffff_8000: GPU sat16 -> 0 (unsigned clamp); DSP sat16s -> unchanged
    let mut code = vec![];
    code.extend_from_slice(&movei(2, 0xffff_8000));
    code.push(word(33, 0, 2)); // op 33
    let (mut cpu, bus) = cpu_ram(Chip::Gpu, &code);
    let mut bus = bus;
    step_n(&mut cpu, &mut bus, 2);
    assert_eq!(cpu.r(2), 0, "GPU sat16 treats negatives as 0");
    let (mut cpu, bus) = cpu_ram(Chip::Dsp, &code);
    let mut bus = bus;
    step_n(&mut cpu, &mut bus, 2);
    assert_eq!(cpu.r(2), 0xffff_8000, "DSP sat16s keeps -32768");
}

#[test]
fn gpu_loadp_stashes_hidata_then_loads_next_long() {
    // op 42 as GPU = loadp (DSP = sat32s)
    let mut ext = vec![0u8; 0x20];
    ext[0x00..0x04].copy_from_slice(&0xAABB_CCDDu32.to_be_bytes());
    ext[0x04..0x08].copy_from_slice(&0x1122_3344u32.to_be_bytes());
    let mut code = vec![];
    code.extend_from_slice(&movei(1, 0x1000));
    code.push(word(42, 1, 2)); // loadp (r1),r2
    let (mut cpu, bus) = cpu_ram(Chip::Gpu, &code);
    let mut bus = bus.with_external(0x1000, ext);
    step_n(&mut cpu, &mut bus, 2);
    assert_eq!(cpu.hidata, 0xAABB_CCDD, "loadp stashes the first long in hidata");
    assert_eq!(cpu.r(2), 0x1122_3344);
}

#[test]
fn dsp_sat32s_reads_the_accumulator() {
    // op 42 as DSP = sat32s: clamp from accum>>32
    let mut code = vec![];
    code.extend_from_slice(&movei(0, 0xffff_8000)); // i16 = -32768
    code.extend_from_slice(&movei(1, 0xffff_8000));
    for _ in 0..4 {
        code.push(word(20, 0, 1)); // imacn: 4 * (-32768*-32768) = 2^32
    }
    code.extend_from_slice(&movei(2, 0x1234_5678));
    code.push(word(42, 0, 2)); // sat32s r2
    let (mut cpu, bus) = cpu_ram(Chip::Dsp, &code);
    let mut bus = bus;
    step_n(&mut cpu, &mut bus, 8);
    assert_eq!(cpu.r(2), 0x7fff_ffff, "sat32s clamps when accum>>32 > 0");
}

#[test]
fn gpu_storep_writes_hidata_then_register() {
    // op 48 as GPU = storep (DSP = mirror)
    let mut ext = vec![0u8; 0x20];
    let mut code = vec![];
    code.extend_from_slice(&movei(1, 0x1000));
    code.extend_from_slice(&movei(2, 0x1234_5678));
    code.push(word(48, 1, 2)); // storep r2,(r1)
    let (mut cpu, bus) = cpu_ram(Chip::Gpu, &code);
    let mut bus = bus.with_external(0x1000, ext);
    cpu.hidata = 0xDEAD_BEEF;
    step_n(&mut cpu, &mut bus, 3);
    assert_eq!(bus.read_long(0x1000), 0xDEAD_BEEF, "hidata written first");
    assert_eq!(bus.read_long(0x1004), 0x1234_5678);
}

#[test]
fn dsp_mirror_bit_reverses() {
    let mut code = vec![];
    code.extend_from_slice(&movei(2, 0x0000_0001));
    code.push(word(48, 0, 2)); // op 48 as DSP = mirror r2
    let (mut cpu, bus) = cpu_ram(Chip::Dsp, &code);
    let mut bus = bus;
    step_n(&mut cpu, &mut bus, 2);
    assert_eq!(cpu.r(2), 0x8000_0000, "mirror reverses all 32 bits");
}

#[test]
fn gpu_pack_then_unpack_roundtrip() {
    // op 63 as GPU: src=0 -> pack; src!=0 -> unpack
    let mut code = vec![];
    code.extend_from_slice(&movei(2, 0x1234_5678));
    code.push(word(63, 0, 2)); // pack
    code.push(word(63, 1, 2)); // unpack
    let (mut cpu, bus) = cpu_ram(Chip::Gpu, &code);
    let mut bus = bus;
    step_n(&mut cpu, &mut bus, 2);
    // packed: ((r>>10)&0xf000)|((r>>5)&0x0f00)|(r&0xff)
    // r = 0x12345678: r>>10 = 0x48D15 -> 0x8000; r>>5 = 0x91A2B3 -> 0x0200; |0x78
    assert_eq!(cpu.r(2), 0x0000_8278, "pack");
    step_n(&mut cpu, &mut bus, 1);
    assert_eq!(cpu.r(2), 0x0200_4078, "unpack");
}

#[test]
fn dsp_addqmod_wraps_with_modulo_mask() {
    // op 63 as DSP = addqmod: (r2 + n) & ~modulo | r2 & modulo
    let mut code = vec![];
    code.extend_from_slice(&movei(2, 0x0000_00ff));
    code.push(word(63, 1, 2)); // addqmod #1, r2
    let (mut cpu, bus) = cpu_ram(Chip::Dsp, &code);
    let mut bus = bus;
    cpu.modulo = 0x0000_00ff; // low byte masked
    step_n(&mut cpu, &mut bus, 2);
    // MAME: res = (sum & ~modulo) | (r2 & modulo) = (0x100 & ~0xff) | (0xff & 0xff)
    assert_eq!(cpu.r(2), 0x0000_01ff, "addqmod preserves the masked bits of r2");
}

#[test]
fn gpu_moveta_movefa_cross_bank_under_default_bank1() {
    // GPU default bank = 1: moveta writes bank 0, movefa reads bank 0 back
    let mut code = vec![];
    code.extend_from_slice(&movei(0, 0x1234_5678));
    code.push(word(36, 0, 5)); // moveta r0 -> bank0 r5
    code.push(word(37, 5, 6)); // movefa bank0 r5 -> bank1 r6
    let (mut cpu, bus) = cpu_ram(Chip::Gpu, &code);
    let mut bus = bus;
    step_n(&mut cpu, &mut bus, 3);
    assert_eq!(cpu.r(6), 0x1234_5678, "round trip through the other bank");
    assert_eq!(cpu.regs.alt_get(5), 0x1234_5678, "bank 0 still holds the value");
    // with REGPAGE set, the active bank flips to 0
    let (mut cpu, bus) = cpu_ram(Chip::Gpu, &code);
    let mut bus = bus;
    cpu.flags.regpage = true;
    step_n(&mut cpu, &mut bus, 3);
    assert_eq!(cpu.r(6), 0x1234_5678, "works identically from bank 0");
    assert_eq!(cpu.regs.alt_get(5), 0x1234_5678);
}

#[test]
fn gpu_mmult_reads_bank1_operands() {
    // mmult: 16-bit pairs from bank 1 registers times memory words at mtx_addr
    let mut code = vec![];
    code.extend_from_slice(&movei(0, 0x0003_0002)); // bank1 r0: low=2, high=3
    code.push(word(54, 0, 1)); // mmult r0 -> r1
    let (mut cpu, bus) = cpu_ram(Chip::Gpu, &code);
    let mut bus = bus;
    // matrix words live in INTERNAL RAM (MAME: ram_start | mtxa & 0xffc);
    // mmult reads at addr+2, +4 per step — keep clear of the program bytes
    cpu.mtx_addr = Chip::Gpu.ram_base() + 0x20;
    bus.internal[0x22..0x24].copy_from_slice(&2i16.to_be_bytes());
    bus.internal[0x26..0x28].copy_from_slice(&3i16.to_be_bytes());
    cpu.mtx_width = 2;
    step_n(&mut cpu, &mut bus, 2);
    // accum = 2*2 + 3*3 = 13
    assert_eq!(cpu.r(1), 13, "mmult accumulates 16x16 products");
}
