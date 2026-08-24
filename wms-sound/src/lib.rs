//! Board-level emulator for the early Williams/Midway sound board.
//!
//! The first target is the Stargate board: a 6800-compatible CPU, 6821 PIA,
//! 128 bytes of RAM, an 8-bit DAC, and an 2 KiB ROM mapped at `$f800`.

mod board;
mod dac;
mod harness;

pub use board::{PiaAccess, SoundBus, WmsSoundBoard, SOUND_PIA_BASE, SOUND_ROM_BASE};
pub use dac::{Dac8, DacEvent};
pub use emucore::pia::{Pia6821, PiaEvent};
pub use harness::{HarnessConfig, SoundCommandEvent, SoundHarness};
