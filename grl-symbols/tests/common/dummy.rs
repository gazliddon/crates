use grl_symbols::prelude::*;
use serde::{ Serialize, Deserialize };

pub type SymbolId = u64;
pub type ScopeId = u64;

#[derive(Clone,Debug,Serialize, Deserialize )]
pub enum ValueType {
    String(String),
    Num(u64)
}

impl ValueTrait for ValueType {}

pub type Tree = SymbolTree<ScopeId,SymbolId,ValueType>;

