#![deny(unused_imports)]
use mem::memmap::MemMap;
use serde::Deserialize;

use crate::mem;

#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
pub enum MemType {
    Ram,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct MemInit {
    kind: MemType,
    name: String,
    start: usize,
    last: usize,
}

fn create_regions_from_str(input: &str) -> serde_yaml::Result<MemMap> {
    use mem::MemMapIO;

    let mut mm = mem::memmap::MemMap::new();
    let loaded: Vec<MemInit> = serde_yaml::from_str(input)?;

    for l in loaded.iter() {
        let size = (l.last - l.start) + 1;
        mm.add_mem_block(&l.name, false, l.start as u16, size as u32);
    }

    Ok(mm)
}

#[allow(unused_imports)]
mod test {
    use super::*;

    #[test]
    pub fn mem_test() {
        // let x = include_str!("../../../simple.yaml");
        // let mm = create_regions_from_str::<byteorder::BigEndian>(x).unwrap();
        // println!("{:?}", mm);
        // assert!(false);
    }
}
