use std::fs;
use crate::instruction_mapping::instruction_utils::{self, get_upcodes, map_register_to_value};
use crate::memory;
use crate::parser;
use crate::symtab::{SymTab, Function};

// const PROGRAM_ENTRY: &str = "_start";
pub struct InstructionTokenized {
    pub name: Option<String>,
    pub arg1: Option<String>,
    pub arg2: Option<String>,
}

impl InstructionTokenized {
    pub fn print_instruction_tokenized(&self, n: u8) {

        let name = match &self.name {
            Some(n) => n.to_string(),
            None    => String::from("-"),
        };

        let arg1 = match &self.arg1 {
            Some(a1) => a1.to_string(),
            None     => String::from("-"),
        };

        let arg2 = match &self.arg2 {
            Some(a2) => a2.to_string(),
            None     => String::from("-"),
        };

        match n {
            0 => println!("<{}>", name),
            1 => println!("<{} {}>", name, arg1),
            2 => println!("<{} {} {}>", name, arg1, arg2),
            _ => println!("Error: Printing <{}> number of arguments is not valid.", n),
        }
    }
}


// Prints the values in a vector.
#[allow(dead_code)]
pub fn print_vec(v: &[String]) {
    for i in v {
        println!("{}", i);
    }
}

// Reads a programfile and puts the instructions in a vector.
fn read_and_parse_programfile(file: &str) -> Vec<String> {
    let fc = fs::read_to_string(file)
                .expect("Could not open file :(");
    
    let mut instructions = Vec::<String>::new();

    for instr in fc.lines() {

        let code = instr.split(';').next().unwrap_or("").trim();

        // Trim whitespace and check if the line is empty
        if !code.trim().is_empty() {
            instructions.push(String::from(code));
        }
    }

    // Check if entry point is set.
    if !instructions.contains(&String::from("_START:")) {
        panic!("Error: Program has no entry point.");
    }

    instructions
}

// HELPER FUNCTION - DO NOT USE!
// Takes an instruction as a string and splits it into a tokens.
fn tokenize_helper(inst: String) -> InstructionTokenized {
    let mut token = InstructionTokenized {
        name: None,
        arg1: None,
        arg2: None,
    };

    let mut split = inst.split_whitespace();
    token.name = split.next().map(|s| s.to_string());
    token.arg1 = split.next().map(|s| s.to_string());
    token.arg2 = split.next().map(|s| s.to_string());

    token
}

// Takes a vector of instructions and returns a vector of tokenized instructions.
pub fn tokenize_instructions(insts: Vec<String>) -> Result<Vec<InstructionTokenized>, &'static str>  {
    let tokens: Vec<InstructionTokenized> = insts.into_iter().map(tokenize_helper).collect();

    // Validate the instructions.
    for t in &tokens {
        parser::is_valid_instruction(t)?;
    }

    Ok(tokens)
}

// Takes a instruction token and returns a tuple of the first and second byte
// as u8 values.
fn token_to_value(token: &InstructionTokenized, symtab: &mut SymTab) -> Result<(u8, u8), String> {
    let mut res = (0, 0);
    
    // Match on the instruction name and get its upcode
    match token.name.as_ref() {
        Some(name) => {
            if let Ok(n) = get_upcodes(name) {
                match n {
                    instruction_utils::InstructionNameMap::Instruction(upcode) => {
                        res.0 += upcode;
                    }
                    _ => {
                        return Err(format!("Error: Name '{}' was not an instruction.", name));
                    }
                }
            } else {
                return Err(format!("Error: Could not retrieve upcode for '{}'.", name));
            }
        },
        None => return Err("Error: Name is none".into()),
    }

    // Handle the arguments
    match (&token.arg1, &token.arg2) {
        (Some(a1), Some(a2)) => { 
            res.0 += map_register_to_value(a1);
            res.1 += map_register_to_value(a2);
        },
        (Some(a1), None) => {
            match token.name.as_ref() {
                Some(name) if name == "CALL" => { 
                    match symtab.symtab_lookup(a1) {
                        Ok(f) => res.1 += f,
                        Err(_) => {
                            symtab.print_symtab();
                            return Err(format!("Error: Symbol '{}' not found in symtab.", a1));
                        }
                    }
                }
                _ => {
                    res.0 += map_register_to_value(a1);
                }
            }
        },
        (None, None) => (),
        _ => return Err("Error: Token had only a second arg".into()),
    }

    Ok(res)
}


// Takes a vector of instruction tokens and returns the memory
fn write_tokens_to_mem(insts: Vec<InstructionTokenized>, symtab: &mut SymTab) -> Result<Vec<u8>, &'static str> {
    let mut current_address = 0;

    // Filter out labels and add them to the symtab
    let without_labels: Vec<&InstructionTokenized> = insts.iter()
    .filter(|t| {
        if let Some(name) = &t.name {
            if let Ok(n) = get_upcodes(name) {
                match n {
                    instruction_utils::InstructionNameMap::Label(l) => {
                        let fun = Function::new(l, current_address);
                        if let Err(e) = symtab.symtab_insert(fun) {
                            panic!("{e}");
                        }
                        false // Exclude labels
                    }
                    _ => {
                        current_address += 2; // Assume 2 bytes per instruction
                        true // Keep non-label instructions
                    }
                }
            } else {
                panic!("Error: Getting upcode with name: {}", name);
            }
        } else {
            panic!("Error: Empty instruction, name is None.");
        }
    })
    .collect();

    // Translate the instructions into memory values
    let mem: Vec<u8> = without_labels.into_iter()
        .map(|token| {
            token_to_value(token, symtab)
                .unwrap_or_else(|err| panic!("{err}"))  // Unwrap or panic on token-to-value failure
        })
        .flat_map(|(e1, e2)| vec![e1, e2])
        .collect();

    // Validate the memory
    if memory::assert_memory_size(&mem) {
        Ok(mem)
    } else {
        Err("Error: Program exceeded maximum size.")
    }
}


pub fn init_program_in_memory(file: &str) -> Vec<u8>{
    let mut symtab = SymTab::new(); 
    let parsed_prg = read_and_parse_programfile(file);
    let tokens = tokenize_instructions(parsed_prg);

    //Checking if program is valid.
    match tokens {
        Ok(t) => {
            match write_tokens_to_mem(t, &mut symtab) {
                Ok(mem) => mem,
                Err(e)  => { panic!("{e}") }
            }
        },
        Err(e) => { panic!("{e}") }
    }
}




