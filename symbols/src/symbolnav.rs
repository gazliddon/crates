/// Trait for navigating around a symbol tree
use super::symboltree::{ SymbolTree,ValueTrait };
use super::prelude::*;

pub enum NavError {}

type NResult<T> = Result<T, NavError>;

trait SymbolNav<SCOPEID>
where
    SCOPEID: std::hash::Hash + std::ops::AddAssign<u64> + std::clone::Clone,
{
    fn up(&mut self) -> NResult<()>;
    fn root(&mut self);
    fn cd(&mut self, dir: &str) -> NResult<SCOPEID>;
    fn get_id(&self) -> SCOPEID;
}

pub struct Naver<'a, SCOPEID, SYMID, SYMVALUE>
where
    SCOPEID: ScopeIdTraits,
    SYMID: SymIdTraits,
        SYMVALUE: ValueTrait,
{
    tree: &'a SymbolTree<SCOPEID, SYMID, SYMVALUE>,
    current_scope: SCOPEID,
}

impl<'a, SCOPEID, SYMID, SYMVALUE> Naver<'a, SCOPEID, SYMID,SYMVALUE>
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

impl<'a, SCOPEID, SYMID, SYMVALUE> SymbolNav<SCOPEID> for Naver<'a, SCOPEID, SYMID, SYMVALUE>
where
    SCOPEID: ScopeIdTraits,
    SYMID: SymIdTraits,
    SYMVALUE: ValueTrait,
{
    fn up(&mut self) -> NResult<()> {
        todo!()
    }

    fn root(&mut self) {
        self.current_scope = self.tree.get_root_scope_id();
    }

    fn cd(&mut self, dir: &str) -> NResult<SCOPEID> {
        let _x = ScopedName::new(dir);
        todo!()
    }

    fn get_id(&self) -> SCOPEID {
        self.current_scope
    }
}
