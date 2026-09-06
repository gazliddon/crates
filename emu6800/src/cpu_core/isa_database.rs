use super::{
    flags_from_hnzvc, AddrModeEnum, Instruction, Isa, Mnemonic, OpcodeData, RegEnum, StatusReg,
};
use std::collections::{HashMap, HashSet};

use strum::{EnumIter, IntoEnumIterator};

#[derive(Debug, Clone)]
pub struct InstructionInfo<'a> {
    pub mnemonic: Mnemonic,
    pub addr_mode: AddrModeEnum,
    pub opcode_data: &'a OpcodeData,
    pub instruction: &'a Instruction,
}

impl<'a> InstructionInfo<'a> {
    pub fn get_mnemonic_text(&self) -> String {
        format!("{:?}", self.get_mnemonic()).to_lowercase()
    }
    pub fn get_mnemonic(&self) -> Mnemonic {
        self.mnemonic
    }
}

pub struct IsaDatabase {
    op_code_to_data: HashMap<usize, (Mnemonic, AddrModeEnum, OpcodeData)>,
    m_to_addr_modes: HashMap<Mnemonic, Instruction>,
    opcode_to_mnemonic: HashMap<String, Mnemonic>,
}

impl IsaDatabase {
    pub fn new(isa: &Isa) -> Self {
        // The v2 table carries flags/register/memory effects itself; the
        // older computed fallbacks are gone.
        let mut instructions = isa.instructions.clone();

        // Synthesise instructions for the MAME-verified undocumented
        // opcodes so every opcode maps to an InstructionInfo (the
        // executor's op_table and the dissassembler cover all 256).
        for op in &isa.undocumented {
            let (regs_written, memory_write) = match op.mnemonic {
                Mnemonic::StaIm => (regs(&[RegEnum::A]), true),
                Mnemonic::StbIm => (regs(&[RegEnum::B]), true),
                Mnemonic::StsIm => (regs(&[RegEnum::SP]), true),
                Mnemonic::StxIm => (regs(&[RegEnum::X]), true),
                Mnemonic::JsrUndoc => (HashSet::new(), true),
                _ => (HashSet::new(), false),
            };
            instructions.insert(
                op.mnemonic,
                Instruction {
                    flags_read: StatusReg::empty(),
                    flags_written: flags_from_hnzvc(&op.flags_hnzvc),
                    flags_hnzvc: op.flags_hnzvc.clone(),
                    flags_read_hnzvc: String::new(),
                    undocumented: true,
                    addr_modes: HashMap::from([(
                        AddrModeEnum::Inherent,
                        OpcodeData {
                            regs_read: HashSet::new(),
                            regs_written,
                            memory_read: false,
                            memory_write,
                            opcode: op.opcode,
                            cycles: op.cycles,
                            size: op.size,
                        },
                    )]),
                },
            );
        }

        let mut op_code_to_data = HashMap::new();
        let mut opcode_to_mnemonic = HashMap::new();

        for (m, a_modes) in instructions.iter() {
            for (amode, data) in a_modes.addr_modes.iter() {
                let v = (*m, *amode, data.clone());
                op_code_to_data.insert(data.opcode, v);
            }
        }

        for _m in Mnemonic::iter() {
            let text = format!("{_m:?}").to_lowercase();
            opcode_to_mnemonic.insert(text, _m);
        }

        Self {
            m_to_addr_modes: instructions,
            op_code_to_data,
            opcode_to_mnemonic,
        }
    }

    fn get_instruction_address_modes(&self, _m: Mnemonic) -> Option<&Instruction> {
        self.m_to_addr_modes.get(&_m)
    }

    pub fn get_opcode(&self, _name: &str) -> Option<&Instruction> {
        self.opcode_to_mnemonic
            .get(_name)
            .and_then(|m| self.get_instruction_address_modes(*m))
    }

    pub fn get_instruction_info_from_opcode(&self, op_code: usize) -> Option<InstructionInfo<'_>> {
        self.op_code_to_data
            .get(&op_code)
            .map(|(mnemonic, addr_mode, data)| {
                let instruction = self.m_to_addr_modes.get(mnemonic).unwrap();
                InstructionInfo {
                    mnemonic: *mnemonic,
                    addr_mode: *addr_mode,
                    opcode_data: data,
                    instruction,
                }
            })
    }
}

fn memory_access_for(mnemonic: Mnemonic, mode: AddrModeEnum) -> (bool, bool) {
    use AddrModeEnum::*;
    use Mnemonic::*;
    if !matches!(mode, Direct | Extended | Indexed) {
        return (false, false);
    }
    let write_only = matches!(mnemonic, StaA | StaB | Stx | Sts);
    let read_modify_write = matches!(
        mnemonic,
        Asl | Asr | Lsr | Rol | Ror | Clr | Com | Dec | Inc | Neg
    );
    (!write_only, write_only || read_modify_write)
}

