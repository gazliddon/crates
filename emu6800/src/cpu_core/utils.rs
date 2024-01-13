
pub fn calc_rel(addr: u16, rel_byte: u8) -> u16 {
    let addr = ( addr as isize ) + ((rel_byte as i8) as isize);
    (addr & 0xffff) as u16
}
