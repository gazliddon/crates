use serde::{Deserialize, Deserializer, Serialize};

// TODO: Maybe extend this to N bits and always represent internally as a u64?
// would allow to work on procflags up to 64 bits

/// Represents how a set of bits of 8 bits can be modified
/// used for debugging to check emulator against desired results
#[derive(Debug, Copy, Clone, PartialEq, Hash, Eq, Deserialize, Serialize, Default)]
pub enum FlagMod {
    #[default]
    /// Not altered
    Unaltered,
    /// Always 1
    One,
    /// Always 0
    Zero,
    /// Potentially altered
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

/// A collection of 8 FlagMods
/// and masks representing each mod type
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

    pub fn set_mod(self, idx: usize, val: FlagMod) -> Self {
        assert!(idx < 8);
        let mut mods = self.mods;
        mods[idx] = val;
        Self::from_mods(mods)
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
        use FlagMod::*;
        self.mods
            .iter()
            .rev()
            .map(|x| match x {
                Unaltered => '-',
                One => '1',
                Zero => '0',
                Altered => '+',
            })
            .collect()
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