/// Condition-code dependencies that are not explicit in the JSON table.  The
/// instruction descriptions historically recorded written flags only, which
/// made every read dependency appear empty to tools consuming the ISA.
fn flags_read_for(mnemonic: Mnemonic) -> StatusReg {
    use Mnemonic::*;
    match mnemonic {
        AdcA | AdcB | SbcA | SbcB => StatusReg::C,
        Daa => StatusReg::H | StatusReg::C,
        Bcc | Bcs => StatusReg::C,
        Beq | Bne => StatusReg::Z,
        Bge | Bgt | Ble | Blt => StatusReg::N | StatusReg::V | StatusReg::Z,
        Bhi | Bls => StatusReg::C | StatusReg::Z,
        Bmi | Bpl => StatusReg::N,
        Bvc | Bvs => StatusReg::V,
        _ => StatusReg::empty(),
    }
}

fn regs(values: &[RegEnum]) -> HashSet<RegEnum> {
    values.iter().copied().collect()
}

fn registers_for(mnemonic: Mnemonic, mode: AddrModeEnum) -> (HashSet<RegEnum>, HashSet<RegEnum>) {
    use AddrModeEnum::*;
    use Mnemonic::*;

    let indexed = matches!(mode, Indexed);
    let mut read = HashSet::new();
    let mut written = HashSet::new();
    if indexed {
        read.insert(RegEnum::X);
    }

    match mnemonic {
        LdaA => {
            written.insert(RegEnum::A);
        }
        LdaB => {
            written.insert(RegEnum::B);
        }
        Ldx => {
            written.insert(RegEnum::X);
        }
        Lds => {
            written.insert(RegEnum::SP);
        }
        StaA => {
            read.insert(RegEnum::A);
        }
        StaB => {
            read.insert(RegEnum::B);
        }
        Stx => {
            read.insert(RegEnum::X);
        }
        Sts => {
            read.insert(RegEnum::SP);
        }

        AddA | AdcA | SubA | SbcA | AndA | OraA | EorA => {
            read.insert(RegEnum::A);
            written.insert(RegEnum::A);
        }
        AddB | AdcB | SubB | SbcB | AndB | OraB | EorB => {
            read.insert(RegEnum::B);
            written.insert(RegEnum::B);
        }
        CmpA | BitA => {
            read.insert(RegEnum::A);
        }
        CmpB | BitB => {
            read.insert(RegEnum::B);
        }

        Aba | Sba => {
            read.extend(regs(&[RegEnum::A, RegEnum::B]));
            written.insert(RegEnum::A);
        }
        Cba => read.extend(regs(&[RegEnum::A, RegEnum::B])),
        Tab => {
            read.insert(RegEnum::A);
            written.insert(RegEnum::B);
        }
        Tba => {
            read.insert(RegEnum::B);
            written.insert(RegEnum::A);
        }

        Inx | Dex => {
            read.insert(RegEnum::X);
            written.insert(RegEnum::X);
        }
        Ins | Des => {
            read.insert(RegEnum::SP);
            written.insert(RegEnum::SP);
        }
        Tsx => {
            read.insert(RegEnum::SP);
            written.insert(RegEnum::X);
        }
        Txs => {
            read.insert(RegEnum::X);
            written.insert(RegEnum::SP);
        }
        Tpa => {
            read.insert(RegEnum::SR);
            written.insert(RegEnum::A);
        }
        Tap => {
            read.insert(RegEnum::A);
            written.insert(RegEnum::SR);
        }

        PshA => {
            read.extend(regs(&[RegEnum::A, RegEnum::SP]));
            written.insert(RegEnum::SP);
        }
        PshB => {
            read.extend(regs(&[RegEnum::B, RegEnum::SP]));
            written.insert(RegEnum::SP);
        }
        PulA => {
            read.insert(RegEnum::SP);
            written.extend(regs(&[RegEnum::A, RegEnum::SP]));
        }
        PulB => {
            read.insert(RegEnum::SP);
            written.extend(regs(&[RegEnum::B, RegEnum::SP]));
        }

        Jsr | Bsr => {
            read.insert(RegEnum::SP);
            written.extend(regs(&[RegEnum::SP, RegEnum::PC]));
        }
        Jmp => {
            written.insert(RegEnum::PC);
        }
        Rts => {
            read.insert(RegEnum::SP);
            written.extend(regs(&[RegEnum::SP, RegEnum::PC]));
        }
        Rti => {
            read.insert(RegEnum::SP);
            written.extend(regs(&[
                RegEnum::A,
                RegEnum::B,
                RegEnum::X,
                RegEnum::SP,
                RegEnum::PC,
                RegEnum::SR,
            ]));
        }
        Swi => {
            read.extend(regs(&[
                RegEnum::A,
                RegEnum::B,
                RegEnum::X,
                RegEnum::SP,
                RegEnum::SR,
            ]));
            written.extend(regs(&[RegEnum::SP, RegEnum::PC]));
        }
        Wai => {
            read.extend(regs(&[
                RegEnum::A,
                RegEnum::B,
                RegEnum::X,
                RegEnum::SP,
                RegEnum::PC,
                RegEnum::SR,
            ]));
            written.insert(RegEnum::SP);
        }

        AslA | AsrA | LsrA | RolA | RorA | ClrA | ComA | DecA | IncA | NegA => {
            read.insert(RegEnum::A);
            written.insert(RegEnum::A);
        }
        AslB | AsrB | LsrB | RolB | RorB | ClrB | ComB | DecB | IncB | NegB => {
            read.insert(RegEnum::B);
            written.insert(RegEnum::B);
        }
        Clc | Cli | Clv | Sec | Sei | Sev => {
            written.insert(RegEnum::SR);
        }
        Daa => {
            read.insert(RegEnum::A);
            written.insert(RegEnum::A);
        }
        _ => {}
    }

    (read, written)
}

