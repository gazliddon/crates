use super::prelude::*;
use super::symboltable::SymbolTable;
use std::collections::HashMap;

type ESymbolTreeTree<SCOPEID, SYMID> = ego_tree::Tree<SymbolTable<SCOPEID, SYMID>>;
type ESymbolNodeRef<'a, SCOPEID, SYMID> = ego_tree::NodeRef<'a, SymbolTable<SCOPEID, SYMID>>;
type ESymbolNodeId = ego_tree::NodeId;
type ESymbolNodeMut<'a, SCOPEID, SYMID> = ego_tree::NodeMut<'a, SymbolTable<SCOPEID, SYMID>>;

#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) struct Tree<SCOPEID, SYMID>
where
    SCOPEID: ScopeIdTraits,
    SYMID: SymIdTraits,
{
    tree: ego_tree::Tree<SymbolTable<SCOPEID, SYMID>>,
    scope_id_to_node_id: HashMap<SCOPEID, ESymbolNodeId>,
    child_scope_ids: HashMap<SCOPEID, HashMap<String, SCOPEID>>,
}

impl<SCOPEID, SYMID> Default for Tree<SCOPEID, SYMID>
where
    SCOPEID: ScopeIdTraits,
    SYMID: SymIdTraits,
{
    fn default() -> Self {
        let root = SymbolTable::new(
            "",
            "",
            SCOPEID::from(0),
            None,
            SymbolResolutionBarrier::default(),
        );
        Self::new(root)
    }
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
    ) -> Result<ESymbolNodeRef<'_, SCOPEID, SYMID>, SymbolError> {
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
            .ok()
            .and_then(|node| node.parent())
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
            child_scope_ids: HashMap::new(),
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
    ) -> Box<dyn Iterator<Item = &SymbolTable<SCOPEID, SYMID>> + '_> {
        match self.get_node_from_id(scope_id) {
            Ok(node) => Box::new(node.children().map(|n| n.value())),
            Err(_) => Box::new(std::iter::empty()),
        }
    }

    pub fn get_child_scope_id(
        &self,
        parent_id: SCOPEID,
        name: &str,
    ) -> Result<SCOPEID, SymbolError> {
        self.child_scope_ids
            .get(&parent_id)
            .and_then(|children| children.get(name))
            .copied()
            .ok_or(SymbolError::NotFound)
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

    /// Return a vector with ids of all scopes
    pub(crate) fn get_scopes(&self) -> Vec<SCOPEID> {
        let ret: Vec<_> = self.scope_id_to_node_id.keys().cloned().collect();
        ret
    }
    /// Return a vector with ids of all scopes
    pub(crate) fn get_scopes_info(&self) -> Vec<&SymbolTable<SCOPEID, SYMID>> {
        self.scope_id_to_node_id
            .keys()
            .cloned()
            .filter_map(|id| self.get_scope(id).ok())
            .collect()
    }

    pub fn insert_new_table(
        &mut self,
        tab: SymbolTable<SCOPEID, SYMID>,
    ) -> Result<SCOPEID, SymbolError> {
        let parent_id = tab.get_parent_id().ok_or(SymbolError::InvalidScope)?;
        let tab_id = tab.get_scope_id();
        let parent_node_id = self
            .scope_id_to_node_id
            .get(&parent_id)
            .ok_or(SymbolError::InvalidScope)?;
        let mut parent_mut = self
            .tree
            .get_mut(*parent_node_id)
            .ok_or(SymbolError::InvalidScope)?;
        let mut n = parent_mut.append(tab);
        self.scope_id_to_node_id.insert(tab_id, n.id());
        self.child_scope_ids
            .entry(parent_id)
            .or_default()
            .insert(n.value().get_scope_name().to_owned(), tab_id);
        Ok(n.value().get_scope_id())
    }
}
