
pub fn calc_rel(addr: u16, rel_byte: u8) -> u16 {
    let addr = ( addr as isize ) + ((rel_byte as i8) as isize);
    (addr & 0xffff) as u16
}

pub fn u8_sign_extend(byte: u8) -> u16 {
    if (byte & 0x80) == 0x80 {
        byte as u16
    } else {
        (byte as u16) | 0xff00
    }
}
