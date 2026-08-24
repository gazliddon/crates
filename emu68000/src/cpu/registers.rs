//! M68000 register file (base-68000 programmer's model).
//!
//! 8 data + 8 address registers (32-bit), PC, SR, and the supervisor/user
//! stack pointers. A7 aliases the active stack pointer (selected by SR.S,
//! bit 13).

use std::fmt::Display;
use std::str::FromStr;

use emucore::traits::{RegEnumTrait, RegisterFileTrait};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub enum Reg68k {
    #[default]
    D0,
    D1,
    D2,
    D3,
    D4,
    D5,
    D6,
    D7,
    A0,
    A1,
    A2,
    A3,
    A4,
    A5,
    A6,
    A7,
    Pc,
    Sr,
    Ssp,
    Usp,
}

impl Reg68k {
    pub fn from_index(i: usize) -> Option<Self> {
        Some(match i {
            0 => Reg68k::D0,
            1 => Reg68k::D1,
            2 => Reg68k::D2,
            3 => Reg68k::D3,
            4 => Reg68k::D4,
            5 => Reg68k::D5,
            6 => Reg68k::D6,
            7 => Reg68k::D7,
            8 => Reg68k::A0,
            9 => Reg68k::A1,
            10 => Reg68k::A2,
            11 => Reg68k::A3,
            12 => Reg68k::A4,
            13 => Reg68k::A5,
            14 => Reg68k::A6,
            15 => Reg68k::A7,
            16 => Reg68k::Pc,
            17 => Reg68k::Sr,
            18 => Reg68k::Ssp,
            19 => Reg68k::Usp,
            _ => return None,
        })
    }

    pub fn index(self) -> usize {
        self as usize
    }
}

impl Display for Reg68k {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl FromStr for Reg68k {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim().to_ascii_uppercase();
        match s.as_str() {
            "D0" => Ok(Reg68k::D0),
            "D1" => Ok(Reg68k::D1),
            "D2" => Ok(Reg68k::D2),
            "D3" => Ok(Reg68k::D3),
            "D4" => Ok(Reg68k::D4),
            "D5" => Ok(Reg68k::D5),
            "D6" => Ok(Reg68k::D6),
            "D7" => Ok(Reg68k::D7),
            "A0" => Ok(Reg68k::A0),
            "A1" => Ok(Reg68k::A1),
            "A2" => Ok(Reg68k::A2),
            "A3" => Ok(Reg68k::A3),
            "A4" => Ok(Reg68k::A4),
            "A5" => Ok(Reg68k::A5),
            "A6" => Ok(Reg68k::A6),
            "A7" => Ok(Reg68k::A7),
            "PC" => Ok(Reg68k::Pc),
            "SR" => Ok(Reg68k::Sr),
            "SSP" => Ok(Reg68k::Ssp),
            "USP" => Ok(Reg68k::Usp),
            _ => Err(()),
        }
    }
}

impl RegEnumTrait for Reg68k {
    fn get_size_bytes(&self) -> usize {
        4
    }
}

/// Programmer's-visible state. A7 aliases the active stack pointer.
#[derive(Debug, Clone, Default)]
pub struct Registers {
    pub d: [u32; 8],
    pub a: [u32; 8],
    pub pc: u32,
    pub sr: u16,
    pub ssp: u32,
    pub usp: u32,
}

impl Registers {
    pub fn new() -> Self {
        Self::default()
    }

    /// the active stack pointer (A7): USP when SR.S = 0, SSP when set
    pub fn sp(&self) -> u32 {
        if self.sr & 0x2000 != 0 {
            self.ssp
        } else {
            self.usp
        }
    }

    pub fn set_sp(&mut self, v: u32) {
        if self.sr & 0x2000 != 0 {
            self.ssp = v;
        } else {
            self.usp = v;
        }
    }

    pub fn read_d(&self, n: usize) -> u32 {
        self.d[n & 7]
    }

    pub fn read_a(&self, n: usize) -> u32 {
        if n & 7 == 7 {
            self.sp()
        } else {
            self.a[n & 7]
        }
    }
}

impl RegisterFileTrait<Reg68k> for Registers {
    fn get(&self, r: &Reg68k) -> u64 {
        match r.index() {
            0..=7 => self.d[r.index()] as u64,
            8..=14 => self.a[r.index() - 8] as u64,
            15 => self.sp() as u64,
            16 => self.pc as u64,
            17 => self.sr as u64,
            18 => self.ssp as u64,
            19 => self.usp as u64,
            _ => 0,
        }
    }

    fn set(&mut self, r: &Reg68k, v: u64) {
        match r.index() {
            0..=7 => self.d[r.index()] = v as u32,
            8..=14 => self.a[r.index() - 8] = v as u32,
            15 => self.set_sp(v as u32),
            16 => self.pc = v as u32,
            17 => self.sr = v as u16,
            18 => self.ssp = v as u32,
            19 => self.usp = v as u32,
            _ => {}
        }
    }
}
