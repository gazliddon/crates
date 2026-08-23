//! Temporary debug: log every store to $CC00-$CFFF during the boot.
use std::path::Path;
use emu6809::cpu::{Context, Pins, Regs};
use emucore::mem::MemoryIO;
use stargate_emu::bus::StargateBus;

struct LogBus(StargateBus);
impl std::ops::Deref for LogBus {
    type Target = StargateBus;
    fn deref(&self) -> &Self::Target { &self.0 }
}
impl std::ops::DerefMut for LogBus {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}
impl MemoryIO for LogBus {
    fn inspect_byte(&self, a: usize) -> emucore::mem::MemResult<u8> { self.0.inspect_byte(a) }
    fn inspect_word(&self, a: usize) -> emucore::mem::MemResult<u16> { self.0.inspect_word(a) }
    fn upload(&mut self, a: usize, d: &[u8]) -> emucore::mem::MemResult<()> { self.0.upload(a, d) }
    fn get_range(&self) -> std::ops::Range<usize> { self.0.get_range() }
    fn update_sha1(&self, d: &mut emucore::sha1::Sha1) { self.0.update_sha1(d) }
    fn load_byte(&mut self, a: usize) -> emucore::mem::MemResult<u8> { self.0.load_byte(a) }
    fn store_byte(&mut self, a: usize, v: u8) -> emucore::mem::MemResult<()> {
        if (0xcc00..0xd000).contains(&a) {
            println!("WRITE {a:04x} <= {v:02x}");
        }
        self.0.store_byte(a, v)
    }
    fn store_word(&mut self, a: usize, v: u16) -> emucore::mem::MemResult<()> { self.0.store_word(a, v) }
    fn load_word(&mut self, a: usize) -> emucore::mem::MemResult<u16> { self.0.load_word(a) }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rom_dir = std::env::args().nth(1).ok_or("usage: writes <rom-dir>")?;
    let mut mem = LogBus(StargateBus::from_rom_dir(Path::new(&rom_dir))?);
    let mut regs = Regs::default();
    let mut pins = Pins::default();
    {
        let mut cpu = Context::new(&mut mem, &mut regs, &mut pins)?;
        cpu.reset()?;
        for i in 0..2_697_820usize {
            if let Err(e) = cpu.step() {
                println!("ERR at {i}: {e}");
                break;
            }
        }
    }
    Ok(())
}
