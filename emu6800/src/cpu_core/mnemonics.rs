use serde::{Deserialize, Serialize};
use strum::{ EnumString, EnumIter };

#[derive(
    Clone, Copy, Deserialize, Serialize, Debug, EnumString, Hash, PartialEq, PartialOrd, Eq, Default,
    EnumIter,
)]
pub enum Mnemonic {
    Aba,
    AdcA,
    AdcB,
    AddA,
    AddB,
    AndA,
    AndB,
    Asl,
    AslA,
    AslB,
    Asr,
    AsrA,
    AsrB,
    Bcc,
    Bcs,
    Beq,
    Bge,
    Bgt,
    Bhi,
    BitA,
    BitB,
    Ble,
    Bls,
    Blt,
    Bmi,
    Bne,
    Bpl,
    Bra,
    Bsr,
    Bvc,
    Bvs,
    Cba,
    Clc,
    Cli,
    Clr,
    ClrA,
    ClrB,
    Clv,
    CmpA,
    CmpB,
    Com,
    ComA,
    ComB,
    Cpx,
    Daa,
    Dec,
    DecA,
    DecB,
    Des,
    Dex,
    EorA,
    EorB,
    Inc,
    IncA,
    IncB,
    Ins,
    Inx,
    Jmp,
    Jsr,
    LdaA,
    LdaB,
    Lds,
    Ldx,
    Lsr,
    LsrA,
    LsrB,
    Neg,
    NegA,
    NegB,
    Nop,
    OraA,
    OraB,
    Psh,
    Pul,
    Rol,
    RolA,
    RolB,
    Ror,
    RorA,
    RorB,
    Rti,
    Rts,
    Sba,
    SbcA,
    SbcB,
    Sec,
    Sei,
    Sev,
    StaA,
    StaB,
    Sts,
    Stx,
    SubA,
    SubB,
    Swi,
    Tab,
    Tap,
    Tba,
    Tpa,
    Tst,
    TstA,
    TstB,
    Tsx,
    Txs,
    Wai,

    #[default]
    Illegal,
}
