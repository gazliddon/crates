use serde::{Deserialize, Serialize};
use strum::EnumString;

#[derive(
    Clone, Copy, Deserialize, Serialize, Debug, EnumString, Hash, PartialEq, PartialOrd, Eq, Default,
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
    Asr,
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
    Clv,
    CmpA,
    CmpB,
    Com,
    Cpx,
    Daa,
    Dec,
    Des,
    Dex,
    EorA,
    EorB,
    Inc,
    Ins,
    Inx,
    Jmp,
    Jsr,
    LdaA,
    LdaB,
    Lds,
    Ldx,
    Lsr,
    Neg,
    Nop,
    OraA,
    OraB,
    Psh,
    Pul,
    Rol,
    Ror,
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
    Tsx,
    Txs,
    Wai,

    #[default]
    Illegal,
}