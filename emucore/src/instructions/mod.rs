////////////////////////////////////////////////////////////////////////////////
pub trait InstructionDbaseTrait<INF> {
    fn from_text(json_str: &str) -> Self;
    fn get_opcode_info_from_name(&self, input: &str) -> Option<&INF>;
    fn get_opcode_info_from_opcode(&self, opcode: usize) -> Option<&INF>;
    fn generate_source(&self) -> String;
}

// A = Addressing mode enum
pub trait InstructionInfoTrait<A, I>
where
    A: PartialEq + Eq + std::hash::Hash + Copy + Clone,
{
    fn supports_addr_mode(&self, m: A) -> bool;
    fn get_instruction(&self, amode: A) -> Option<&I>;
}

////////////////////////////////////////////////////////////////////////////////
