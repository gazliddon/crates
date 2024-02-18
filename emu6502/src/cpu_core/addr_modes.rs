use serde::{Deserialize, Serialize};
use strum::Display;

#[derive(Debug, Copy, Clone, PartialEq, Hash, Eq, Deserialize, Serialize, Default, Display)]
pub enum AddrModeEnum {
    Immediate,
    Inherent,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    IndirectX,
    IndirectY,
    Relative,
    Indirect,
    #[default]
    Illegal,
}

