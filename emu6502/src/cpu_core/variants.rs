use serde::{Deserialize, Serialize};

bitflags::bitflags! {
    #[derive(Clone, Copy, Debug,Deserialize, Serialize, PartialEq, Default)]
    #[serde(transparent)]
    pub struct CpuSupport : u8
        {
            const Mos6502 = 1 << 0;
            const Wdc65C02 = 1 << 1;
            const Hu6280  = 1 << 3;
        }
}

#[derive(Default, Copy, Clone, Debug, PartialEq, strum::Display, Deserialize, Serialize)]
pub enum InstructionKind {
    #[default]
    Standard,
    Undocumented,
}
