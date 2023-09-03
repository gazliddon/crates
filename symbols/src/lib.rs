//!Crate for handling hierachical symbol tables
#![allow(dead_code)]
mod scopedname;
mod symboltable;
mod symboltree;
mod types;

#[cfg(feature="serde_support")]
mod serialize;

pub mod symbolnav;
pub mod symboltreereader;
pub mod symboltreewriter;

pub mod prelude {
    pub use super::scopedname::*;
    pub use super::symboltable::SymbolResolutionBarrier;
    pub use super::symboltree::*;
    pub use super::symboltreereader;
    pub use super::symboltreewriter;
    pub use super::symbolnav;
    pub use super::types::*;
}

pub use prelude::*;


