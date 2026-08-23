//! Temporary debug: run the machine for a target cycle count, dump
//! video RAM / palette / CMOS for comparison against MAME.
use std::path::Path;

use emucore::mem::MemoryIO;
use stargate_emu::StargateMachine;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rom_dir = std::env::args().nth(1).ok_or("usage: dump <rom-dir> [cycles]")?;
    let target = std::env::args()
        .nth(2)
        .map(|v| v.parse())
        .transpose()?
        .unwrap_or(60_000_000u64);

    let mut machine = StargateMachine::from_rom_dir(Path::new(&rom_dir))?;
    machine.reset()?;
    while machine.cycles < target {
        machine.step()?;
    }

    let read = |start: usize, len: usize| -> Vec<u8> {
        (start..start + len)
            .map(|a| machine.bus.inspect_byte(a).unwrap_or(0))
            .collect()
    };

    std::fs::write("/tmp/our_vram.bin", read(0x0000, 0xc000))?;
    std::fs::write("/tmp/our_pal.bin", read(0xc000, 0x10))?;
    std::fs::write("/tmp/our_cmos.bin", read(0xcc00, 0x0400))?;
    eprintln!(
        "dumped at {} cycles / {} instructions, pc=${:04x}",
        machine.cycles, machine.instructions, machine.regs.pc
    );
    Ok(())
}
