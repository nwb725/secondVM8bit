const SYMTAB_INIT_SIZE: usize = 10;

pub struct Function {
    pub label:   String,
    pub address: u8,
}

impl Function {
    pub fn new(label: String, address: u8) -> Self {
        Self {
            label,
            address,
        }
    }
}

pub struct SymTab {
    pub table: Vec<Function>,
}

impl SymTab {
    pub fn new() -> Self {
        Self {
            table: Vec::with_capacity(SYMTAB_INIT_SIZE),
        }
    }

    pub fn symtab_lookup(&self, target: &str) -> Result<u8, String> {
        if let Some(lab) = self.table.iter().find(|x| x.label == target) {
            return Ok(lab.address)
        } 
        Err(format!("Error: Could not find the target function: LABEL = '{}'", target))
    } 
    
    pub fn symtab_insert(&mut self, f: Function) -> Result<(), String>{
        // Check if it already exists.
        if self.table.iter().any(|fun| fun.label == f.label) {
            return Err(format!("Error: Function declared twice at: addr<{}>", f.address));
        }
        self.table.push(f);
        Ok(())
    }

    #[allow(dead_code)]
    pub fn print_symtab(&self) {
        let tab = self.table
            .iter()
            .enumerate();
        
        println!("i: | addr | label |");
        for (i, f) in tab {
            println!("_______________");
            println!("{}: | {} | {} |",i, f.address, f.label);
        }
        println!("_______________");
    }
}
