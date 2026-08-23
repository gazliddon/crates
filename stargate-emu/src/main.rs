use emu6809::cpu::{Context, Pins, Regs};
use std::collections::VecDeque;
use std::env;
use std::path::Path;
mod bus;
mod input;
pub use input::StargateInput;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rom_dir = env::args()
        .nth(1)
        .ok_or("usage: stargate-emu <rom-directory> [cycles]")?;
    let limit = env::args()
        .nth(2)
        .map(|v| v.parse())
        .transpose()?
        .unwrap_or(1_000_000usize);

    let mut mem = bus::StargateBus::from_rom_dir(Path::new(&rom_dir))?;
    let mut regs = Regs::default();
    let mut pins = Pins::default();
    let mut cpu = Context::new(&mut mem, &mut regs, &mut pins)?;
    cpu.reset()?;
    let mut recent = VecDeque::with_capacity(32);

    println!("reset pc=${:04x}", cpu.get_pc());
    for _ in 0..limit {
        let pc = cpu.get_pc();
        let opcode = cpu.mem.inspect_byte(pc).unwrap_or(0);
        recent.push_back((pc, opcode, cpu.cycles()));
        if recent.len() > 32 {
            recent.pop_front();
        }
        match cpu.step() {
            Ok(()) => {}
            Err(error) => {
                for (address, opcode, cycles) in &recent {
                    println!("  pc=${address:04x} op=${opcode:02x} cycle={cycles}");
                }
                println!(
                    "stopped after {} instructions / {} cycles at pc=${pc:04x}: {error}",
                    cpu.instructions(),
                    cpu.cycles()
                );
                return Err(error.into());
            }
        }
    }
    println!(
        "ran {} instructions / {} cycles, pc=${:04x}",
        cpu.instructions(),
        cpu.cycles(),
        cpu.get_pc()
    );
    Ok(())
}
