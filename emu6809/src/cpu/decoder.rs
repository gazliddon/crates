#![deny(unused_imports)]
use super::mem::MemoryIO;
use super::{CpuErr, CpuResult};
use crate::isa::{Dbase, Instruction};
use crate::mem::MemReader;

const RBYTE: &[u8] = include_bytes!("resources/opcodes.json");

lazy_static::lazy_static! {
    static ref DBASE: Dbase = {
        let bytes_str = std::str::from_utf8(RBYTE).unwrap();
        Dbase::from_text(bytes_str)
    };
}

#[derive(Debug, Clone)]
pub struct InstructionDecoder {
    pub op_code: u16,
    pub cycles: usize,
    pub addr: usize,
    pub data: Vec<u8>,
    pub next_addr: usize,
    pub instruction_info: &'static Instruction,
    pub size: usize,
    pub operand_addr: usize,
}

fn decode_op_mem(addr: usize, reader: &mut dyn MemoryIO) -> CpuResult<InstructionDecoder> {
    let mut reader = MemReader::new(reader);
    reader.set_addr(addr);
    decode_op(&mut reader)
}

// Decode an op
// Takes memory read as closure
// means we can destructively read op code when emulating
// or non destructively inspect for disassembly
fn decode_op(reader: &mut MemReader) -> CpuResult<InstructionDecoder> {
    let addr = reader.get_addr();
    let mut index_size = 0;

    use crate::isa::AddrModeEnum;

    let a = reader.next_byte()? as u16;

    // Fetch the next byte if it's an extended opcode
    let op_code = match a {
        0x10 | 0x11 => (a << 8) + reader.next_byte()? as u16,
        _ => a,
    };

    let operand_addr = reader.get_addr();

    // Fetch a reference to the extended infomation about this
    // opcode
    let instruction_info = DBASE.get(op_code);

    if instruction_info.addr_mode == AddrModeEnum::Indexed {
        let index_mode_id = reader.peek_byte()?;
        let index_mode = super::indexed::IndexedFlags::new(index_mode_id);
        index_size = index_mode.get_index_type().get_size();
    }

    let size = instruction_info.size + index_size;

    reader.set_addr(addr);
    reader.skip_bytes(size);

    let range = reader.get_taken_range();
    let data = reader.get_taken_bytes();

    // Create the decoded instruction
    let ret = InstructionDecoder {
        size: range.len(),
        next_addr: range.end,
        addr,
        op_code,
        instruction_info,
        cycles: instruction_info.cycles,
        data,
        operand_addr,
    };

    Ok(ret)
}

impl InstructionDecoder {
    pub fn new(_addr: u16) -> Self {
        panic!()
    }

    pub fn fetch_inspect_word(&mut self, _mem: &dyn MemoryIO) -> Result<u16, CpuErr> {
        panic!()
        // let w = mem.inspect_word(self.addr.wrapping_add(self.index).into())?;
        // self.index = self.index.wrapping_add(2);
        // Ok(w)
    }

    pub fn fetch_inspecte_byte(&mut self, _mem: &dyn MemoryIO) -> Result<u8, CpuErr> {
        panic!()
        // let b = mem.inspect_byte(self.addr.wrapping_add(self.index.into()).into())?;
        // self.index = self.index.wrapping_add(1);
        // Ok(b)
    }

    pub fn new_from_reader_mut(mem: &mut MemReader) -> CpuResult<Self> {
        decode_op(mem)
    }

    pub fn new_from_reader(mem: &mut MemReader) -> CpuResult<Self> {
        decode_op(mem)
    }

    pub fn new_from_inspect_mem(_addr: usize, _mem: &mut dyn MemoryIO) -> CpuResult<Self> {
        panic!();
    }

    pub fn new_from_read_mem(addr: usize, _mem: &mut dyn MemoryIO) -> CpuResult<Self> {
        decode_op_mem(addr, _mem)
    }

    pub fn fetch_byte(&mut self, mem: &mut dyn MemoryIO) -> u8 {
        let b = mem.load_byte(self.operand_addr).unwrap();
        self.operand_addr += 1;
        b
    }

    pub fn fetch_word(&mut self, mem: &mut dyn MemoryIO) -> Result<u16, CpuErr> {
        let w = mem.load_word(self.operand_addr)?;
        self.operand_addr += 1;
        Ok(w)
    }

    pub fn fetch_byte_as_i8(&mut self, mem: &mut dyn MemoryIO) -> i8 {
        self.fetch_byte(mem) as i8
    }

    pub fn fetch_byte_as_i16(&mut self, mem: &mut dyn MemoryIO) -> i16 {
        i16::from(self.fetch_byte_as_i8(mem))
    }
}
