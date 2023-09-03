
#[cfg(feature = "serde_support")]
use serde::{Deserialize, Serialize};

////////////////////////////////////////////////////////////////////////////////
// Traits

pub trait ScopeIdTraits:
    std::hash::Hash
    + std::ops::AddAssign<u64>
    + std::clone::Clone
    + std::cmp::Eq
    + From<u64>
    + Copy
    + Default
{
}

pub trait SymIdTraits:
    std::hash::Hash
    + std::ops::AddAssign<u64>
    + std::clone::Clone
    + std::cmp::Eq
    + From<u64>
    + Copy
    + Default
{
}

pub trait SymValueTraits: Clone {}

impl ScopeIdTraits for u64 {}
impl SymIdTraits for u64 {}

////////////////////////////////////////////////////////////////////////////////
#[derive(Debug, PartialEq, Eq, Clone, Hash, Copy)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))] 
pub struct SymbolScopeId<SCOPEID, SYMID>
where
    SCOPEID: ScopeIdTraits,
    SYMID: SymIdTraits,
{
    pub scope_id: SCOPEID,
    pub symbol_id: SYMID,
}

impl<SCOPEID, SYMID> SymbolScopeId<SCOPEID, SYMID>
where
    SCOPEID: ScopeIdTraits,
    SYMID: SymIdTraits,
{
    pub fn new(scope_id: SCOPEID, symbol_id: SYMID) -> Self {
        Self {
            scope_id,
            symbol_id,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
/// Holds information about a symbol

#[derive(Debug, PartialEq, Eq, Clone)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))] 
pub struct SymbolInfo<SCOPEID, SYMID, SYMVALUE>
where
    SCOPEID: ScopeIdTraits,
    SYMID: SymIdTraits,
    SYMVALUE: Clone,
{
    name: String,
    scoped_name: String,
    pub value: Option<SYMVALUE>,
    pub symbol_id: SymbolScopeId<SCOPEID, SYMID>,
}

impl<SCOPEID, SYMID, SYMVALUE> SymbolInfo<SCOPEID, SYMID, SYMVALUE>
where
    SCOPEID: ScopeIdTraits,
    SYMID: SymIdTraits,
    SYMVALUE: Clone,
{
    pub fn new(
        name: &str,
        value: Option<SYMVALUE>,
        symbol_id: SymbolScopeId<SCOPEID, SYMID>,
        fqn: &str,
    ) -> Self {
        Self {
            name: name.to_string(),
            value,
            symbol_id,
            scoped_name: format!("{fqn}::{name}"),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn scoped_name(&self) -> &str {
        &self.scoped_name
    }
}

#[derive(PartialEq, Eq, Clone,Debug)]
pub enum SymbolError
{
    AlreadyDefined,
    InvalidScope,
    Mismatch,
    NotFound,
    NoValue,
    InvalidId,
    HitScopeBarrier,
}

