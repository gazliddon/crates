use std::collections::HashMap;
use thin_vec::ThinVec;

use super::prelude::*;

#[cfg(feature = "serde_support")]
use serde::Serialize;

use super::tree::Tree;

use super::{
    symboltable::SymbolTable, symboltreereader::SymbolTreeReader,
    symboltreewriter::SymbolTreeWriter,
};

// @TODO implement this
struct WalkScopesStruct<'a, SCOPEID, SYMID>
where
    SCOPEID: ScopeIdTraits,
    SYMID: SymIdTraits,
{
    parent: Option<&'a SymbolTable<SCOPEID, SYMID>>,
    table: &'a SymbolTable<SCOPEID, SYMID>,
}

#[cfg(feature = "serde_support")]
pub trait ValueTrait: Clone + Serialize {}

#[cfg(not(feature = "serde_support"))]
pub trait ValueTrait: Clone {}

#[derive(Debug, PartialEq, Clone)]
pub struct ScopeInfo<SCOPEID>
where
    SCOPEID: ScopeIdTraits,
{
    pub name: String,
    pub fqn: String,
    pub scope_id: SCOPEID,
    pub parent_id: Option<SCOPEID>,
}

impl ValueTrait for i64 {}
impl ValueTrait for u64 {}


#[derive(Debug, PartialEq, Eq, Clone)]
// #[cfg_attr(feature = "serde_support", derive(Serialize,Deserialize))]
pub struct SymbolTree<SCOPEID, SYMID, SYMVALUE>
where
    SCOPEID: ScopeIdTraits,
    SYMID: SymIdTraits,
    SYMVALUE: ValueTrait ,
{
    // #[cfg_attr(feature = "serde_support", serde(skip))]
    pub(crate) etree: Tree<SCOPEID, SYMID>,
    pub(crate) root_scope_id: SCOPEID,
    pub(crate) next_scope_id: SCOPEID,

    // #[cfg_attr(feature = "serde_support", serde(serialize_with = "serialize_vals"))]
    pub(crate) scope_id_to_symbol_info:
        HashMap<SymbolScopeId<SCOPEID, SYMID>, SymbolInfo<SCOPEID, SYMID, SYMVALUE>>,
}

