use crate::symboltable::SymbolTable;
use serde::ser::SerializeSeq;
use serde::{Deserialize, Serialize, Serializer};
use std::collections::HashMap;

use crate::prelude::*;
use crate::tree::Tree;

////////////////////////////////////////////////////////////////////////////////
// Serializable version of SymbolTree
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Seriablizable<SCOPEID, SYMID, SYMVALUE>
where
    SCOPEID: ScopeIdTraits + Serialize,
    SYMID: SymIdTraits + Serialize,
    SYMVALUE: Clone + Serialize,
{
    root_scope_id: SCOPEID,
    next_scope_id: SCOPEID,
    scopes: Vec<SymbolTable<SCOPEID, SYMID>>,
    symbols: Vec<SymbolInfo<SCOPEID, SYMID, SYMVALUE>>,
}

impl<SCOPEID, SYMID, SYMVALUE> SymbolTree<SCOPEID, SYMID, SYMVALUE>
where
    SCOPEID: ScopeIdTraits + Serialize,
    SYMID: SymIdTraits + Serialize,
    SYMVALUE: ValueTrait + Serialize,
{
    pub fn to_json(&self) -> String {
        let x: Seriablizable<SCOPEID, SYMID, SYMVALUE> = self.into();
        serde_json::to_string_pretty(&x).expect("Error!")
    }
}

impl<'a, SCOPEID, SYMID, SYMVALUE> SymbolTree<SCOPEID, SYMID, SYMVALUE>
where
    SCOPEID: ScopeIdTraits + Deserialize<'a>,
    SYMID: SymIdTraits + Deserialize<'a>,
    SYMVALUE: ValueTrait + Deserialize<'a>,
{
    pub fn from_json(json_as_string: &'a str) -> Self {
        let y: Seriablizable<SCOPEID, SYMID, SYMVALUE> =
            serde_json::from_str(json_as_string).expect("Error!");
        y.into()
    }
}

impl<SCOPEID, SYMID, SYMVALUE> From<Seriablizable<SCOPEID, SYMID, SYMVALUE>>
    for SymbolTree<SCOPEID, SYMID, SYMVALUE>
where
    SCOPEID: ScopeIdTraits,
    SYMID: SymIdTraits,
    SYMVALUE: ValueTrait,
{
    fn from(value: Seriablizable<SCOPEID, SYMID, SYMVALUE>) -> Self {
        let root_scope_id = value.root_scope_id;
        let next_scope_id = value.next_scope_id;

        let scopes_hash: HashMap<SCOPEID, SymbolTable<SCOPEID, SYMID>> = value
            .scopes
            .iter()
            .map(|st| (st.scope_id, st.clone()))
            .collect();

        let scope_id_to_symbol_info: HashMap<
            SymbolScopeId<SCOPEID, SYMID>,
            SymbolInfo<SCOPEID, SYMID, SYMVALUE>,
        > = value
            .symbols
            .iter()
            .map(|si| (si.symbol_id, si.clone()))
            .collect();

        let _root_scope = scopes_hash
            .get(&root_scope_id)
            .expect("Can't find root scope");

        let etree: Tree<SCOPEID, SYMID> =
            Tree::from(root_scope_id, &scopes_hash, &scope_id_to_symbol_info);

        Self {
            etree,
            next_scope_id,
            root_scope_id,
            scope_id_to_symbol_info,
        }
    }
}

impl<SCOPEID, SYMID, SYMVALUE> From<&SymbolTree<SCOPEID, SYMID, SYMVALUE>>
    for Seriablizable<SCOPEID, SYMID, SYMVALUE>
where
    SCOPEID: ScopeIdTraits,
    SYMID: SymIdTraits,
    SYMVALUE: ValueTrait,
{
    fn from(sym_tree: &SymbolTree<SCOPEID, SYMID, SYMVALUE>) -> Self {
        Seriablizable {
            root_scope_id: sym_tree.root_scope_id,
            next_scope_id: sym_tree.next_scope_id,
            symbols: sym_tree.scope_id_to_symbol_info.values().cloned().collect(),
            scopes: sym_tree
                .etree
                .get_scopes_info()
                .into_iter()
                .cloned()
                .collect(),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
impl<SCOPEID, SYMID> Serialize for Tree<SCOPEID, SYMID>
where
    SCOPEID: ScopeIdTraits + Serialize,
    SYMID: SymIdTraits + Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let scopes = self.get_scopes_info();

        let mut seq = serializer.serialize_seq(Some(scopes.len()))?;

        for e in scopes.iter() {
            seq.serialize_element(e)?;
        }

        seq.end()
    }
}

impl<SCOPEID, SYMID> Tree<SCOPEID, SYMID>
where
    SCOPEID: ScopeIdTraits,
    SYMID: SymIdTraits,
{
    pub fn from<SYMVALUE>(
        root_scope_id: SCOPEID,
        _scopes: &HashMap<SCOPEID, SymbolTable<SCOPEID, SYMID>>,
        _syms: &HashMap<SymbolScopeId<SCOPEID, SYMID>, SymbolInfo<SCOPEID, SYMID, SYMVALUE>>,
    ) -> Self
    where
        SYMVALUE: ValueTrait,
    {
        // create all of the scopes
        let root_scope = _scopes.get(&root_scope_id).unwrap();

        let mut tree = Self::new(root_scope.clone());

        let parent_id = root_scope.scope_id;

        let mut parent_scopes = vec![parent_id];

        while !parent_scopes.is_empty() {

            let mut new_scopes = vec![];

            for parent_id in &parent_scopes {
                for scope in _scopes
                    .values()
                    .filter(|s| s.get_parent_id() == Some(*parent_id))
                {
                    tree.insert_new_table(scope.clone());
                        new_scopes.push(scope.scope_id);

                }
            }
            parent_scopes = new_scopes;
        }

        todo!("Need to reconstruct the tree!")
    }
}

#[cfg(test)]
mod test {
    use super::*;
}
