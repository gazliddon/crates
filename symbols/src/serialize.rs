use serde::ser::SerializeSeq;
use serde::{Serialize, Serializer};

use crate::prelude::*;

use crate::tree::Tree;

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

#[cfg(test)]
mod test {
    use super::*;
}
