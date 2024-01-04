use serde::Deserialize;
use serde::Serialize;
#[derive(Debug, Copy, Clone, PartialEq, Hash, Eq, Deserialize, Serialize, )]
pub enum AddrModeEnum {
    AccA,
    AccB,
    Immediate,
    Direct,
    Extended,
    Indexed,
    Inherent,
    Relative,
}
