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
        Self { entries: ALL.to_vec() }
    }

    /// The process-wide database (populated once from the generated table).
    pub fn get() -> &'static Dbase {
        static DB: std::sync::OnceLock<Dbase> = std::sync::OnceLock::new();
        DB.get_or_init(Dbase::new)
    }
}

impl Default for Dbase {
    fn default() -> Self {
        Self::new()
    }
}

mod generated {
    include!(concat!(env!("OUT_DIR"), "/isa_macros_68000.rs"));
}