////////////////////////////////////////////////////////////////////////////////
impl<SCOPEID, SYMID, SYMVALUE> Default for SymbolTree<SCOPEID, SYMID, SYMVALUE>
where
    SCOPEID: ScopeIdTraits,
    SYMID: SymIdTraits,
    SYMVALUE: ValueTrait,
{
    fn default() -> Self {
        let root_scope = 0;

        let root_table: SymbolTable<SCOPEID, SYMID> = SymbolTable::new(
            "",
            "",
            root_scope.into(),
            None,
            SymbolResolutionBarrier::default(),
        );
        let etree = Tree::new(root_table);

        Self {
            etree,
            root_scope_id: root_scope.into(),
            next_scope_id: (root_scope + 1).into(),
            scope_id_to_symbol_info: Default::default(),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// Scope management
impl<SCOPEID, SYMID, V> SymbolTree<SCOPEID, SYMID, V>
where
    SCOPEID: ScopeIdTraits,
    SYMID: SymIdTraits,
    V: ValueTrait,
{
    pub fn set_symbol_for_id(
        &mut self,
        symbol_id: SymbolScopeId<SCOPEID, SYMID>,
        val: V,
    ) -> Result<(), SymbolError> {
        self.on_symbol_mut(symbol_id, |si| {
            si.value = Some(val.clone());
            Ok(())
        })
    }

    pub fn remove_symbol_for_id(
        &mut self,
        name: &str,
        scope_id: SCOPEID,
    ) -> Result<(), SymbolError> {
        self.etree
            .on_value_mut(scope_id, |syms| syms.remove_symbol(name))
    }

    fn on_symbol_mut<F, R>(
        &mut self,
        id: SymbolScopeId<SCOPEID, SYMID>,
        mut f: F,
    ) -> Result<R, SymbolError>
    where
        F: FnMut(&mut SymbolInfo<SCOPEID, SYMID, V>) -> Result<R, SymbolError>,
    {
        let x = self
            .scope_id_to_symbol_info
            .get_mut(&id)
            .ok_or(SymbolError::NotFound)?;
        f(x)
    }
    pub fn set_value_for_id(
        &mut self,
        id: SymbolScopeId<SCOPEID, SYMID>,
        val: V,
    ) -> Result<(), SymbolError> {
        self.on_symbol_mut(id, move |sym| {
            sym.value = Some(val.clone());
            Ok(())
        })
    }
    pub fn add_reference_symbol(
        &mut self,
        name: &str,
        scope_id: SCOPEID,
        symbol_id: SymbolScopeId<SCOPEID, SYMID>,
    ) -> Result<(), SymbolError> {
        self.etree
            .on_value_mut(scope_id, |syms| syms.add_reference_symbol(name, symbol_id))
    }
    pub fn create_symbol_in_scope(
        &mut self,
        scope_id: SCOPEID,
        name: &str,
    ) -> Result<SymbolScopeId<SCOPEID, SYMID>, SymbolError> {
        let (si, symbol_id) = self.etree.on_value_mut(scope_id, |syms| {
            let symbol_id = syms.create_symbol(name)?;
            let si = SymbolInfo::new(name, None, symbol_id, syms.get_scope_fqn_name());
            Ok((si, symbol_id))
        })?;

        self.scope_id_to_symbol_info.insert(symbol_id, si);
        Ok(symbol_id)
    }

    // @TODO this is bad, get rid of it
    pub fn dump_syms(&self, scope_id: SCOPEID) {
        let syms = self.etree.get_scope(scope_id).unwrap();
        println!("{:#?}", syms.name_to_id.keys());
    }

    pub fn resolve_label(
        &self,
        name: &str,
        scope_id: SCOPEID,
        barrier: SymbolResolutionBarrier,
    ) -> Result<SymbolScopeId<SCOPEID, SYMID>, SymbolError> {
        let mut node_scope_id = Some(scope_id);

        while let Some(n) = node_scope_id {
            let v = self.etree.get_scope(n).unwrap();

            if let Ok(exists) = v.get_symbol_id(name) {
                return Ok(exists);
            }

            if !v.get_symbol_resoultion_barrier().can_pass_barrier(barrier) {
                return Err(SymbolError::HitScopeBarrier);
            }

            node_scope_id = self.etree.get_parent_scope_id(n);
        }

        Err(SymbolError::NotFound)
    }

    pub fn get_symbol_info_from_scoped_name(
        &self,
        name: &ScopedName,
    ) -> Result<&SymbolInfo<SCOPEID, SYMID, V>, SymbolError> {
        assert!(name.is_abs());

        let scopes = name.path();
        let name = name.symbol();

        let mut current_node = self.get_root_scope_id();

        let mut found = false;

        for path_part in scopes.iter() {
            for c in self.etree.children(current_node) {
                if c.get_scope_name() == *path_part {
                    current_node = c.get_scope_id();
                    found = true;
                    break;
                }
            }

            if !found {
                return Err(SymbolError::NotFound);
            }
        }

        self.get_symbol_info(name, current_node)
    }

    pub fn get_root_scope_id(&self) -> SCOPEID {
        self.root_scope_id
    }

    pub fn create_or_get_scope_for_parent(&mut self, name: &str, id: SCOPEID) -> SCOPEID {
        for v in self.etree.children(id) {
            if v.get_scope_name() == name {
                let id = v.get_scope_id();
                return id;
            }
        }

        let new_table = self.create_new_table(name, id, SymbolResolutionBarrier::default());
        self.etree.insert_new_table(new_table)
    }
}

////////////////////////////////////////////////////////////////////////////////
// Public functions
impl<SCOPEID, SYMID, V> SymbolTree<SCOPEID, SYMID, V>
where
    SCOPEID: ScopeIdTraits,
    SYMID: SymIdTraits,
    V: ValueTrait,
{
    pub fn get_sub_scope_id(&self, name: &str, scope_id: SCOPEID) -> Result<SCOPEID, SymbolError> {
        let name = ScopedName::new(name);
        assert!(name.is_relative());
        let path = name.path();
        self.find_sub_scope_id(path, scope_id)
    }

    pub fn get_scope_id(&self, name: &str) -> Result<SCOPEID, SymbolError> {
        let name = ScopedName::new(name);
        assert!(name.is_abs());
        let scope_id = self.get_root_scope_id();
        let path = name.path();
        self.find_sub_scope_id(path, scope_id)
    }
    pub fn new() -> Self {
        Self::default()
    }

    pub fn find_sub_scope_id(
        &self,
        path: &[&str],
        scope_id: SCOPEID,
    ) -> Result<SCOPEID, SymbolError> {
        let mut current_node = scope_id;

        for path_part in path {
            let mut found = false;

            for k in self.etree.children(current_node) {
                if path_part == &k.get_scope_name() {
                    found = true;
                    current_node = k.get_scope_id();
                }
            }

            if !found {
                return Err(SymbolError::NoValue);
            }
        }

        Ok(current_node)
    }

    pub fn create_symbols_in_scope(
        &mut self,
        scope_id: SCOPEID,
        names: &[String],
    ) -> Result<ThinVec<SymbolScopeId<SCOPEID, SYMID>>, SymbolError> {
        let ret: Result<ThinVec<SymbolScopeId<SCOPEID, SYMID>>, SymbolError> = names
            .iter()
            .map(|name| self.create_symbol_in_scope(scope_id, name))
            .collect();
        ret
    }

    pub fn scope_exists(&self, scope_id: SCOPEID) -> bool {
        self.etree.get_scope(scope_id).is_ok()
    }

    pub fn get_fqn_from_id(&self, scope_id: SCOPEID) -> String {
        let scope = self.etree.get_scope(scope_id).expect("Invalid scope");
        scope.get_scope_fqn_name().to_owned()
    }

    pub fn get_writer(&mut self, scope_id: SCOPEID) -> SymbolTreeWriter<SCOPEID, SYMID, V> {
        SymbolTreeWriter::new(self, scope_id)
    }

    pub fn get_root_writer(&mut self) -> SymbolTreeWriter<SCOPEID, SYMID, V> {
        SymbolTreeWriter::new(self, self.get_root_scope_id())
    }

    pub fn get_reader(&self, scope_id: SCOPEID) -> SymbolTreeReader<SCOPEID, SYMID, V> {
        SymbolTreeReader::new(self, scope_id)
    }

    pub fn get_root_reader(&self) -> SymbolTreeReader<SCOPEID, SYMID, V> {
        self.get_reader(self.get_root_scope_id())
    }

    pub fn get_symbol_info_from_id(
        &self,
        symbol_id: SymbolScopeId<SCOPEID, SYMID>,
    ) -> Result<&SymbolInfo<SCOPEID, SYMID, V>, SymbolError> {
        self.scope_id_to_symbol_info
            .get(&symbol_id)
            .ok_or(SymbolError::InvalidId)
    }

    pub fn get_symbol_info_from_name(
        &self,
        name: &str,
    ) -> Result<&SymbolInfo<SCOPEID, SYMID, V>, SymbolError> {
        let name = ScopedName::new(name);
        self.get_symbol_info_from_scoped_name(&name)
    }

    pub fn get_symbol_info(
        &self,
        name: &str,
        scope_id: SCOPEID,
    ) -> Result<&SymbolInfo<SCOPEID, SYMID, V>, SymbolError> {
        let n = self.etree.get_scope(scope_id)?;
        let id = n.get_symbol_id(name)?;
        self.scope_id_to_symbol_info
            .get(&id)
            .ok_or(SymbolError::NotFound)
    }

    pub fn get_scope_info_from_id(&self, scope_id: SCOPEID) -> Option<ScopeInfo<SCOPEID>> {
        let x = self.etree.get_scope(scope_id).ok()?;

        let ret = ScopeInfo {
            name: x.get_scope_name().to_owned(),
            fqn: x.get_scope_fqn_name().to_owned(),
            scope_id,
            parent_id: x.get_parent_id(),
        };

        Some(ret)
    }
}

////////////////////////////////////////////////////////////////////////////////
// Private implementation funcs
impl<SCOPEID, SYMID, V> SymbolTree<SCOPEID, SYMID, V>
where
    SCOPEID: ScopeIdTraits,
    SYMID: SymIdTraits,
    V: ValueTrait,
{
    fn get_and_inc_next_scope_id(&mut self) -> SCOPEID {
        let ret = self.next_scope_id;
        self.next_scope_id += 1;
        ret
    }

    fn get_next_scope_id(&self) -> SCOPEID {
        self.next_scope_id
    }

    fn create_new_table(
        &mut self,
        name: &str,
        parent_id: SCOPEID,
        barrier: SymbolResolutionBarrier,
    ) -> SymbolTable<SCOPEID, SYMID> {
        let parent_fqn = self.get_fqn_from_id(parent_id);
        let fqn = format!("{parent_fqn}::{name}");
        let scope_id = self.get_and_inc_next_scope_id();
        SymbolTable::new(name, &fqn, scope_id, Some(parent_id), barrier)
    }
}

#[allow(unused_imports)]
mod test {
    use super::*;
    use crate::symboltreewriter::SymbolTreeWriter;

    type ScopeId = u64;
    type SymId = u64;
    type SymTree = SymbolTree<ScopeId, SymId, u64>;

    #[test]
    fn test_sym_tree() {
        let syms = [
            ("::scope_a::gaz", 10),
            ("::scope_b::gaz", 20),
            ("::gaz", 30),
        ];

        let mut st = SymTree::default();

        let mut w = st.get_root_writer();

        for (name, val) in syms {
            w.create_and_set_symbol(name, val).expect("Can't create symbols");
        }

        let mut w = st.get_root_writer();

        let _ = w.create_and_set_symbol("root_gaz", 10);

        w.create_or_set_scope("scope_a");

        let _ = w.create_and_set_symbol("gaz", 20);

        let scope_fqn = w.get_scope_fqn();
        println!("SCOPE is {scope_fqn}");
        w.pop();

        let scope_fqn = w.get_scope_fqn();
        println!("SCOPE is {scope_fqn}");

        let gaz = st.get_symbol_info_from_name("::scope_a::gaz").unwrap();
        println!("{:#?}", gaz);
        assert_eq!(gaz.value, Some(20));

        let root_gaz = st.get_symbol_info_from_name("::root_gaz").unwrap();
        println!("{:#?}", root_gaz);
        assert_eq!(root_gaz.value, Some(10));
    }
}
