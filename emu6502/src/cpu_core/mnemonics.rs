use serde::{ Serialize,Deserialize };

#[derive(Debug, PartialEq, Eq, Clone, Copy,Hash, Serialize, Deserialize, strum::Display)]
pub enum Mnemonic {
  Adc,
  And,
  Asl,
  Bcc,
  Bcs,
  Beq,
  Bit,
  Bmi,
  Bne,
  Bpl,
  Brk,
  Bvc,
  Bvs,
  Clc,
  Cld,
  Cli,
  Clv,
  Cmp,
  Cpx,
  Cpy,
  Dec,
  Dex,
  Dey,
  Eor,
  Inc,
  Inx,
  Iny,
  Jmp,
  Jsr,
  Lda,
  Ldx,
  Ldy,
  Lsr,
  Nop,
  Ora,
  Pha,
  Php,
  Pla,
  Plp,
  Rol,
  Ror,
  Rti,
  Rts,
  Sbc,
  Sec,
  Sed,
  Sei,
  Sta,
  Stx,
  Sty,
  Tax,
  Tay,
  Tsx,
  Txa,
  Txs,
  Tya,
  Illegal,
}

