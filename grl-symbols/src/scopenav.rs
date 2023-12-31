use super::prelude::*;
/// Trait for navigating around a symbol tree
use super::symboltree::{SymbolTree, ValueTrait};

pub enum NavError {
    PathNotFound,
    NoParent,
}

pub type NResult<T> = Result<T, NavError>;

// @TODO Remove this trait and incorporate methods into ScopeNav
trait ScopeNavTrait<SCOPEID>
where
    SCOPEID: std::ops::AddAssign<u64> + std::clone::Clone,
{
    fn up(&mut self) -> NResult<SCOPEID> {
        let id = self.get_parent()?;
        self.set_scope(id.clone());
        Ok(id)
    }

    fn cd(&mut self, dir: &str) -> NResult<SCOPEID> {
        let path = ScopePath::new(dir);

        // If this is an abs path then cd to root
        if path.is_abs() {
            self.set_root()
        }
        // make sure the path is relative
        let path = path.as_relative();

        for path_part in path.path_parts {
            if path_part == ".." {
                self.up()?;
            } else {
                self.cd(path_part)?;
            }
        }

        Ok(self.get_current_scope())
    }

    fn set_root(&mut self) {
        let root_id = self.get_root();
        self.set_scope(root_id)
    }

    fn set_scope(&mut self, id: SCOPEID);
    fn get_root(&self) -> SCOPEID;
    fn get_current_scope(&self) -> SCOPEID;
    fn get_parent(&self) -> NResult<SCOPEID>;
}

#[derive(Clone, Debug)]
pub struct ScopeNav<'a, SCOPEID, SYMID, SYMVALUE>
where
    SCOPEID: ScopeIdTraits,
    SYMID: SymIdTraits,
    SYMVALUE: ValueTrait,
{
    tree: &'a SymbolTree<SCOPEID, SYMID, SYMVALUE>,
    current_scope: SCOPEID,
}

impl<'a, SCOPEID, SYMID, SYMVALUE> ScopeNav<'a, SCOPEID, SYMID, SYMVALUE>
where
    SCOPEID: ScopeIdTraits,
    SYMID: SymIdTraits,
    SYMVALUE: ValueTrait,
{
    pub fn new(tree: &'a SymbolTree<SCOPEID, SYMID, SYMVALUE>) -> Self {
        Self {
            tree,
            current_scope: tree.get_root_scope_id(),
        }
    }
}

impl<'a, SCOPEID, SYMID, SYMVALUE> ScopeNavTrait<SCOPEID> for ScopeNav<'a, SCOPEID, SYMID, SYMVALUE>
where
    SCOPEID: ScopeIdTraits,
    SYMID: SymIdTraits,
    SYMVALUE: ValueTrait,
{
    fn get_parent(&self) -> NResult<SCOPEID> {
        self.tree
            .etree
            .get_scope(self.get_current_scope())
            .and_then(|scope| scope.get_parent_id().ok_or(SymbolError::NoValue))
            .map_err(|_| NavError::NoParent)
    }

    fn get_root(&self) -> SCOPEID {
        self.tree.get_root_scope_id()
    }

    fn get_current_scope(&self) -> SCOPEID {
        self.current_scope
    }

    fn set_scope(&mut self, id: SCOPEID) {
        self.current_scope = id;
    }
}
