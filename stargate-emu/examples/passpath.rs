//! Temporary debug: run the machine with a preloaded nvram file (the pass
//! path) and trace where it goes vs MAME's pass-path trace.
use std::path::Path;

use emucore::mem::MemoryIO;
use stargate_emu::StargateMachine;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rom_dir = std::env::args()
        .nth(1)
        .ok_or("usage: passpath <rom-dir> <nvram-file> [count]")?;
    let nvram_file = std::env::args().nth(2).ok_or("nvram file")?;
    let count: u64 = std::env::args()
        .nth(3)
        .map(|v| v.parse())
        .transpose()?
        .unwrap_or(3_000_000);

    let mut machine = StargateMachine::from_rom_dir(Path::new(&rom_dir))?;
    machine.bus.load_nvram(&std::fs::read(&nvram_file)?);
    machine.reset()?;

    let mut out = String::new();
    let mut pcs = Vec::new();
    let mut irq_count: u64 = 0;
    while machine.instructions < count {
        machine.step().map_err(|e| {
            eprintln!(">>> error at pc=${:04x} instr={}", machine.regs.pc, machine.instructions);
            std::fs::write(
                "/tmp/our_ram.bin",
                (0x0000..0x0400)
                    .map(|a| machine.bus.inspect_byte(a).unwrap_or(0))
                    .collect::<Vec<_>>(),
            )
            .ok();
            e
        })?;
        if machine.pins.irq { irq_count += 1; }
        pcs.push(machine.regs.pc);
        if machine.regs.pc == 0x15FE { eprintln!(">>> INC $39 at instr={}", machine.instructions); }
        if machine.regs.pc == 0x9C6B && irq_count % 20 == 0 {
            eprintln!(">>> IRQ at instr={} beam={}", machine.instructions, machine.bus.video_scanline());
        }
        println!("{:04X}", machine.regs.pc);
    }
    print!("{out}");
    std::io::Write::flush(&mut std::io::stdout())?;

    // tail analysis
    let tail: Vec<u16> = pcs.iter().rev().take(100_000).copied().collect();
    let mut counts: Vec<(u16, usize)> = Vec::new();
    for pc in tail {
        if let Some(e) = counts.iter_mut().find(|(p, _)| *p == pc) {
            e.1 += 1;
        } else {
            counts.push((pc, 1));
        }
    }
    counts.sort_by(|a, b| b.1.cmp(&a.1));
    eprintln!("pc=${:04x} instructions={} irq_seen={irq_count}", machine.regs.pc, machine.instructions);
    eprintln!("top PCs in last 100k: {:?}", &counts[..8.min(counts.len())]);
    let nonzero: usize = (0x0000..0xc000)
        .filter(|&a| machine.bus.inspect_byte(a).unwrap_or(0) != 0)
        .count();
    eprintln!("nonzero VRAM: {nonzero}");
    std::fs::write(
        "/tmp/our_vram.bin",
        (0x0000..0xc000).map(|a| machine.bus.inspect_byte(a).unwrap_or(0)).collect::<Vec<_>>(),
    )?;
    std::fs::write(
        "/tmp/our_pal.bin",
        (0xc000..0xc010).map(|a| machine.bus.inspect_byte(a).unwrap_or(0)).collect::<Vec<_>>(),
    )?;
    for row in machine.video_ascii(64, 24) {
        println!("{row}");
    }
    Ok(())
}
