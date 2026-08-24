//! ALU semantics — skeleton.
//!
//! The decode/dissembly milestone is complete and validated; instruction
//! *execution* is not implemented yet. This module will map each mnemonic to
//! its semantics; the plan:
//!
//! - 32-bit two's-complement ops: add/addc/sub/subc (carry in/out), and/or/
//!   xor/not, neg/abs, sh/shlq/shrq/sha/sharq/ror/rorq, cmp/cmpq (flags),
//!   mult/imult/imultn (and the MAC: imacn/resmac/mmult/mtoi/normi).
//! - GPU-only sat8/sat16/sat24/pack/unpack vs DSP-only sat16s/sat32s/
//!   mirror/subqmod/addqmod — the shared-opcode numbers must dispatch by
//!   chip (see `crate::isa::Dbase::lookup`).
//! - div: the `.GAS` sources implement software divide via the `sdiv` macro;
//!   the hardware `div` semantics per the SRM Rev 8.
//! - movei's word-swapped immediate must be un-swapped on fetch (done in the
//!   decoder).
//!
//! Reference: "Jaguar Technical Reference Manual for Tom & Jerry", Rev 8
//! (the table vasm's jagrisc backend implements).

/// Placeholder until execution lands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Alu;

impl Alu {
    pub fn add(a: u32, b: u32) -> u32 {
        a.wrapping_add(b)
    }

    pub fn sub(a: u32, b: u32) -> u32 {
        a.wrapping_sub(b)
    }

    pub fn and(a: u32, b: u32) -> u32 {
        a & b
    }

    pub fn or(a: u32, b: u32) -> u32 {
        a | b
    }

    pub fn xor(a: u32, b: u32) -> u32 {
        a ^ b
    }
}
