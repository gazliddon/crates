#[derive(Debug, Copy, Clone, PartialEq, Hash, Eq)]
pub enum AddrModeEnum {
    Acc,
    Immediate,
    Direct,
    Extended,
    Indexed,
    Inherent,
    Relative,
}
