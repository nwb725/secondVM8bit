use crate::assembler::InstructionTokenized;

const VALID_NAME_TOKENS: [&str; 16]  = ["LDI", "LD", "ST", "MOV", 
                                        "ADD", "SUB", "MUL", "ADDI", 
                                        "AND", "OR", "XOR", "NOT", 
                                        "JMPZ", "RET", "CALL", "HLT"];

const VALID_ARGUMENT_TOKENS: [&str; 16] = ["r0", "r1", "r2", 
                                           "r3", "r4", "r5", 
                                           "r6", "r7", "r8", 
                                           "r9", "r10", "r11", 
                                           "r12", "r13", "r14", "r15"];

pub fn is_valid_instruction(inst: &InstructionTokenized) -> Result<(), &'static str> {
    
    // Validate name:
    let name = match &inst.name {
        Some(n) => n,
        None    => return Err("Error: Name was None, should never happen.")
    };

    // Its not an instruction, must be function label.
    if !VALID_NAME_TOKENS.contains(&name.as_str()) {
        if name.ends_with(":") && name.starts_with("_") {
            return Ok(());
        } else {
            inst.print_instruction_tokenized(0);
            return Err("Error: Instruction name does not exist, and is not a function label.");
        }
    }

    // It is an instruction.
    match (&inst.arg1, &inst.arg2) {
        (Some(arg1), Some(arg2)) => {
            if VALID_ARGUMENT_TOKENS.contains(&arg1.as_str()) {
                // Both args are used, arg1 has to be a register.
                if VALID_ARGUMENT_TOKENS.contains(&arg2.as_str()) {
                    // Case arg1 and arg2 are register its ok.
                    Ok(())
                } else if arg2.parse::<u8>().is_ok() {
                    // If arg1 is register and arg2 is immidiate.
                    return Ok(());
                } else {
                    inst.print_instruction_tokenized(2);
                    return Err("Error: Arg2 is not valid.");
                }
            } else {
                // Arg1 is always a register if the instruction uses two args.
                inst.print_instruction_tokenized(2);
                Err("Error: Arg1 is not valid")
            }
        },
        (None, None) => Ok(()),

        // Carefull, because if only arg1 is there it can be reg, imm or label.
        (Some(arg1), None) => {
            if arg1.parse::<u8>().is_ok() || arg1.starts_with("_") || VALID_ARGUMENT_TOKENS.contains(&arg1.as_str())  {
                Ok(())
            } else {
                inst.print_instruction_tokenized(1);
                Err("Error: Instruction had 1 argument, but the argument was invalid.")
            }
        },

        (None, Some(_)) =>  { 
            inst.print_instruction_tokenized(2);
            Err("Error: Has arg2 but not arg1") 
        }
    }
}   