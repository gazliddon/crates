//! Temporary debug: disassemble the reset/IRQ handler paths of the ROM.
use std::path::Path;

use emu6809::diss::Diss;
use emucore::mem::MemoryIO;
use stargate_emu::bus::StargateBus;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rom_dir = std::env::args().nth(1).ok_or("usage: diss <rom-dir>")?;
    let mut bus = StargateBus::from_rom_dir(Path::new(&rom_dir))?;
    let diss = Diss::new();

    let dump = |bus: &mut StargateBus, start: usize, count: usize| {
        let mut pc = start;
        for _ in 0..count {
            let decoded = diss.diss(bus, pc);
            let bytes = decoded
                .decoded
                .data
                .iter()
                .map(|b| format!("{b:02X}"))
                .collect::<Vec<_>>()
                .join(" ");
            println!("{pc:04X}  {bytes:<11} {}", decoded.text);
            pc = decoded.decoded.next_addr & 0xffff;
            if pc <= start {
                break;
            }
        }
    };

    // enable the ROM bank (C900 bit 0) so low reads come from ROM
    bus.store_byte(0xc900, 1).unwrap();

    println!("== the CMOS write helper $1424:");
    dump(&mut bus, 0x1424, 12);

    // What does the entry do right after reset? Follow the first few branches
    // by dumping the bytes we saw referenced.
    println!("\n== bytes at $F400-$F4A0:");
    for i in (0..0xa0).step_by(16) {
        let a = 0xf400 + i;
        let bytes: Vec<String> = (0..16)
            .map(|j| format!("{:02X}", bus.inspect_byte(a + j).unwrap_or(0)))
            .collect();
        println!("{a:04X}  {}", bytes.join(" "));
    }
    Ok(())
}
