//! M68000 bus: 24-bit-addressable byte/word/long accesses, big-endian.
//!
//! Like `JriscBus`, this is a raw access trait (the CPU applies any
//! hardware quirks). The test/execution implementation is region-mapped:
//! reads hit the first region containing the address (else 0), writes hit
//! regions (else ignored) — enough for the T2K module (RAM at $4000, BSS
//! at $8000, the $F1xxxx register window and the DSP program RAM).

/// External bus seen by the 68000.
pub trait M68kBus {
    fn read_byte(&self, addr: u32) -> u8;
    fn read_word(&self, addr: u32) -> u16;
    fn read_long(&self, addr: u32) -> u32;
    fn write_byte(&mut self, addr: u32, v: u8);
    fn write_word(&mut self, addr: u32, v: u16);
    fn write_long(&mut self, addr: u32, v: u32);
}

/// Forwarding impl: `&mut B` is itself a bus (keeps `step` generic while
/// `&mut dyn M68kBus` callers keep working).
impl<B: M68kBus + ?Sized> M68kBus for &mut B {
    fn read_byte(&self, addr: u32) -> u8 {
        (**self).read_byte(addr)
    }
    fn read_word(&self, addr: u32) -> u16 {
        (**self).read_word(addr)
    }
    fn read_long(&self, addr: u32) -> u32 {
        (**self).read_long(addr)
    }
    fn write_byte(&mut self, addr: u32, v: u8) {
        (**self).write_byte(addr, v)
    }
    fn write_word(&mut self, addr: u32, v: u16) {
        (**self).write_word(addr, v)
    }
    fn write_long(&mut self, addr: u32, v: u32) {
        (**self).write_long(addr, v)
    }
}

/// Region-mapped bus. Regions are searched in map order; unread regions
/// read as 0, unwritten regions ignore the write.
#[derive(Debug, Clone, Default)]
pub struct Ram68k {
    regions: Vec<(u32, Vec<u8>)>,
}

impl Ram68k {
    pub fn new() -> Self {
        Self::default()
    }

    /// Map `data` at `base` (existing region at the same base is replaced).
    pub fn map(&mut self, base: u32, data: Vec<u8>) -> &mut Self {
        if let Some((_, old)) = self.regions.iter_mut().find(|(b, _)| *b == base) {
            *old = data;
        } else {
            self.regions.push((base, data));
        }
        self
    }

    /// Convenience: map a byte slice.
    pub fn map_slice(&mut self, base: u32, data: &[u8]) -> &mut Self {
        self.map(base, data.to_vec())
    }

    /// region index + offset for a full `len`-byte access at `addr`
    fn at(&self, addr: u32, len: u32) -> Option<(usize, usize)> {
        self.regions
            .iter()
            .position(|(base, data)| {
                addr >= *base && (addr as u64 - *base as u64) + len as u64 <= data.len() as u64
            })
            .map(|ri| (ri, (addr - self.regions[ri].0) as usize))
    }

    /// Raw peek at one byte (for tests).
    pub fn peek(&self, addr: u32) -> Option<u8> {
        let (ri, off) = self.at(addr, 1)?;
        Some(self.regions[ri].1[off])
    }
}

impl M68kBus for Ram68k {
    fn read_byte(&self, addr: u32) -> u8 {
        match self.at(addr, 1) {
            Some((ri, off)) => self.regions[ri].1[off],
            None => 0,
        }
    }

    fn read_word(&self, addr: u32) -> u16 {
        match self.at(addr, 2) {
            Some((ri, off)) => {
                let d = &self.regions[ri].1;
                u16::from_be_bytes([d[off], d[off + 1]])
            }
            None => 0,
        }
    }

    fn read_long(&self, addr: u32) -> u32 {
        match self.at(addr, 4) {
            Some((ri, off)) => {
                let d = &self.regions[ri].1;
                u32::from_be_bytes([d[off], d[off + 1], d[off + 2], d[off + 3]])
            }
            None => 0,
        }
    }

    fn write_byte(&mut self, addr: u32, v: u8) {
        if let Some((ri, off)) = self.at(addr, 1) {
            self.regions[ri].1[off] = v;
        }
    }

    fn write_word(&mut self, addr: u32, v: u16) {
        if let Some((ri, off)) = self.at(addr, 2) {
            let bytes = v.to_be_bytes();
            let d = &mut self.regions[ri].1;
            d[off] = bytes[0];
            d[off + 1] = bytes[1];
        }
    }

    fn write_long(&mut self, addr: u32, v: u32) {
        if let Some((ri, off)) = self.at(addr, 4) {
            let bytes = v.to_be_bytes();
            let d = &mut self.regions[ri].1;
            d[off..off + 4].copy_from_slice(&bytes);
        }
    }
}
