use super::prelude::*;
use super::symboltree::{SymbolTree, ValueTrait};

pub struct SymbolTreeReader<'a, SCOPEID, SYMID, SYMVALUE>
where
    SCOPEID: ScopeIdTraits,
    SYMID: SymIdTraits,
    SYMVALUE: ValueTrait,
{
    current_scope: SCOPEID,
    syms: &'a SymbolTree<SCOPEID, SYMID, SYMVALUE>,
}

impl<'a, SCOPEID, SYMID, SYMVALUE> SymbolTreeReader<'a, SCOPEID, SYMID, SYMVALUE>
where
    SCOPEID: ScopeIdTraits,
    SYMID: SymIdTraits,
    SYMVALUE: ValueTrait,
{
    pub fn new(syms: &'a SymbolTree<SCOPEID, SYMID, SYMVALUE>, current_scope: SCOPEID) -> Self {
        Self {
            syms,
            current_scope,
        }
    }

    pub fn syms(&self) -> &SymbolTree<SCOPEID, SYMID, SYMVALUE> {
        self.syms
    }

    pub fn get_symbol_info(
        &self,
        name: &str,
    ) -> Result<&SymbolInfo<SCOPEID, SYMID, SYMVALUE>, SymbolError> {
        let id = self.syms.resolve_label(
            name,
            self.current_scope,
            SymbolResolutionBarrier::default(),
        )?;

        self.get_symbol_info_from_id(id)
    }

    pub fn get_symbol_info_from_id(
        &self,
        id: SymbolScopeId<SCOPEID, SYMID>,
    ) -> Result<&SymbolInfo<SCOPEID, SYMID, SYMVALUE>, SymbolError> {
        self.syms.get_symbol_info_from_id(id)
    }
}
