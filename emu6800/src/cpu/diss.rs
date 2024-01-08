use emucore::mem::MemoryIO;

use super::IsaDatabase;

/// Returns a string disassmbly string and the next instruction address
pub fn diss<M: MemoryIO>(mem: &M,addr: usize, isa: &IsaDatabase) -> (usize, String) {
    let op_code = mem.inspect_byte(addr).unwrap();
    let ins = isa.get_instruction_info_from_opcode(op_code as usize).unwrap();
    let op = ins.get_mnemonic_text();

    let diss = format!("{op} {:?}", ins.addr_mode);

    (addr + ins.opcode_data.size, diss)
}
