//! Temporary debug: run the machine, pulse the Advance (service) switch
//! after the CMOS re-init so the game leaves the error-ack poll, and report
//! where it ends up (does it reach attract mode?).
use std::path::Path;

use emucore::mem::MemoryIO;
use stargate_emu::{StargateInput, StargateMachine};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rom_dir = std::env::args()
        .nth(1)
        .ok_or("usage: advance <rom-dir> [count] [press-at]")?;
    let count: u64 = std::env::args()
        .nth(2)
        .map(|v| v.parse())
        .transpose()?
        .unwrap_or(10_000_000);
    let press_at: u64 = std::env::args()
        .nth(3)
        .map(|v| v.parse())
        .transpose()?
        .unwrap_or(2_800_000);

    let mut machine = StargateMachine::from_rom_dir(Path::new(&rom_dir))?;
    machine.reset()?;
    let mut advanced = false;
    let mut released = false;
    let mut last_pcs = Vec::new();
    while machine.instructions < count {
        if !advanced && machine.instructions >= press_at {
            machine
                .bus
                .set_input(StargateInput { advance: true, ..Default::default() });
            advanced = true;
            eprintln!(">>> pressed Advance at {} instr (pc=${:04x})", machine.instructions, machine.regs.pc);
        }
        if advanced && !released && machine.instructions >= press_at + 20_000 {
            machine.bus.set_input(StargateInput::default());
            released = true;
            eprintln!(">>> released Advance at {} instr", machine.instructions);
        }
        machine.step().map_err(|e| {
            std::fs::write(
                "/tmp/our_ram.bin",
                (0x0000..0x0400)
                    .map(|a| machine.bus.inspect_byte(a).unwrap_or(0))
                    .collect::<Vec<_>>(),
            )
            .ok();
            eprintln!(">>> step error at pc=${:04x} instr={}", machine.regs.pc, machine.instructions);
            e
        })?;
        last_pcs.push(machine.regs.pc);
    }

    // Where did we end up? Look for a tight loop in the final PCs.
    let tail: Vec<u16> = last_pcs.iter().rev().take(200_000).copied().collect();
    let mut counts: Vec<(u16, usize)> = Vec::new();
    for pc in tail {
        if let Some(e) = counts.iter_mut().find(|(p, _)| *p == pc) {
            e.1 += 1;
        } else {
            counts.push((pc, 1));
        }
    }
    counts.sort_by(|a, b| b.1.cmp(&a.1));
    eprintln!("pc=${:04x} instructions={}", machine.regs.pc, machine.instructions);
    eprintln!("top PCs in last 200k instr: {:?}", &counts[..8.min(counts.len())]);

    // Frame stats
    let nonzero_vram: usize = (0x0000..0xc000)
        .filter(|&a| machine.bus.inspect_byte(a).unwrap_or(0) != 0)
        .count();
    eprintln!("nonzero VRAM bytes: {nonzero_vram}");
    std::fs::write(
        "/tmp/our_ram.bin",
        (0x0000..0x0400).map(|a| machine.bus.inspect_byte(a).unwrap_or(0)).collect::<Vec<_>>(),
    )?;
    std::fs::write(
        "/tmp/our_vram.bin",
        (0x0000..0xc000).map(|a| machine.bus.inspect_byte(a).unwrap_or(0)).collect::<Vec<_>>(),
    )?;
    std::fs::write(
        "/tmp/our_pal.bin",
        (0xc000..0xc010).map(|a| machine.bus.inspect_byte(a).unwrap_or(0)).collect::<Vec<_>>(),
    )?;
    let ascii = machine.video_ascii(64, 24);
    for row in ascii {
        println!("{row}");
    }
    Ok(())
}
