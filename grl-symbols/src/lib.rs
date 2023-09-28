//!Crate for handling hierachical symbol tables
#![allow(dead_code)]
mod scopedname;
mod symboltable;
mod symboltree;
mod types;
mod tree;
mod error;

#[cfg(feature="serde_support")]
pub mod serialize;
#[cfg(feature="serde_support")]
pub mod deserialize;

pub mod scopenav;
pub mod symboltreereader;
pub mod symboltreewriter;

pub mod prelude {
    pub use super::scopedname::*;
    pub use super::symboltable::SymbolResolutionBarrier;
    pub use super::symboltree::*;
    pub use super::symboltreereader;
    pub use super::symboltreewriter;
    pub use super::scopenav;
    pub use super::types::*;
    pub use super::error::*;
}

pub use prelude::*;


