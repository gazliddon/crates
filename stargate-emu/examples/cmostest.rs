use std::path::Path;
use emucore::mem::MemoryIO;
use stargate_emu::bus::StargateBus;
fn main() {
    let mut bus = StargateBus::from_rom_dir(Path::new(&std::env::args().nth(1).unwrap())).unwrap();
    bus.store_byte(0xcca0, 0x5a).unwrap();
    println!("after store 0x5a: CCA0={:02x} CCA1={:02x}", bus.load_byte(0xcca0).unwrap(), bus.load_byte(0xcca1).unwrap());
}
