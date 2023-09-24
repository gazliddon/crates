use super::{
    ScopeIdTraits, SymIdTraits, SymbolError, symboltree::SymbolNodeRef, SymbolResolutionBarrier, SymbolScopeId,
    symboltree::SymbolTree,
};

use serde::{ser::SerializeMap, Deserialize};
use std::collections::HashMap;

fn split_fqn(text: &str) -> Vec<&str> {
    text.split("::").collect()
}
fn get_subscope<'a, SCOPEID, SYMID>(
    n: SymbolNodeRef<'a, SCOPEID, SYMID>,
    name: &str,
) -> Option<SymbolNodeRef<'a, SCOPEID, SYMID>>
where
    SCOPEID: ScopeIdTraits,
    SYMID: SymIdTraits,
{
    n.children().find(|c| c.value().get_scope_name() == name)
}
impl<SCOPEID, SYMID, V> serde::Serialize for SymbolTree<SCOPEID, SYMID, V>
where
    SCOPEID: ScopeIdTraits,
    SYMID: SymIdTraits,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let hm = self.to_hash_map();
        let mut map = serializer.serialize_map(Some(hm.len()))?;
        for (k, v) in hm {
            map.serialize_entry(&k, &v)?;
        }
        map.end()
    }
}

impl<'de, SCOPEID, SYMID, V> serde::Deserialize<'de> for SymbolTree<SCOPEID, SYMID, V>
where
    SCOPEID: ScopeIdTraits,
    SYMID: SymIdTraits,
    V : Deserialize<'de>,
{
    fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let mut ret = Self::new();
        let hm: HashMap<String, Option<V>> = serde::Deserialize::deserialize(_deserializer)?;

        for (k, v) in hm {
            let symbol_id = ret.create_fqn(&k).unwrap();
            if let Some(v) = v {
                ret.set_symbol_from_id(symbol_id, v).unwrap();
            }
        }

        Ok(ret)
    }

    fn deserialize_in_place<D>(_deserializer: D, _place: &mut Self) -> Result<(), D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        panic!()
        // Default implementation just delegates to `deserialize` impl.
        // *place = try!(Deserialize::deserialize(deserializer));
        // Ok(())
    }
}

impl<SCOPEID, SYMID, V> SymbolTree<SCOPEID, SYMID, V>
where
    SCOPEID: ScopeIdTraits,
    SYMID: SymIdTraits,
{
    pub fn to_hash_map(&self) -> HashMap<String, Option<i64>> {
        panic!()
    }

    pub fn to_json(&self) -> String {
        let hm = self.to_hash_map();
        serde_json::to_string_pretty(&hm).unwrap()
    }

    // pub fn create_and_set_fqn(&mut self, text: &str, val : i64) -> Result<SymbolScopeId, SymbolError> {
    //     let symbol_id = self.create_fqn(text)?;
    //     self.set_symbol_from_id(symbol_id, val)?;
    //     Ok(symbol_id)
    // }

    // This is shit, much shame
    pub fn create_fqn(
        &mut self,
        text: &str,
    ) -> Result<SymbolScopeId<SCOPEID, SYMID>, SymbolError<SCOPEID, SYMID>> {
        let items: Vec<_> = split_fqn(text);

        let (path, sym) = match items.len() {
            0 => panic!("WTF"),
            1 => panic!("Neeed 2!"),
            _ => (&items[0..items.len() - 1], &items[items.len() - 1]),
        };

        assert!(path[0].is_empty());

        // pop the first one off
        let mut scope_id = self.tree.root().value().get_scope_id();

        for part in &path[1..] {
            let n = self.get_node_from_id(scope_id).unwrap();
            let n_id = n.value().get_scope_id();

            if let Some(new_id) = get_subscope(n, part) {
                scope_id = new_id.value().get_scope_id();
            } else {
                let new_scope_id =
                    self.insert_new_table(part, n_id, SymbolResolutionBarrier::default());
                scope_id = new_scope_id
            }
        }

        let symbol_id = self.create_symbol_in_scope(scope_id, sym)?;
        Ok(symbol_id)
    }
}
