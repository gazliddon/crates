#![deny(unused_imports)]
use super::{MemMap, MemResult, MemoryIO};
use sha1::Sha1;
use std::cell::RefCell;
use std::fmt;

#[derive(Debug, Clone, Default)]
pub struct LogEntry {
    pub addr: usize,
    pub write: bool,
    pub val: u16,
    pub word: bool,
}

impl LogEntry {
    fn write_byte(addr: usize, val: u8) -> LogEntry {
        LogEntry {
            addr,
            write: true,
            val: u16::from(val),
            word: false,
        }
    }

    fn read_byte(addr: usize, val: u8) -> LogEntry {
        LogEntry {
            addr,
            write: false,
            val: u16::from(val),
            word: false,
        }
    }

    fn write_word(addr: usize, val: u16) -> LogEntry {
        LogEntry {
            addr,
            write: true,
            val,
            word: true,
        }
    }

    fn read_word(addr: usize, val: u16) -> LogEntry {
        LogEntry {
            addr,
            write: false,
            val,
            word: true,
        }
    }
}

impl fmt::Display for LogEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let width_str = if self.word { "16" } else { "8 " };

        let (op_str, arr_str) = if self.write { ("W", "->") } else { ("R", "<-") };

        let val_str = if self.word {
            format!("{:04x}", self.val)
        } else {
            format!("  {:02x}", self.val)
        };

        write!(
            f,
            "{}{} {} {} {:04x}",
            op_str, width_str, val_str, arr_str, self.addr
        )
    }
}

pub struct LoggingMemMap {
    max_log_size: usize,
    mem_map: MemMap,
    log_cell: RefCell<Vec<LogEntry>>,
}

#[allow(dead_code)]
impl LoggingMemMap {
    pub fn new(mm: MemMap) -> LoggingMemMap {
        LoggingMemMap {
            max_log_size: 100,
            mem_map: mm,
            log_cell: RefCell::new(vec![]),
        }
    }

    pub fn get_log(&self) -> Vec<LogEntry> {
        self.log_cell.borrow().clone()
    }

    fn log(&self, txt: LogEntry) {
        let mut v = self.log_cell.borrow_mut();

        v.push(txt);

        if v.len() > self.max_log_size {
            v.truncate(self.max_log_size)
        }
    }

    pub fn clear_log(&mut self) {
        let mut v = self.log_cell.borrow_mut();
        v.truncate(0);
    }
}

impl MemoryIO for LoggingMemMap {
    fn inspect_word(&self, _addr: usize) -> MemResult<u16> {
        panic!()
    }

    fn inspect_byte(&self, _addr: usize) -> MemResult<u8> {
        panic!()
    }
    fn update_sha1(&self, digest: &mut Sha1) {
        self.mem_map.update_sha1(digest)
    }

    fn upload(&mut self, addr: usize, data: &[u8]) -> MemResult<()> {
        self.mem_map.upload(addr, data)
    }

    fn get_name(&self) -> String {
        self.mem_map.get_name()
    }

    fn get_range(&self) -> std::ops::Range<usize> {
        self.mem_map.get_range()
    }

    fn load_byte(&mut self, addr: usize) -> MemResult<u8> {
        let val = self.mem_map.load_byte(addr)?;
        let msg = LogEntry::read_byte(addr, val);
        self.log(msg);
        Ok(val)
    }

    fn store_byte(&mut self, addr: usize, val: u8) -> MemResult<()> {
        self.mem_map.store_byte(addr, val)?;
        let msg = LogEntry::write_byte(addr, val);
        self.log(msg);
        Ok(())
    }

    fn store_word(&mut self, addr: usize, val: u16) -> MemResult<()> {
        self.mem_map.store_word(addr, val)?;
        let msg = LogEntry::write_word(addr, val);
        self.log(msg);
        Ok(())
    }

    fn load_word(&mut self, addr: usize) -> MemResult<u16> {
        let val = self.mem_map.load_word(addr)?;
        let msg = LogEntry::read_word(addr, val);
        self.log(msg);
        Ok(val)
    }
}
