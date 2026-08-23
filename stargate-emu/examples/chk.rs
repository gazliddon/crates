//! Temporary debug: run to the checksum-validation branch and print the
//! exact state the game sees (CMOS region, computed vs stored checksum).
use std::path::Path;

use emu6809::cpu::{Context, Pins, Regs};
use emucore::mem::MemoryIO;
use stargate_emu::bus::StargateBus;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rom_dir = std::env::args().nth(1).ok_or("usage: chk <rom-dir>")?;
    let mut mem = StargateBus::from_rom_dir(Path::new(&rom_dir))?;
    let mut regs = Regs::default();
    let mut pins = Pins::default();

    let target = 2_697_819usize; // just before the first E6F1 hit
    let (pc, a, b) = {
        let mut cpu = Context::new(&mut mem, &mut regs, &mut pins)?;
        cpu.reset()?;
        for _ in 0..target {
            cpu.step()?;
        }
        (cpu.get_pc(), regs.a, regs.b)
    };

    let byte = |addr: usize| mem.inspect_byte(addr).unwrap_or(0);
    let mut sum = 0u16;
    for addr in 0xcc36..0xcc9e {
        sum += (byte(addr) & 0x0f) as u16;
    }
    let checksum = (sum + 0x37) & 0xff;
    let stored = ((byte(0xcca0) << 4) & 0xff) | (byte(0xcca1) & 0x0f);

    println!("pc=${pc:04x} A=${a:02x} B=${b:02x}");
    println!("computed checksum: {checksum:02x} ({checksum})");
    println!("stored @CCA0/CCA1: {stored:02x}  (raw CCA0={:02x} CCA1={:02x})", byte(0xcca0), byte(0xcca1));
    println!("cmos CC36-CC9E low-nibble sum: {sum}");
    println!("cmos nonzero count (CC00-CFFF): {}", (0xcc00..0xd000).filter(|&addr| byte(addr) != 0).count());
    let mut nz = Vec::new();
    for addr in 0xcc00..0xd000 {
        if byte(addr) != 0 {
            nz.push(format!("{addr:04x}={:02x}", byte(addr)));
            if nz.len() >= 10 {
                break;
            }
        }
    }
    println!("first nonzero cmos: {nz:?}");
    Ok(())
}
