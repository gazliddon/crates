//! JRISC register file: r0–r31, 32-bit, with register-bank support.
//!
//! r31 is the stack pointer by convention (the T2K DSP program does
//! `movei #STACKPOS, r31` on entry). The bank field mirrors `G_REGPAGE`
//! (GPU only); the T2K programs do not switch banks.

use std::fmt::Display;
use std::str::FromStr;

use emucore::traits::{RegEnumTrait, RegisterFileTrait};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub enum RegEnum {
    #[default]
    R0,
    R1,
    R2,
    R3,
    R4,
    R5,
    R6,
    R7,
    R8,
    R9,
    R10,
    R11,
    R12,
    R13,
    R14,
    R15,
    R16,
    R17,
    R18,
    R19,
    R20,
    R21,
    R22,
    R23,
    R24,
    R25,
    R26,
    R27,
    R28,
    R29,
    R30,
    R31,
}

impl RegEnum {
    pub fn from_index(i: usize) -> Option<Self> {
        (0..32).find(|&n| n == i).map(|n| match n {
            0 => RegEnum::R0,
            1 => RegEnum::R1,
            2 => RegEnum::R2,
            3 => RegEnum::R3,
            4 => RegEnum::R4,
            5 => RegEnum::R5,
            6 => RegEnum::R6,
            7 => RegEnum::R7,
            8 => RegEnum::R8,
            9 => RegEnum::R9,
            10 => RegEnum::R10,
            11 => RegEnum::R11,
            12 => RegEnum::R12,
            13 => RegEnum::R13,
            14 => RegEnum::R14,
            15 => RegEnum::R15,
            16 => RegEnum::R16,
            17 => RegEnum::R17,
            18 => RegEnum::R18,
            19 => RegEnum::R19,
            20 => RegEnum::R20,
            21 => RegEnum::R21,
            22 => RegEnum::R22,
            23 => RegEnum::R23,
            24 => RegEnum::R24,
            25 => RegEnum::R25,
            26 => RegEnum::R26,
            27 => RegEnum::R27,
            28 => RegEnum::R28,
            29 => RegEnum::R29,
            30 => RegEnum::R30,
            31 => RegEnum::R31,
            _ => unreachable!(),
        })
    }

    pub fn index(self) -> usize {
        self as usize
    }
}

impl Display for RegEnum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "r{}", self.index())
    }
}

impl FromStr for RegEnum {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_lowercase();
        let s = s.strip_prefix('r').ok_or(())?;
        let n: usize = s.parse().map_err(|_| ())?;
        Self::from_index(n).ok_or(())
    }
}

impl RegEnumTrait for RegEnum {
    fn get_size_bytes(&self) -> usize {
        4
    }
}

/// 32 × 32-bit registers plus the active bank.
#[derive(Debug, Clone, Default)]
pub struct Registers {
    regs: [u32; 32],
    bank: u8,
}

impl Registers {
    pub fn new(bank: u8) -> Self {
        Self { regs: [0; 32], bank }
    }

    pub fn bank(&self) -> u8 {
        self.bank
    }

    pub fn set_bank(&mut self, bank: u8) {
        self.bank = bank;
    }

    /// convenience: read a register by index without an enum value
    pub fn get_index(&self, i: usize) -> u32 {
        self.regs[i & 31]
    }

    pub fn set_index(&mut self, i: usize, v: u32) {
        self.regs[i & 31] = v;
    }
}

impl RegisterFileTrait<RegEnum> for Registers {
    fn get(&self, r: &RegEnum) -> u64 {
        self.regs[r.index()] as u64
    }

    fn set(&mut self, r: &RegEnum, v: u64) {
        self.regs[r.index()] = v as u32;
    }
}
