use serde::Deserialize;
use std::fmt;

#[derive(Debug, Copy, Clone, Deserialize, PartialEq, Hash, Eq)]
pub enum AddrModeEnum {
    Indexed,
    Direct,
    Extended,
    Relative,
    Relative16,
    Inherent,
    Immediate8,
    Immediate16,
    RegisterSet,
    RegisterPair,
}
impl fmt::Display for AddrModeEnum {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

mod isa_reader;
pub use isa_reader::*;
