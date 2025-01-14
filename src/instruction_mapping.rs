pub mod instruction_utils {
    use crate::cpu::cpu_state::NUM_REGS;


    pub enum InstructionNameMap {
        Instruction(u8),
        Label(String),
    }
    
    pub fn get_upcodes(name: &str) -> Result<InstructionNameMap, String> {
        match name {
            "LDI"   => Ok(InstructionNameMap::Instruction(0b0000_0000)),  // 0
            "LD"    => Ok(InstructionNameMap::Instruction(0b0001_0000)),  // 16
            "ST"    => Ok(InstructionNameMap::Instruction(0b0010_0000)),  // 32
            "MOV"   => Ok(InstructionNameMap::Instruction(0b0011_0000)),  // 48
            "ADD"   => Ok(InstructionNameMap::Instruction(0b0100_0000)),  // 64
            "SUB"   => Ok(InstructionNameMap::Instruction(0b0101_0000)),  // 80
            "MUL"   => Ok(InstructionNameMap::Instruction(0b0110_0000)),  // 96
            "ADDI"  => Ok(InstructionNameMap::Instruction(0b0111_0000)),  // 112
            "AND"   => Ok(InstructionNameMap::Instruction(0b1000_0000)),  // 128
            "OR"    => Ok(InstructionNameMap::Instruction(0b1001_0000)),  // 144
            "XOR"   => Ok(InstructionNameMap::Instruction(0b1010_0000)),  // 160
            "NOT"   => Ok(InstructionNameMap::Instruction(0b1011_0000)),  // 176
            "JMPZ"  => Ok(InstructionNameMap::Instruction(0b1100_0000)),  // 192
            "RET"   => Ok(InstructionNameMap::Instruction(0b1101_0000)),   // 208
            "CALL"  => Ok(InstructionNameMap::Instruction(0b1110_0000)),   // 224
            "HLT"   => Ok(InstructionNameMap::Instruction(0b1111_0000)),  // 240
            _ => {
                if name.ends_with(':') {
                    let lab = name.trim_end_matches(':');
                    return Ok(InstructionNameMap::Label(String::from(lab)));
                }
                // Handle the case where the instruction name does not exist
                panic!("Error: Instruction '{}' does not exist.", name);
            }
        }
    }

    pub fn map_register_to_value(reg: &str) -> u8 {
        if reg.starts_with('r') {
            if let Ok(val) = reg.strip_prefix("r").unwrap().parse::<u8>() {
                if val < NUM_REGS as u8 {
                    return val;
                }
            }
        } else if let Ok(imm) = reg.parse::<u8>() {
            return imm;
        }
        panic!("Error: Unidentified register name or immidiate value: {}. ", reg);
    }
        
}