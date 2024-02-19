use serde::{Deserialize, Deserializer, Serialize};
use emucore::flagmods::FlagMods;

// TODO: Can any of these be made generic and put into emu_core
// FlagMod can
// construction of FlagsMods needs attention

bitflags::bitflags! {
    #[derive(Clone, Copy, Debug,Deserialize, Serialize, PartialEq, Default)]
    #[serde(transparent)]
    pub struct StatusReg : u8
        {
        const N = 1 << 7;
        const V = 1 << 6;
        const B = 1 << 5;
        const D = 1 << 3;
        const I = 1 << 2;
        const Z = 1 << 1;
        const C = 1 << 0;
        }
}

pub fn text_6_to_flag_mods(txt: &str) -> FlagMods {
    let ret: FlagMods = add_missing_bits(txt).into();
    ret
}

pub fn flag_mods_to_text_6(fmods: FlagMods) -> String {
    let txt: String = fmods.into();
    remove_unwanted_bits(&txt)
}

pub fn from_text<'de, D>(deserializer: D) -> Result<FlagMods, D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;
    let ret: FlagMods = add_missing_bits(s).into();
    Ok(ret)
}

fn remove_unwanted_bits(txt: &str) -> String {
    format!("{}{}", &txt[..2], &txt[4..])
}

fn add_missing_bits(txt: &str) -> String {
    assert!(txt.len() == 6);
    let b67 = &txt[0..2];
    let b03 = &txt[2..];
    format!("{}--{}", b67, b03)
}

#[cfg(test)]
mod test {
    use super::*;

    use pretty_assertions::assert_eq;

    #[test]
    fn test_flagsmods() {
        let txt = "11111-";
        let fmods = text_6_to_flag_mods(txt);
        let txt2 = flag_mods_to_text_6(fmods);
        assert_eq!(txt2, txt);
    }
}
