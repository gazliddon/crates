use super::prelude::*;
/// Trait for navigating around a symbol tree
use super::symboltree::{SymbolTree, ValueTrait};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NavError {
    PathNotFound,
    NoParent,
}

pub type NResult<T> = Result<T, NavError>;

/// Navigation operations for a cursor over a [`SymbolTree`].
pub trait ScopeNavTrait<SCOPEID>
where
    SCOPEID: std::ops::AddAssign<u64> + std::clone::Clone,
{
    fn up(&mut self) -> NResult<SCOPEID> {
        let id = self.get_parent()?;
        self.set_scope(id.clone());
        Ok(id)
    }

    fn cd(&mut self, dir: &str) -> NResult<SCOPEID> {
        let path = self.parse_path(dir);

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
                let next = self.find_child(&path_part).ok_or(NavError::PathNotFound)?;
                self.set_scope(next);
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
    fn find_child(&self, name: &str) -> Option<SCOPEID>;

    /// Parses a navigation path using the syntax configured by the owner.
    /// Implementors that do not have custom syntax retain the legacy `::` form.
    fn parse_path<'a>(&self, dir: &'a str) -> ScopePath<'a> {
        ScopePath::new(dir)
    }
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

    fn find_child(&self, name: &str) -> Option<SCOPEID> {
        self.tree
            .etree
            .children(self.current_scope)
            .find(|scope| scope.get_scope_name() == name)
            .map(|scope| scope.get_scope_id())
    }

    fn parse_path<'b>(&self, dir: &'b str) -> ScopePath<'b> {
        ScopePath::parse(dir, self.tree.syntax())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn cd_descends_and_up_returns_to_parent() {
        let mut tree = SymbolTree::<u64, u64, u64>::new();
        let root = tree.get_root_scope_id();
        tree.create_or_get_scope_for_parent("module", root).unwrap();

        let mut nav = ScopeNav::new(&tree);
        assert_eq!(nav.cd("module"), Ok(1));
        assert_eq!(nav.up(), Ok(root));
        assert_eq!(nav.cd("missing"), Err(NavError::PathNotFound));
    }

    #[test]
    fn cd_uses_tree_scope_syntax() {
        let mut tree = SymbolTree::<u64, u64, u64>::with_syntax(ScopeSyntax::new("."));
        let root = tree.get_root_scope_id();
        tree.create_or_get_scope_for_parent("module", root).unwrap();

        let mut nav = ScopeNav::new(&tree);
        assert_eq!(nav.cd(".module"), Ok(1));
    }
}
