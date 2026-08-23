//! Temporary debug: dump the PC stream like MAME's tracer for differential
//! comparison. Prints one PC per executed instruction.
use std::path::Path;

use emu6809::cpu::{Context, Pins, Regs};
use stargate_emu::bus::StargateBus;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rom_dir = std::env::args().nth(1).ok_or("usage: trace <rom-dir> [count]")?;
    let count = std::env::args()
        .nth(2)
        .map(|v| v.parse())
        .transpose()?
        .unwrap_or(552_506usize);

    let mut mem = StargateBus::from_rom_dir(Path::new(&rom_dir))?;
    let mut regs = Regs::default();
    let mut pins = Pins::default();
    let mut cpu = Context::new(&mut mem, &mut regs, &mut pins)?;
    cpu.reset()?;

    let mut out = String::with_capacity(count * 5);
    for _ in 0..count {
        out.push_str(&format!("{:04X}\n", cpu.get_pc()));
        if out.len() > 1 << 16 {
            print!("{out}");
            std::io::Write::flush(&mut std::io::stdout())?;
            out.clear();
        }
        cpu.step()?;
    }
    print!("{out}");
    std::io::Write::flush(&mut std::io::stdout())?;
    eprintln!("traced {count} instructions, pc=${:04x}", cpu.get_pc());
    Ok(())
}
