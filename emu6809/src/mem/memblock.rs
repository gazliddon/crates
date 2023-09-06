use std::marker::PhantomData;

use crate::mem::MemErrorTypes;

use super::{MemResult, MemoryIO, Region};
use byteorder::ByteOrder;
use sha1::Sha1;

pub struct MemBlock<E: ByteOrder> {
    pub read_only: bool,
    pub data: Vec<u8>,
    pub name: String,
    pub region: Region,
    phanton: PhantomData<E>,
}

#[allow(dead_code)]
impl<E: ByteOrder> MemBlock<E> {
    pub fn new(name: &str, read_only: bool, r : &std::ops::Range<usize>) -> MemBlock<E> {
        let data = vec![0u8; r.len()];
        Self::from_data(r.start, name, &data, read_only)
    }

    pub fn from_data(addr: usize, name: &str, data: &[u8], read_only: bool) -> MemBlock<E> {
        let size = data.len();

        let mr = Region::checked_new(addr, size).unwrap();

        MemBlock {
            read_only,
            data: data.to_vec(),
            name: name.to_string(),
            region: mr,
            phanton: Default::default(),
        }
    }
    fn to_index(&self, addr: usize) -> MemResult<usize> {
        if self.region.is_in_region(addr) {
            Ok( addr - self.region.addr )

        } else {
            Err(MemErrorTypes::IllegalAddress(addr))
        }
    }
}

#[allow(dead_code)]
impl<E: ByteOrder> MemoryIO for MemBlock<E> {
    fn inspect_byte(&self, addr: usize) -> MemResult<u8> {
        let i = self.to_index(addr)?;
        let d = self.data[i];
        Ok(d)
    }

    fn inspect_word(&self, addr: usize) -> MemResult<u16> {
        let i = self.to_index(addr)?;
        let ab = &self.data[i..i+2];
        Ok(E::read_u16(ab))
    }

    fn update_sha1(&self, digest: &mut Sha1) {
        digest.update(&self.data);
    }

    fn upload(&mut self, _addr: usize, _data: &[u8]) -> MemResult<()> {
        let mut addr = _addr;
        for b in _data {
            self.store_byte(addr, *b)?;
            addr += 1;
        }
        Ok(())
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn get_range(&self) -> std::ops::Range<usize> {
        self.region.as_range()
    }

    fn load_byte(&mut self, addr: usize) -> MemResult<u8> {
        let i = self.to_index(addr)?;
        let v = self.data[i];
        Ok(v)
    }

    fn store_byte(&mut self, addr: usize, val: u8) -> MemResult<()> {
        let idx = self.to_index(addr)?;
        self.data[idx] = val;
        Ok(())
    }

    fn store_word(&mut self, addr: usize, val: u16) -> MemResult<()> {
        let mut buf = [0; 2];
        E::write_u16(&mut buf, val);
        self.store_byte(addr, buf[0])?;
        self.store_byte(addr.wrapping_add(1), buf[1])
    }

    fn load_word(&mut self, addr: usize) -> MemResult<u16> {
        let a = self.load_byte(addr)?;
        let b = self.load_byte(addr.wrapping_add(1))?;
        let buf = [a, b];
        Ok(E::read_u16(&buf))
    }
}
