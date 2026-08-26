use emujrisc::cpu::{Chip, Cpu};
use emujrisc::mem::{JriscBus, RamBus};
fn word(op: u8, src: u8, dst: u8) -> u16 {
    ((op as u16) << 10) | ((src as u16) << 5) | (dst as u16)
}
fn movei(reg: u8, imm: u32) -> [u16; 3] {
    [word(38, 0, reg), (imm & 0xffff) as u16, (imm >> 16) as u16]
}
fn main() {
    let mut code: Vec<u16> = vec![];
    code.extend_from_slice(&movei(0, 0x0002_0003));
    code.push(word(54, 0, 1));
    let mut bytes = Vec::new();
    for w in &code {
        bytes.extend_from_slice(&w.to_be_bytes());
    }
    while bytes.len() < 4096 {
        bytes.push(0);
    }
    let mut cpu = Cpu::new(Chip::Gpu);
    let mut bus = RamBus::new(Chip::Gpu, bytes);
    bus.internal[2..4].copy_from_slice(&2i16.to_be_bytes());
    bus.internal[6..8].copy_from_slice(&3i16.to_be_bytes());
    cpu.mtx_width = 2;
    cpu.step(&mut bus).unwrap();
    println!(
        "r0 = {:08X} (bank {}), bank1 r0 = {:08X}",
        cpu.r(0),
        cpu.regs.bank(),
        cpu.regs.bank_get(1, 0)
    );
    println!("mem[2..4] = {:02X?}", &bus.internal[2..4]);
    println!("mtx_addr = {:08X}, width = {}", cpu.mtx_addr, cpu.mtx_width);
    cpu.step(&mut bus).unwrap();
    println!("r1 = {:08X}", cpu.r(1));
}
