use emu68000::cpu::{bus::Ram68k, Cpu, M68kBus};

#[test]
fn trace() {
    let module = std::fs::read("tests/data/TEST.TXT").unwrap();
    let mut bus = Ram68k::new();
    bus.map(0x4000, module);
    bus.map(0x8000, vec![0u8; 0x800]);
    bus.map(0xF1A100, vec![0u8; 0x400]);
    bus.map(0xF1B000, vec![0u8; 0x1000]);
    let mut cpu = Cpu::new();
    cpu.regs.pc = 0x4D46;
    cpu.regs.usp = 0x8800;
    cpu.regs.sr = 0;
    cpu.push_long(&mut bus, 0x1234);
    let mut printed = false;
    for i in 0..600 {
        let pc = cpu.regs.pc;
        if let Err(e) = cpu.step(&mut bus) {
            println!("step {i}: ERR at ${pc:06X}: {e}");
            break;
        }
        if cpu.regs.pc == 0x1234 {
            println!("returned at step {i}");
            break;
        }
        if pc >= 0x4D7C && !printed {
            printed = true;
        }
        if printed && pc <= 0x4DE0 {
            println!("{i:3}: ${pc:06X} -> ${:06X}", cpu.regs.pc);
        }
    }
    println!("final pc ${:06X}", cpu.regs.pc);
}
