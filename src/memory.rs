
const MEMORY_SIZE: usize = 1024;

pub fn assert_memory_size(mem: &[u8]) -> bool {
    if mem.len() >= MEMORY_SIZE { return false; }
    true
}   

#[allow(dead_code)]
pub fn write_to_memory(mem: &mut [u8], addr: usize, val: u8) {
    assert!(addr < MEMORY_SIZE);
    mem[addr] = val;
}

#[allow(dead_code)]
pub fn read_from_memory(mem: &[u8], addr: usize) -> u8 {
    assert!(addr < MEMORY_SIZE);
    mem[addr]
}

#[allow(dead_code)]
pub fn reset_memory(mem: &mut [u8]) {
    mem.fill(0);
}

pub fn fetch_instruction(index: &mut u8, mem: &[u8]) -> (u8, u8){
    let i = (mem[*index as usize], mem[*index as usize + 1]);
    *index += 2;
    i
}