use serde::Serializer;

use crate::prelude::*;
use crate::symboltable::SymbolTable;

fn tree_symbolize<S,SCOPEID,SYMID>(_tree: &ego_tree::Tree<SymbolTable<SCOPEID, SYMID>>, _s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    SCOPEID: ScopeIdTraits,
    SYMID: SymIdTraits,
{
    panic!()
}
