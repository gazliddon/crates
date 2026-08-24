#![deny(unused_imports)]
use super::{CpuErr, CpuResult};
use crate::isa::{Dbase, Instruction};
use emucore::mem::{MemReader, MemoryIO};

const RBYTE: &[u8] = include_bytes!("../../resources/opcodes6809.json");

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

    // The indexed postbyte (peeked while the reader still points at the
    // operand area) drives both the instruction size and the
    // effective-address cycle extras.
    let index_mode_id = if instruction_info.addr_mode == AddrModeEnum::Indexed {
        let index_mode_id = reader.peek_byte()?;
        let index_mode = super::indexed::IndexedFlags::new(index_mode_id);
        index_size = index_mode.get_index_type().get_size();
        Some(index_mode_id)
    } else {
        None
    };

    let size = instruction_info.size + index_size;

    reader.set_addr(addr);
    reader.skip_bytes(size);

    let range = reader.get_taken_range();
    let data = reader.get_taken_bytes();

    // Indexed addressing adds effective-address cycles beyond the flat
    // table value.  The table base already includes one EA cycle (the
    // `,R`/5-bit-offset dummy read), so the extras below are
    // MAME-m6809 EA cycles minus one:
    //   ,R+ ,-R     3 -> +2        ,R++ ,--R   4 -> +3
    //   B,R A,R     2 -> +1        ,R          1 -> +0
    //   8-bit,PC8   2 -> +1        5-bit       2 -> +1
    //   16-bit,D,R  5 -> +4        PC16        6 -> +5
    //   [abs]       6 -> +5
    let ea_cycles = match index_mode_id {
        Some(index_mode_id) => {
            let index_mode = super::indexed::IndexedFlags::new(index_mode_id);
            use super::IndexModes::*;
            let base = match index_mode.get_index_type() {
                RPlus(_) | RSub(_) => 2,
                RPlusPlus(_) | RSubSub(_) => 3,
                RAddB(_) | RAddA(_) => 1,
                RAddi8(_) | PCAddi8 => 1,
                RAddi16(_) | RAddD(_) => 4,
                PCAddi16 => 5,
                Ea => 5,
                ROff(_, _) => 1,
                RZero(_) => 0,
                Illegal => 0,
            };
            // Indirect postbytes (bit 4) read a 16-bit pointer from the
            // computed address: 2 reads + 1 dummy = 3 cycles on top of
            // the direct EA.  The `[abs]` (0x9F) case is already
            // indirect and its 5 covers the pointer read.
            let indirect = index_mode.is_indirect() && !index_mode.is_ea();
            base + if indirect { 3 } else { 0 }
        }
        None => 0,
    };

    // Create the decoded instruction
    let ret = InstructionDecoder {
        size: range.len(),
        next_addr: range.end,
        addr,
        op_code,
        instruction_info,
        cycles: instruction_info.cycles + ea_cycles,
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
