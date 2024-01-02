use std::{
    fmt::{Debug, Display},
    hash::Hash,
    str::FromStr,
};

pub trait RegEnumTrait:
    Display + Debug + Clone + PartialEq + Eq + Hash + Ord + FromStr<Err = ()> + Default
{
    fn get_size_bytes(&self) -> usize;
    fn get_size_bits(&self) -> usize {
        self.get_size_bytes() * 8
    }
}

pub trait RegistersTrait<R>
where
    R: RegEnumTrait,
{
    fn get(&self, r: &R) -> u64;
    fn set(&mut self, r: &R, v: u64);
}

pub trait FlagsTrait {
    fn le(self) -> bool;
    fn gt(self) -> bool ;
    fn lt(self) -> bool ;
    fn ge(self) -> bool ;
    fn hi(self) -> bool ;
    fn ls(self) -> bool ;
}
