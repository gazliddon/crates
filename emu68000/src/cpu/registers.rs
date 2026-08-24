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

    // ---- SR flag helpers ----

    /// size mask and sign bit for byte/word/long
    #[inline]
    fn smask(size: u8) -> u32 {
        match size {
            1 => 0xFF,
            2 => 0xFFFF,
            _ => 0xFFFF_FFFF,
        }
    }

    #[inline]
    fn sbit(size: u8) -> u32 {
        match size {
            1 => 0x80,
            2 => 0x8000,
            _ => 0x8000_0000,
        }
    }

    #[inline]
    fn set_sr_bits(&mut self, mask: u16, v: bool) {
        if v {
            self.sr |= mask;
        } else {
            self.sr &= !mask;
        }
    }

    #[inline]
    pub fn set_c(&mut self, v: bool) {
        self.set_sr_bits(0x0001, v);
    }
    #[inline]
    pub fn set_v(&mut self, v: bool) {
        self.set_sr_bits(0x0002, v);
    }
    #[inline]
    pub fn set_z(&mut self, v: bool) {
        self.set_sr_bits(0x0004, v);
    }
    #[inline]
    pub fn set_n(&mut self, v: bool) {
        self.set_sr_bits(0x0008, v);
    }
    #[inline]
    pub fn set_x(&mut self, v: bool) {
        self.set_sr_bits(0x0010, v);
    }

    pub fn c(&self) -> bool {
        self.sr & 0x0001 != 0
    }
    pub fn v(&self) -> bool {
        self.sr & 0x0002 != 0
    }
    pub fn z(&self) -> bool {
        self.sr & 0x0004 != 0
    }
    pub fn n(&self) -> bool {
        self.sr & 0x0008 != 0
    }
    pub fn x(&self) -> bool {
        self.sr & 0x0010 != 0
    }

    /// Z/N from a result (masked to size).
    pub fn set_zn(&mut self, size: u8, v: u32) {
        let m = Self::smask(size);
        self.set_z(v & m == 0);
        self.set_n(v & Self::sbit(size) != 0);
    }

    /// Z/N/V for logical ops (V cleared).
    pub fn set_znv_logic(&mut self, size: u8, v: u32) {
        self.set_zn(size, v);
        self.set_v(false);
    }

    /// Z/N/V/C for addition (C = carry out).
    pub fn set_znv_c_add(&mut self, size: u8, a: u32, b: u32, r: u32) {
        let m = Self::smask(size);
        let sb = Self::sbit(size);
        self.set_zn(size, r);
        self.set_c((r & m) < (a & m)); // carry out (unsigned wrap)
        let o = ((a ^ r) & (b ^ r)) & sb != 0; // overflow: sign(a)==sign(b)!=sign(r)
        self.set_v(o);
    }

    /// Z/N/V/C for subtraction (C = borrow).
    pub fn set_znv_c_sub(&mut self, size: u8, a: u32, b: u32, r: u32) {
        let m = Self::smask(size);
        let sb = Self::sbit(size);
        self.set_zn(size, r);
        self.set_c((a & m) < (b & m)); // borrow
        let o = ((a ^ b) & (a ^ r)) & sb != 0; // overflow: sign(a)!=sign(b)!=sign(r)
        self.set_v(o);
    }

    /// Z/N/V for compare = subtraction without write; sets C too.
    pub fn set_znv_c_cmp(&mut self, size: u8, a: u32, b: u32) {
        let m = Self::smask(size);
        let r = (a & m).wrapping_sub(b & m);
        self.set_znv_c_sub(size, a & m, b & m, r);
    }

    /// The 16 condition codes against the current flags.
    pub fn cond(&self, cc: u8) -> bool {
        match cc & 0xF {
            0x0 => true,                      // T
            0x1 => false,                     // F
            0x2 => !self.c() && !self.z(),    // HI
            0x3 => self.c() || self.z(),      // LS
            0x4 => !self.c(),                 // CC
            0x5 => self.c(),                  // CS
            0x6 => !self.z(),                 // NE
            0x7 => self.z(),                  // EQ
            0x8 => !self.v(),                 // VC
            0x9 => self.v(),                  // VS
            0xA => !self.n(),                 // PL
            0xB => self.n(),                  // MI
            0xC => self.n() == self.v(),      // GE
            0xD => self.n() != self.v(),      // LT
            0xE => !self.z() && self.n() == self.v(), // GT
            _ => self.z() || self.n() != self.v(),    // LE
        }
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
