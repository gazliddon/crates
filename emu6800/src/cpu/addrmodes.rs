use std::u8;

use emucore::mem::MemResult;
use emucore::mem::MemoryIO;
use serde::Deserialize;
use serde::Serialize;

use super::Machine;

#[derive(Debug, Copy, Clone, PartialEq, Hash, Eq, Deserialize, Serialize)]
pub enum AddrModeEnum {
    AccA,
    AccB,
    Immediate,
    Direct,
    Extended,
    Indexed,
    Inherent,
    Relative,
    Illegal,
}

pub fn u8_sign_extend(byte: u8) -> u16 {
    if (byte & 0x80) == 0x80 {
        byte as u16
    } else {
        (byte as u16) | 0xff00
    }
}

pub trait Bus {
    fn fetch_rel_addr<M: MemoryIO>(&mut self, _m: &mut Machine<M>) -> MemResult<u16> {
        panic!()
    }

    fn fetch_operand<M: MemoryIO>(&mut self, _m: &mut Machine<M>) -> MemResult<u8> {
        panic!()
    }

    fn fetch_operand_16<M: MemoryIO>(&mut self, _m: &mut Machine<M>) -> MemResult<u16> {
        panic!()
    }

    fn store_byte<M: MemoryIO>(
        &self,
        _m: &mut Machine<M>,
        _val: u8,
    ) -> MemResult<()>{
        panic!()
    }

    fn store_word<M: MemoryIO>(
        &self,
        _m: &mut Machine<M>,
        _val: u16,
    ) -> MemResult<()> {
        panic!()
    }
}

struct AccA;
struct AccB;
struct Immediate;
struct Direct;
struct Extended;
struct Indexed;
struct Inherent;
struct Relative;
struct Illegal;

impl Bus for AccA {}
impl Bus for AccB {}
impl Bus for Immediate {}

impl Bus for Direct {}
impl Bus for Extended {}
impl Bus for Indexed {}
impl Bus for Inherent {}
impl Bus for Relative {}
impl Bus for Illegal {}
