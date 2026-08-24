use emujrisc::cpu::{Chip, Cpu};
use emujrisc::mem::{JriscBus, RamBus};
fn word(op: u8, src: u8, dst: u8) -> u16 { ((op as u16)<<10)|((src as u16)<<5)|(dst as u16) }
fn movei(reg: u8, imm: u32) -> [u16;3] { [word(38,0,reg), (imm&0xffff) as u16, (imm>>16) as u16] }
fn main() {
    let mut code: Vec<u16> = vec![];
    code.extend_from_slice(&movei(2, 0x1234_5678));
    code.push(word(63, 0, 2)); // pack
    let mut bytes = Vec::new();
    for w in &code { bytes.extend_from_slice(&w.to_be_bytes()); }
    while bytes.len() < 4096 { bytes.push(0); }
    let mut cpu = Cpu::new(Chip::Gpu);
    let mut bus = RamBus::new(Chip::Gpu, bytes);
    cpu.step(&mut bus).unwrap();
    println!("after movei: r2 = {:08X}", cpu.r(2));
    let d = cpu.step(&mut bus).unwrap();
    println!("after pack:  r2 = {:08X}", cpu.r(2));
    println!("pc = {:X}", cpu.pc);
}
