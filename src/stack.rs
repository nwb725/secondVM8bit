pub struct Stack {
    pub stack: [u8; 64],
    pub top:   usize,
}

impl Stack {
    pub fn create_stack() -> Self {
        Self {
            stack: [0; 64],
            top:   0,
        }
    }

    pub fn stack_push(&mut self, elm: u8) -> Result<(), &'static str> {
        if self.top == self.stack.len() {
            return Err("Error: Stack overflow!")
        } 
        self.stack[self.top] = elm;
        self.top += 1;
        Ok(())
    }

    pub fn stack_pop(&mut self) -> Result<u8, &'static str> {
        if self.top == 0 {
            return Err("Error: Tried to pop from an empty stack.");
        }
        self.top -= 1;
        Ok(self.stack[self.top])

    }
}
