//! 32-bit-addressable memory view.
//!
//! emucore's `Region`/`MemBlock` are limited to a 16-bit address space
//! (designed for the 8-bit cores), so JRISC (32-bit addresses like
//! `$F1B000`) gets its own read-only slice view implementing [`MemoryIO`].

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
