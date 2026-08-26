//! 32-bit-addressable memory views and the execution bus.
//!
//! emucore's `Region`/`MemBlock` are limited to a 16-bit address space
//! (designed for the 8-bit cores), so JRISC (32-bit addresses like
//! `$F1B000`) gets its own views. Two roles:
//!
//! - [`SliceMem`] — read-only view of a byte slice (disassembly),
//! - [`JriscBus`] + [`RamBus`] — the execution bus: raw byte/word/long
//!   accesses with the hardware quirks MAME models (unaligned long
//!   accesses to the internal-RAM window are word-paired; `loadb`/`loadw`
//!   on internal RAM return the aligned long).

use crate::cpu::Chip;
use byteorder::{BigEndian, ByteOrder};
use emucore::mem::{MemErrorTypes, MemResult, MemoryIO};
use emucore::sha1::Sha1;

/// Read-only view of a byte slice mapped at `base`.
pub struct SliceMem<'a> {
    pub base: usize,
    pub data: &'a [u8],
}

impl<'a> SliceMem<'a> {
    pub fn new(base: usize, data: &'a [u8]) -> Self {
        Self { base, data }
    }

    fn idx(&self, addr: usize) -> Option<usize> {
        let i = addr.checked_sub(self.base)?;
        (i < self.data.len()).then_some(i)
    }

    fn idx2(&self, addr: usize) -> Option<usize> {
        let i = self.idx(addr)?;
        i.checked_add(2).filter(|&e| e <= self.data.len())?;
        Some(i)
    }
}

impl MemoryIO for SliceMem<'_> {
    fn inspect_byte(&self, addr: usize) -> MemResult<u8> {
        self.idx(addr)
            .map(|i| self.data[i])
            .ok_or(MemErrorTypes::IllegalAddress(addr))
    }

    fn inspect_word(&self, addr: usize) -> MemResult<u16> {
        let i = self.idx2(addr).ok_or(MemErrorTypes::IllegalAddress(addr))?;
        Ok(BigEndian::read_u16(&self.data[i..i + 2]))
    }

    fn upload(&mut self, addr: usize, _data: &[u8]) -> MemResult<()> {
        Err(MemErrorTypes::IllegalWrite(addr))
    }

    fn get_range(&self) -> std::ops::Range<usize> {
        self.base..(self.base + self.data.len())
    }

    fn update_sha1(&self, _digest: &mut Sha1) {}

    fn load_byte(&mut self, addr: usize) -> MemResult<u8> {
        self.inspect_byte(addr)
    }

    fn store_byte(&mut self, addr: usize, _val: u8) -> MemResult<()> {
        Err(MemErrorTypes::IllegalWrite(addr))
    }

    fn store_word(&mut self, addr: usize, _val: u16) -> MemResult<()> {
        Err(MemErrorTypes::IllegalWrite(addr))
    }

    fn load_word(&mut self, addr: usize) -> MemResult<u16> {
        self.inspect_word(addr)
    }
}

/// Execution bus: raw byte/word/long accesses at 32-bit addresses.
///
/// `read_word` is the *raw* fetch access (MAME's `ROPCODE`); the
/// `loadb`/`loadw` internal-RAM quirk is implemented by the executor using
/// [`JriscBus::internal_range`].
pub trait JriscBus {
    fn read_byte(&self, addr: u32) -> u32;
    fn read_word(&self, addr: u32) -> u32;
    fn read_long(&self, addr: u32) -> u32;
    fn write_byte(&mut self, addr: u32, v: u32);
    fn write_word(&mut self, addr: u32, v: u32);
    fn write_long(&mut self, addr: u32, v: u32);

    /// the CPU's internal program RAM window (base, size); (0,0) if none
    fn internal_range(&self) -> (u32, u32) {
        (0, 0)
    }
}

/// Forwarding impl: `&mut B` is itself a bus. This lets `step` be generic
/// over `B: JriscBus` (monomorphized direct calls) while existing callers
/// holding `&mut dyn JriscBus` keep working (virtual via the forward).
impl<B: JriscBus + ?Sized> JriscBus for &mut B {
    fn read_byte(&self, addr: u32) -> u32 {
        (**self).read_byte(addr)
    }
    fn read_word(&self, addr: u32) -> u32 {
        (**self).read_word(addr)
    }
    fn read_long(&self, addr: u32) -> u32 {
        (**self).read_long(addr)
    }
    fn write_byte(&mut self, addr: u32, v: u32) {
        (**self).write_byte(addr, v)
    }
    fn write_word(&mut self, addr: u32, v: u32) {
        (**self).write_word(addr, v)
    }
    fn write_long(&mut self, addr: u32, v: u32) {
        (**self).write_long(addr, v)
    }
    fn internal_range(&self) -> (u32, u32) {
        (**self).internal_range()
    }
}