#[cfg(test)]
mod tests {
    use super::IsaDatabase;
    use crate::cpu_core::{AddrModeEnum, Isa};

    fn database() -> IsaDatabase {
        let text = include_str!("../../resources/opcodes6800.json");
        let isa: Isa = serde_json::from_str(text).unwrap();
        IsaDatabase::new(&isa)
    }

    #[test]
    fn wai_has_the_documented_nine_cycles() {
        let db = database();
        let wai = db
            .get_opcode("wai")
            .unwrap()
            .get_opcode_data(AddrModeEnum::Inherent)
            .unwrap();
        assert_eq!(wai.opcode, 0x3e);
        assert_eq!(wai.cycles, 9);
        assert_eq!(wai.size, 1);
        assert!(db.get_opcode("wai").unwrap().flags_written.is_empty());
    }

    #[test]
    fn conditional_and_carry_instructions_expose_flag_reads() {
        // The v2 table carries its own read-flag data (the old computed
        // fallbacks are gone).  Bcc reads C, Daa reads H.
        let db = database();
        assert!(db
            .get_opcode("bcc")
            .unwrap()
            .flags_read
            .contains(crate::cpu_core::StatusReg::C));
        assert!(db
            .get_opcode("daa")
            .unwrap()
            .flags_read
            .contains(crate::cpu_core::StatusReg::H));
    }

    #[test]
    fn register_dependencies_are_populated() {
        // v2 semantics: LDA/STA list the accumulator as both read and
        // written; stack and PC effects are deliberately excluded.
        let db = database();
        let ldaa = db
            .get_opcode("ldaa")
            .unwrap()
            .get_opcode_data(AddrModeEnum::Immediate8)
            .unwrap();
        assert!(ldaa.regs_read.contains(&crate::cpu_core::RegEnum::A));
        assert!(ldaa.regs_written.contains(&crate::cpu_core::RegEnum::A));

        let staa = db
            .get_opcode("staa")
            .unwrap()
            .get_opcode_data(AddrModeEnum::Extended)
            .unwrap();
        // STAA stores to memory: the accumulator is read, not written.
        assert!(!staa.regs_written.contains(&crate::cpu_core::RegEnum::A));
        assert!(staa.memory_write);
    }

    #[test]
    fn operand_memory_effects_are_explicit() {
        let db = database();
        let ldaa = db
            .get_opcode("ldaa")
            .unwrap()
            .get_opcode_data(AddrModeEnum::Extended)
            .unwrap();
        assert!(ldaa.memory_read && !ldaa.memory_write);
        let staa = db
            .get_opcode("staa")
            .unwrap()
            .get_opcode_data(AddrModeEnum::Extended)
            .unwrap();
        assert!(!staa.memory_read && staa.memory_write);
        let inc = db
            .get_opcode("inc")
            .unwrap()
            .get_opcode_data(AddrModeEnum::Extended)
            .unwrap();
        assert!(inc.memory_read && inc.memory_write);
        let ldaa_imm = db
            .get_opcode("ldaa")
            .unwrap()
            .get_opcode_data(AddrModeEnum::Immediate8)
            .unwrap();
        assert!(!ldaa_imm.memory_read && !ldaa_imm.memory_write);
    }
}
