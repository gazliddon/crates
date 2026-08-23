//!Crate for handling hierachical symbol tables
#![allow(dead_code)]
mod error;
mod scopedname;
mod symboltable;
mod symboltree;
mod tree;
mod types;

#[cfg(feature = "serde")]
pub mod serialize;

pub mod scopenav;
pub mod symboltreereader;
pub mod symboltreewriter;

pub mod prelude {
    pub use super::error::*;
    pub use super::scopedname::*;
    pub use super::scopenav::{self, ScopeNav, ScopeNavTrait};
    pub use super::symboltable::SymbolResolutionBarrier;
    pub use super::symboltree::*;
    pub use super::symboltreereader;
    pub use super::symboltreewriter;
    pub use super::types::*;
}

pub use prelude::*;
