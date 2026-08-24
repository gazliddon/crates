//! ISA database types and JSON reader.
//!
//! This module is compiled by BOTH the build script (via `#[path]`) and the
//! library, so it must not reference `crate::cpu` or `OUT_DIR`. The
//! generated static table lives in [`generated`] (library only).

pub mod db;
pub use db::*;

/// The process-wide database, populated once from the generated table.
impl Dbase {
    /// Build from the generated static table (OUT_DIR).
    pub fn new() -> Self {
        use generated::ALL;
        let mut by_opcode: Vec<Vec<Insn>> = vec![Vec::new(); 64];
        for insn in ALL {
            by_opcode[insn.opcode].push(*insn);
        }
        Self { unknown: generated::UNKNOWN, by_opcode }
    }

    /// The process-wide database (populated once from the generated table).
    pub fn get() -> &'static Dbase {
        static DB: std::sync::OnceLock<Dbase> = std::sync::OnceLock::new();
        DB.get_or_init(Dbase::new)
    }
}

/// The build-time per-chip opcode dispatch: `DISPATCH[variant][opcode]`
/// is the first entry legal for that chip (or the unknown sentinel). One
/// table load replaces the by-opcode scan + legality check per fetch.
pub fn dispatch(variant: Variant) -> &'static [&'static Insn; 64] {
    &generated::DISPATCH[variant as usize]
}

impl Default for Dbase {
    fn default() -> Self {
        Self::new()
    }
}

mod generated {
    include!(concat!(env!("OUT_DIR"), "/isa_macros_jrisc.rs"));
}
