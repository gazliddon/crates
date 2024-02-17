use serde::{Deserialize, Deserializer, Serialize};

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

/// Represents how a flag is modified by an instruction
#[derive(Debug, Copy, Clone, PartialEq, Hash, Eq, Deserialize, Serialize, Default)]
pub enum FlagMod {
    #[default]
    Unaltered,
    One,
    Zero,
    Altered,
}
/// Converts a char
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

pub fn from_text<'de, D>(deserializer: D) -> Result<[FlagMod; 8], D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;
    let ret = create_flag_changes(s);
    Ok(ret)
}

/// Parses a string showing flag alterations
fn create_flag_changes(txt: &str) -> [FlagMod; 8] {
    println!("input : {txt}");
    assert!(txt.len() == 6);
    // make an 8 bit flag
    let b67 = &txt[5..];
    let b04 = &txt[..5];
    let txt = format!("{}--{}",b04,b67);
    println!("{:?}", txt);
    let ret = txt.chars().map(|x| FlagMod::from(x as char)).collect::<Vec<_>>();
    println!("{:?}", ret);
    ret.try_into().unwrap()
}
