use serde::{Deserialize, Deserializer, Serialize};

/// Represents how a flag is modified by an instruction
#[derive(Debug, Copy, Clone, PartialEq, Hash, Eq, Deserialize, Serialize, Default)]
pub enum FlagMod {
    #[default]
    Unaltered,
    One,
    Zero,
    Altered,
}

/// Converts a char into a FlagMod
/// - => unaltered
/// + => altered
/// 0 => zero
/// 1 => one
impl From<char> for FlagMod {
    fn from(val: char) -> Self {
        match val {
            '-' => FlagMod::Unaltered,
            '1' => FlagMod::One,
            '0' => FlagMod::Zero,
            '+' => FlagMod::Altered,
            _ => panic!("What the hell is this {val}"),
        }
    }
}

#[derive(Debug, Default)]
pub struct FlagMods {
    mods: [FlagMod; 8],
    pub alter_mask: u8,
    pub one_mask: u8,
    pub zero_mask: u8,
    pub unaltered_mask: u8,
}

impl FlagMods {
    pub fn mods(&self) -> &[FlagMod; 8] {
        &self.mods
    }
}

fn create_mask(flags: &[FlagMod; 8], flag_mod: FlagMod) -> u8 {
    let mut ret = 0;
    for (i, flag) in flags.iter().enumerate() {
        if flag == &flag_mod {
            ret |= 1 << i;
        }
    }
    ret
}

impl FlagMods {
    pub fn from_mods(mods: [FlagMod; 8]) -> Self {
        use FlagMod::*;
        FlagMods {
            mods,
            alter_mask: create_mask(&mods, Altered),
            one_mask: create_mask(&mods, One),
            unaltered_mask: create_mask(&mods, Unaltered),
            zero_mask: create_mask(&mods, Zero),
        }
    }
}

impl From<String> for FlagMods {
    fn from(txt: String) -> Self {
        assert_eq!(txt.len(), 8);

        let mods = txt
            .chars()
            .map(|x| FlagMod::from(x as char))
            .rev()
            .collect::<Vec<_>>();

        let mods = mods.try_into().unwrap();
        Self::from_mods(mods)
    }
}

impl Into<String> for FlagMods {
    fn into(self) -> String {
        let ret = self
            .mods
            .iter()
            .rev()
            .map(|x| match x {
                FlagMod::Unaltered => '-',
                FlagMod::One => '1',
                FlagMod::Zero => '0',
                FlagMod::Altered => '+',
            })
            .collect::<String>();

        ret
    }
}

