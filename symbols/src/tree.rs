use super::symboltable::SymbolTable;

type ESymbolTreeTree<SCOPEID, SYMID> = ego_tree::Tree<SymbolTable<SCOPEID, SYMID>>;
type ESymbolNodeRef<'a, SCOPEID, SYMID> = ego_tree::NodeRef<'a, SymbolTable<SCOPEID, SYMID>>;
type ESymbolNodeId = ego_tree::NodeId;
type ESymbolNodeMut<'a, SCOPEID, SYMID> = ego_tree::NodeMut<'a, SymbolTable<SCOPEID, SYMID>>;

use super::prelude::*;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq, Clone)]
pub (crate) struct Tree<SCOPEID, SYMID>
where
    SCOPEID: ScopeIdTraits,
    SYMID: SymIdTraits,
{
    tree: ego_tree::Tree<SymbolTable<SCOPEID, SYMID>>,
    scope_id_to_node_id: HashMap<SCOPEID, ESymbolNodeId>,
}

// Internal
impl<SCOPEID, SYMID> Tree<SCOPEID, SYMID>
where
    SCOPEID: ScopeIdTraits,
    SYMID: SymIdTraits,
{
    fn get_node_id_from_scope_id(&self, scope_id: SCOPEID) -> Result<ESymbolNodeId, SymbolError> {
        self.scope_id_to_node_id
            .get(&scope_id)
            .cloned()
            .ok_or(SymbolError::InvalidScope)
    }

    fn get_node_from_id(
        &self,
        scope_id: SCOPEID,
    ) -> Result<ESymbolNodeRef<SCOPEID, SYMID>, SymbolError> {
        let node_id = self.get_node_id_from_scope_id(scope_id)?;
        self.tree.get(node_id).ok_or(SymbolError::InvalidScope)
    }
}

// Public functions
impl<SCOPEID, SYMID> Tree<SCOPEID, SYMID>
where
    SCOPEID: ScopeIdTraits,
    SYMID: SymIdTraits,
{
    pub fn get_parent_scope_id(&self, scope_id: SCOPEID) -> Option<SCOPEID> {
        self.get_node_from_id(scope_id)
            .expect("Illegal scope id")
            .parent()
            .map(|n| n.value().get_scope_id())
    }

    pub fn new(root: SymbolTable<SCOPEID, SYMID>) -> Self {
        let scope_id = root.get_scope_id();
        let tree: ESymbolTreeTree<SCOPEID, SYMID> = ESymbolTreeTree::new(root);
        let mut scope_id_to_node_id: HashMap<SCOPEID, ESymbolNodeId> = Default::default();
        scope_id_to_node_id.insert(scope_id, tree.root().id());

        Self {
            tree,
            scope_id_to_node_id,
        }
    }
    // @TODO implement this
    // pub fn walk(
    //     &self,
    //     scope_id: SCOPEID,
    // ) -> impl Iterator<Item = &SymbolTable<SCOPEID, SYMID>> + '_ {
    //     panic!()
    // }

    pub fn children(
        &self,
        scope_id: SCOPEID,
    ) -> impl Iterator<Item = &SymbolTable<SCOPEID, SYMID>> + '_ {
        let node = self.get_node_from_id(scope_id).unwrap();
        node.children().map(|n| n.value())
    }

    pub fn get_scope(
        &self,
        scope_id: SCOPEID,
    ) -> Result<&SymbolTable<SCOPEID, SYMID>, SymbolError> {
        self.get_node_from_id(scope_id).map(|n| n.value())
    }

    pub fn on_value_mut<F, R>(&mut self, scope_id: SCOPEID, mut f: F) -> Result<R, SymbolError>
    where
        F: FnMut(&mut SymbolTable<SCOPEID, SYMID>) -> Result<R, SymbolError>,
    {
        let node_id = self.get_node_id_from_scope_id(scope_id)?;

        if let Some(ref mut node_mut) = self.tree.get_mut(node_id) {
            f(node_mut.value())
        } else {
            Err(SymbolError::InvalidId)
        }
    }

    pub fn insert_new_table(
        &mut self,
        tab: SymbolTable<SCOPEID, SYMID>,
    ) -> SCOPEID {
        let parent_id = tab.get_parent_id().expect("Must have a parent");
        let tab_id = tab.get_scope_id();
        let parent_node_id = self.scope_id_to_node_id.get(&parent_id).unwrap();
        let mut parent_mut = self.tree.get_mut(*parent_node_id).unwrap();
        let mut n = parent_mut.append(tab);
        self.scope_id_to_node_id.insert(tab_id, n.id());
        n.value().get_scope_id()
    }
}
