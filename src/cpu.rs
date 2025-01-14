pub mod cpu_state {

    use crate::memory;
    use crate::stack::Stack;
    #[allow(unused_imports)]
    use crate::yoloheap::Heap;

    //const MAX_HEAP_SIZE: usize = 256;
    pub const NUM_REGS: usize = 16;
    pub struct CpuState {
        registers: [u8; NUM_REGS],
        pc:        u8,
        running:   bool,
        //heap:   Heap,
    }

    impl CpuState {
        pub fn new_state() -> Self {
            Self {
                registers: [0; NUM_REGS],
                pc:        0,
                running:   true,
                //heap:      Heap::new_heap(MAX_HEAP_SIZE)
            }
        }
    }

    struct DecodedInstruction {
        pub upcode:  u8,
        pub arg1:    u8,
        pub arg2:    u8,
    }

    impl DecodedInstruction {
        pub fn multibyte_decode(instr: &(u8, u8)) -> Self {
            Self {
                upcode:  (instr.0 >> 4), 
                arg1:    (instr.0 & 0xf),
                arg2:    (instr.1),
            }
        }
    }

    fn execute_instruction(instr: &(u8, u8), state: &mut CpuState, stack: &mut Stack, mem: &mut [u8]) {
        let inst = DecodedInstruction::multibyte_decode(instr);
    
        match inst.upcode {
            0x0 => {
                // LDI: Load Immediate into register.
                println!("LDI r{} {}", inst.arg1, inst.arg2);
                state.registers[inst.arg1 as usize] = inst.arg2;
            }
            0x1 => {
                // LD: Load from Memory
                println!("LD r{} r{}", inst.arg1, inst.arg2);
                state.registers[inst.arg1 as usize] = 
                    memory::read_from_memory(mem, state.registers[inst.arg2 as usize] as usize);
            }
            0x2 => {
                // ST: Store to Memory
                println!("ST r{} r{}", inst.arg1, inst.arg2);
                memory::write_to_memory(
                    mem, 
                    state.registers[inst.arg1 as usize] as usize,
                    state.registers[inst.arg2 as usize]);
            }
            0x3 => {
                // MOV: Move Data, set reg[r1] = reg[r2]
                println!("MOV r{} r{}", inst.arg1, inst.arg2);
                state.registers[inst.arg1 as usize] = state.registers[inst.arg2 as usize];
            }
            0x4 => {
                // ADD: Add value in r1 with r2, place in r1.
                println!("ADD r{} r{}", inst.arg1, inst.arg2);
                let _t = state.registers[inst.arg1 as usize].checked_add(state.registers[inst.arg2 as usize]).expect("Overflow happened in add.");
                println!("R1: {}, R2 (RESULT): {}", state.registers[inst.arg1 as usize], state.registers[inst.arg2 as usize]);
                state.registers[inst.arg1 as usize] = _t;
            }
            0x5 => {
                // SUB: Subtract
                println!("SUB r{} r{}", inst.arg1, inst.arg2);
                println!("reg1: {},  reg2: {}", state.registers[inst.arg1 as usize], state.registers[inst.arg2 as usize]);
                assert!(state.registers[inst.arg1 as usize] >= state.registers[inst.arg2 as usize]);
                state.registers[inst.arg1 as usize] -= state.registers[inst.arg2 as usize];
            }
            0x6 => {
                // MUL: Multiply
                println!("MUL r{} r{}", inst.arg1, inst.arg2);
                state.registers[inst.arg1 as usize] *= state.registers[inst.arg2 as usize];
            }
            0x7 => {
                // ADDI: Add Immediate
                println!("ADDI r{} {}", inst.arg1, inst.arg2);
                state.registers[inst.arg1 as usize] += inst.arg2;
            }
            0x8 => {
                // AND: Bitwise AND
                println!("AND r{} r{}", inst.arg1, inst.arg2);
                state.registers[inst.arg1 as usize] &= state.registers[inst.arg2 as usize];
            }
            0x9 => {
                // OR: Bitwise OR
                println!("OR r{} r{}", inst.arg1, inst.arg2);
                state.registers[inst.arg1 as usize] |= state.registers[inst.arg2 as usize];
            }
            0xA => {
                // XOR: Bitwise XOR
                println!("XOR r{} r{}", inst.arg1, inst.arg2);
                state.registers[inst.arg1 as usize] ^= state.registers[inst.arg2 as usize];
            }
            0xB => {
                // NOT: Bitwise NOT
                println!("NOT r{}", inst.arg1);
                state.registers[inst.arg1 as usize] = !state.registers[inst.arg1 as usize];
            }
            0xC => {
                // JMPZ: Jump to r1 if r2 is zero
                println!("JMPZ r{} r{}", inst.arg1, inst.arg2);
                if state.registers[inst.arg2 as usize] == 0 {
                    state.pc = state.registers[inst.arg1 as usize] - 2;
                }
            }
            0xD => {
                // RET: Return to return address
                println!("RET");
                if let Ok(ret) = stack.stack_pop() {
                    state.pc = ret;
                } else {
                    panic!("Error in CPU: Stack pop failed in RET.");
                }
            }
            0xE => {
                // CALL: Calls a function
                println!("CALL {}", inst.arg2);
                if let Err(e) = stack.stack_push(state.pc) {
                    panic!("{e}");
                }
                state.pc = inst.arg2;
            }
            0xF => {
                // HLT - Halts the program.
                println!("HLT");
                state.running = false;
            }
            _ => {
                panic!("Error: Unknown opcode: {:#X}", inst.upcode);
            }
        }   
    }

    pub fn execute(mem: &mut [u8]) {
        let mut state = CpuState::new_state();
        let mut stack = Stack::create_stack();
        while state.running {  
            let i = memory::fetch_instruction(&mut state.pc, mem);
            execute_instruction(&i, &mut state, &mut stack, mem);
        }
        println!("Fib(n) = {}", state.registers[2]);
        
    }
}




