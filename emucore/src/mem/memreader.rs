use crate::mem::*;

pub struct MemReader<'a> {
    mem: &'a mut dyn MemoryIO,
    start_addr : usize,
    addr: usize,
}

impl<'a> MemReader<'a> {

    pub fn get_taken_range(&self) -> std::ops::Range<usize> {
        self.start_addr .. self.addr
    }

    pub fn get_mem(&mut self) -> &mut dyn MemoryIO {
        self.mem
    }
    pub fn get_addr(&self) -> usize {
        self.addr
    }

    pub fn set_addr(&mut self, addr: usize) {
        self.addr = addr;
        self.start_addr = addr;
    }

    pub fn get_taken_bytes(&self) -> Vec<u8> {
        let r= self.get_taken_range();
        let mut ret = vec![];
        for a in r {
            ret.push( self.mem.inspect_byte(a).unwrap() );
        }
        ret
    }
    pub fn skip_bytes(&mut self, n : usize)  {
        self.addr += n
    }

    pub fn new(mem: &'a mut dyn MemoryIO) -> Self {
        let addr = mem.get_range().start;
        Self { mem, addr, start_addr: addr }
    }

    pub fn peek_byte(&self) -> Result<u8, MemErrorTypes> {
        self.mem.inspect_byte(self.addr )
    }

    pub fn peek_word(&self) -> Result<u16, MemErrorTypes> {
        self.mem.inspect_word(self.addr )
    }

    pub fn next_byte(&mut self) -> Result<u8, MemErrorTypes> {
        let ret = self.mem.load_byte(self.addr );
        self.addr += 1;
        ret
    }
    pub fn next_word(&mut self) -> Result<u16, MemErrorTypes> {
        let ret = self.mem.load_word(self.addr );
        self.addr += 2;
        ret
    }
}
