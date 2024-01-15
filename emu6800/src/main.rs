use emu6800::cpu::{
    Machine, RegisterFile, 
};

use emucore::{
    mem::{MemBlock, MemoryIO},
    byteorder::*,
};


static SND : &[u8;2048] = include_bytes!("../resources/sg.snd");

fn make_machine() -> Machine<MemBlock<BigEndian>, RegisterFile> {
    let regs = RegisterFile::default();
    let mut mem: MemBlock<BigEndian> = MemBlock::new("test", false, &(0..65536));
    mem.store_bytes(0xf800, SND).unwrap();
    Machine::new(mem, regs)
}

fn try_diss() {

    let m = make_machine();

    let mut pc = 0xf800 + 1;
    loop {
        let d = m.diss(pc);

        if let Ok(d) = d {
            let cycles = d.ins.opcode_data.cycles;

            println!("{pc:04x} {:19} [ {cycles} ]    {}", d.mem_string, d.text);
            pc = d.next_pc;
        } else {
            println!("Uknown: {pc:04x} : {:02x}", m.mem().inspect_byte(pc ).unwrap());
            break;
        }
    }

}


fn main() {
    try_diss();
}
