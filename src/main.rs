mod assembler;
mod memory;
mod instruction_mapping;
mod cpu;
mod symtab;
mod stack;
mod parser;
mod yoloheap;
mod byte_utils;
use assembler::init_program_in_memory;
use cpu::cpu_state::execute;

fn main() {
    println!("Enter program to run: ");
    let mut in_buf = String::new();
    match std::io::stdin().read_line(&mut in_buf) {
        Ok(_)  => { in_buf = in_buf.trim().to_string() },
        Err(e) => println!("{e}"),
    }
    let mut mem = init_program_in_memory(&in_buf);
    execute(&mut mem);
}
