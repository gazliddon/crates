use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};

use crate::prelude::*;
use crate::symboltable::SymbolTable;

fn tree_symbolize<S, SCOPEID, SYMID>(
    _tree: &ego_tree::Tree<SymbolTable<SCOPEID, SYMID>>,
    _s: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    SCOPEID: ScopeIdTraits,
    SYMID: SymIdTraits,
{
    panic!()
}


struct Edges<SCOPEID> {
    edges: Vec<(SCOPEID, Vec<SCOPEID>)>,
}

impl <SCOPEID> Edges<SCOPEID> {
}

impl<SCOPEID, SYMID, SYMVALUE> Serialize for SymbolTree<SCOPEID, SYMID, SYMVALUE>
where
    SCOPEID: ScopeIdTraits + Serialize,
    SYMID: SymIdTraits + Serialize,
    SYMVALUE: ValueTrait,
{
    fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
   let mut state = _serializer.serialize_struct("SymbolTree", 3)?;
        state.serialize_field("root_scope_id", &self.root_scope_id)?;
        state.serialize_field("next_scope_id", &self.next_scope_id)?;
        state.serialize_field("scope_id_to_symbol_info", &self.scope_id_to_symbol_info)?;
        state.end()
    }
}

#[cfg(test)]
mod test {
    use super::*;
}
