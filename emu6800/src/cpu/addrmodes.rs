use serde::Deserialize;
#[derive(Debug, Copy, Clone, PartialEq, Hash, Eq, Deserialize)]
pub enum AddrModeEnum {
    Acc,
    Immediate,
    Direct,
    Extended,
    Indexed,
    Inherent,
    Relative,
}
