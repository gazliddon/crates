// use mem::Memory;
use super::{MemErrorTypes, MemResult, MemoryIO};
use sha1::Sha1;
use std::fmt;

pub trait MemMapIO {
    fn add_memory(&mut self, mem: Box<dyn MemoryIO>);

    fn add_mem_block(&mut self, _name: &str, _read_only: bool, _start: u16, _size: u32) {
        todo!()
    }
}

#[derive(Default)]
pub struct MemMap {
    all_memory: Vec<Box<dyn MemoryIO>>,
    name: String,
}

impl fmt::Debug for MemMap {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut strs: Vec<String> = Vec::with_capacity(self.all_memory.len());

        for m in &self.all_memory {
            let r = m.get_range();
            let msg = format!("{} : ${:04x} ${:04x}", m.get_name(), r.start, r.end - 1);
            strs.push(msg)
        }

        write!(f, "{}", strs.join("\n"))
    }
}

impl MemoryIO for MemMap {
    fn inspect_byte(&self, _addr: usize) -> MemResult<u8> {
       panic!() 
    }

    fn inspect_word(&self, _addr: usize) -> MemResult<u16> {
        panic!()
    }
    fn update_sha1(&self, digest: &mut Sha1) {
        for m in &self.all_memory {
            m.update_sha1(digest);
        }
    }

    fn upload(&mut self, addr: usize, data: &[u8]) -> MemResult<()> {
        for (i, item) in data.iter().enumerate() {
            let a = addr.wrapping_add(i );
            self.store_byte(a, *item)?;
        }
        Ok(())
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn get_range(&self) -> std::ops::Range<usize> {
        0..0x1_0000
    }

    fn load_byte(&mut self, addr: usize) -> MemResult<u8> {
        let m = self.get_region(addr)?;
        m.load_byte(addr)
    }
    fn load_word(&mut self, addr: usize) -> MemResult<u16> {
        let m = self.get_region(addr)?;
        m.load_word(addr)
    }

    fn store_byte(&mut self, addr: usize, val: u8) -> MemResult<()> {
        let m = self.get_region(addr)?;
        m.store_byte(addr, val)
    }

    fn store_word(&mut self, addr: usize, val: u16) -> MemResult<()> {
        let m = self.get_region(addr)?;
        m.store_word(addr, val)
    }
}

#[allow(dead_code)]
impl MemMap {
    fn get_region(&mut self, addr: usize) -> MemResult<&mut Box<dyn MemoryIO>> {
        for m in &mut self.all_memory {
            if m.is_in_range(addr) {
                return Ok(m);
            }
        }

        Err(MemErrorTypes::IllegalAddress(addr))
    }

    pub fn new() -> Self {
        Self {
            all_memory: Vec::with_capacity(64*1024),
            name: "all memory".to_string(),
        }
    }
}

impl MemMapIO for MemMap {
    fn add_memory(&mut self, mem: Box<dyn MemoryIO>) {
        self.all_memory.push(mem)
    }
}
