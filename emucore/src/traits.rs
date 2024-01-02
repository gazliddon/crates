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

////////////////////////////////////////////////////////////////////////////////
pub trait FlagsTrait {
    fn le(self) -> bool;
    fn gt(self) -> bool;
    fn lt(self) -> bool;
    fn ge(self) -> bool;
    fn hi(self) -> bool;
    fn ls(self) -> bool;
}

////////////////////////////////////////////////////////////////////////////////
pub trait SingleInstructionTrait<A>
where
    A: PartialEq + Eq + std::hash::Hash + Copy + Clone,
{
    fn get_addressing_mode(&self) -> A;
    fn get_cycles(&self) -> usize;
    fn get_bytes(&self) -> usize;
}

pub trait InstructionDbaseTrait<A, I, INF>
where
    A: PartialEq + Eq + std::hash::Hash + Copy + Clone,
    I: SingleInstructionTrait<A>,
    INF: InstructionInfoTrait<A, I>,
{
    fn from_text(json_str: &str) -> Self;
    fn from_filename(file_name: &str) -> Self;
    fn from_data(instructions: Vec<I>, unknown: I) -> Self;
    fn get_opcode_info(&self, input: &str) -> Option<&INF>;
    fn get_opcode_info_from_opcode(&self, opcode: usize) -> Option<&INF>;
}

// Trait for a CPU instruction and all of the addr modes it supports
pub trait InstructionInfoTrait<A, I>
where
    A: PartialEq + Eq + std::hash::Hash + Copy + Clone,
    I: SingleInstructionTrait<A>,
{
    fn supports_addr_mode(&self, m: A) -> bool;
    fn get_instruction(&self, amode: A) -> Option<&I>;
}
