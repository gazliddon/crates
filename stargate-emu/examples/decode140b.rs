// decode ROM bytes 1400-1425 with the ROM bank enabled
use std::path::Path;
use emu6809::diss::Diss;
use emucore::mem::MemoryIO;
use stargate_emu::bus::StargateBus;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dir = std::env::args().nth(1).unwrap();
    let mut bus = StargateBus::from_rom_dir(Path::new(&dir))?;
    bus.store_byte(0xc900, 1).unwrap(); // ROM bank on
    let diss = Diss::new();
    let mut pc = 0x1400usize;
    for _ in 0..12 {
        let d = diss.diss(&mut bus, pc);
        let b: Vec<String> = d.decoded.data.iter().map(|x| format!("{x:02X}")).collect();
        println!("{pc:04X}  {:<12} {}", b.join(" "), d.text);
        pc = d.decoded.next_addr & 0xffff;
    }
    Ok(())
}