/// Writable execution bus: internal program RAM + an external DRAM window;
/// anything unmapped reads as `unmapped_read` (0) and ignores writes.
pub struct RamBus {
    pub internal_base: u32,
    pub internal: Vec<u8>,
    pub external_base: u32,
    pub external: Vec<u8>,
    pub unmapped_read: u32,
}

impl RamBus {
    pub fn new(chip: Chip, internal: Vec<u8>) -> Self {
        Self {
            internal_base: chip.ram_base(),
            internal,
            external_base: 0,
            external: Vec::new(),
            unmapped_read: 0,
        }
    }

    pub fn with_external(mut self, base: u32, data: Vec<u8>) -> Self {
        self.external_base = base;
        self.external = data;
        self
    }

    fn ext_idx(&self, addr: u32) -> Option<usize> {
        let i = addr.checked_sub(self.external_base)? as usize;
        (i + 4 <= self.external.len()).then_some(i)
    }

    fn ext_idx1(&self, addr: u32) -> Option<usize> {
        let i = addr.checked_sub(self.external_base)? as usize;
        (i < self.external.len()).then_some(i)
    }

    fn ext_idx2(&self, addr: u32) -> Option<usize> {
        let i = addr.checked_sub(self.external_base)? as usize;
        (i + 2 <= self.external.len()).then_some(i)
    }

    fn int_idx1(&self, addr: u32) -> Option<usize> {
        let i = addr.checked_sub(self.internal_base)? as usize;
        (i < self.internal.len()).then_some(i)
    }

    fn int_idx2(&self, addr: u32) -> Option<usize> {
        let i = addr.checked_sub(self.internal_base)? as usize;
        (i + 2 <= self.internal.len()).then_some(i)
    }

    fn int_raw_idx(&self, addr: u32) -> Option<usize> {
        let i = addr.checked_sub(self.internal_base)? as usize;
        (i + 4 <= self.internal.len()).then_some(i)
    }
}

impl JriscBus for RamBus {
    fn read_byte(&self, addr: u32) -> u32 {
        if let Some(i) = self.int_idx1(addr) {
            return self.internal[i] as u32;
        }
        if let Some(i) = self.ext_idx1(addr) {
            return self.external[i] as u32;
        }
        self.unmapped_read
    }

    fn read_word(&self, addr: u32) -> u32 {
        if let Some(i) = self.int_idx2(addr) {
            return BigEndian::read_u16(&self.internal[i..i + 2]) as u32;
        }
        if let Some(i) = self.ext_idx2(addr) {
            return BigEndian::read_u16(&self.external[i..i + 2]) as u32;
        }
        self.unmapped_read
    }

    fn read_long(&self, addr: u32) -> u32 {
        if let Some(i) = self.int_raw_idx(addr) {
            return BigEndian::read_u32(&self.internal[i..i + 4]);
        }
        if let Some(i) = self.ext_idx(addr) {
            return BigEndian::read_u32(&self.external[i..i + 4]);
        }
        self.unmapped_read
    }

    fn write_byte(&mut self, addr: u32, v: u32) {
        if let Some(i) = self.int_idx1(addr) {
            self.internal[i] = v as u8;
            return;
        }
        if let Some(i) = self.ext_idx1(addr) {
            self.external[i] = v as u8;
        }
    }

    fn write_word(&mut self, addr: u32, v: u32) {
        if let Some(i) = self.int_idx2(addr) {
            BigEndian::write_u16(&mut self.internal[i..i + 2], v as u16);
            return;
        }
        if let Some(i) = self.ext_idx2(addr) {
            BigEndian::write_u16(&mut self.external[i..i + 2], v as u16);
        }
    }

    fn write_long(&mut self, addr: u32, v: u32) {
        if let Some(i) = self.int_raw_idx(addr) {
            BigEndian::write_u32(&mut self.internal[i..i + 4], v);
            return;
        }
        if let Some(i) = self.ext_idx(addr) {
            BigEndian::write_u32(&mut self.external[i..i + 4], v);
        }
    }

    fn internal_range(&self) -> (u32, u32) {
        (self.internal_base, self.internal.len() as u32)
    }
}

impl JriscBus for SliceMem<'_> {
    fn read_byte(&self, addr: u32) -> u32 {
        self.idx(addr as usize)
            .map(|i| self.data[i] as u32)
            .unwrap_or(0)
    }

    fn read_word(&self, addr: u32) -> u32 {
        self.idx2(addr as usize)
            .map(|i| BigEndian::read_u16(&self.data[i..i + 2]) as u32)
            .unwrap_or(0)
    }

    fn read_long(&self, addr: u32) -> u32 {
        let i = self.idx(addr as usize);
        match i {
            Some(i) if i + 4 <= self.data.len() => BigEndian::read_u32(&self.data[i..i + 4]),
            _ => 0,
        }
    }

    fn write_byte(&mut self, _addr: u32, _v: u32) {}
    fn write_word(&mut self, _addr: u32, _v: u32) {}
    fn write_long(&mut self, _addr: u32, _v: u32) {}
}
