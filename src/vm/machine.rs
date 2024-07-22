use super::archi::{ Instruction, ProgramAddress, Value, CALLSTACK_SIZE, STACK_SIZE };

pub struct Machine {
    pub pc: ProgramAddress,
    pub sp: usize,
    pub fp: usize,
    pub acc: Value,
    pub stack: [Value; STACK_SIZE],
    pub call_stack: [ProgramAddress; CALLSTACK_SIZE],
}

#[derive(Debug)]
pub enum MachineErrorKind {
    CallStackOverflow,
    CallStackUnderflow,
    StackOverflow,
    StackUnderflow,
} 

#[derive(Debug)]
pub struct MachineError {
    kind: MachineErrorKind,
    pc: ProgramAddress,
}

impl Machine {
    pub fn new() -> Machine {
        Machine {
            pc: 0,
            sp: 0,
            fp: 0,
            acc: 0,
            stack: [0; STACK_SIZE],
            call_stack: [0; CALLSTACK_SIZE],
        }
    }

    pub fn push(&mut self, value: Value) -> Result<(), MachineError> {
        if self.sp >= STACK_SIZE - 1 {
            return Err(MachineError { kind: MachineErrorKind::StackOverflow, pc: self.pc })
        }
        self.stack[self.sp] = value;
        self.sp += 1;
        Ok(())
    }

    pub fn pop(&mut self) -> Result<Value, MachineError> {
        if self.sp == 0 {
            return Err(MachineError { kind: MachineErrorKind::StackUnderflow, pc: self.pc })
        }
        self.sp -= 1;
        Ok(self.stack[self.sp])
    }

    pub fn push_frame(&mut self, addr: ProgramAddress) -> Result<(), MachineError> {
        if self.fp >= CALLSTACK_SIZE - 1 {
            return Err(MachineError { kind: MachineErrorKind::CallStackOverflow, pc: self.pc })
        }
        self.call_stack[self.fp] = addr;
        self.fp += 1;
        Ok(())
    }

    pub fn pop_frame(&mut self) -> Result<ProgramAddress, MachineError> {
        if self.fp == 0 {
            return Err(MachineError { kind: MachineErrorKind::CallStackUnderflow, pc: self.pc })
        }
        self.fp -= 1;
        Ok(self.call_stack[self.fp])
    }

    pub fn execute_instruction(&mut self, instruction: Instruction) -> Result<(), MachineError> {
        match instruction {
            Instruction::Halt | Instruction::Noop => {},
            Instruction::LoadImmediate(n) => self.acc = n as i32,
            Instruction::Push => self.push(self.acc)?,
            Instruction::Pop => self.acc = self.pop()?,
            Instruction::Dup => self.push(self.stack[self.sp])?,
            Instruction::Swap => {
                let tmp = self.stack[self.sp];
                self.stack[self.sp] = self.stack[self.sp - 1];
                self.stack[self.sp - 1] = tmp;
            }
            Instruction::LoadTop => self.acc = self.stack[self.sp],
            Instruction::Over => self.acc = self.stack[self.sp - 1],
            Instruction::Inc => self.acc += 1,
            Instruction::Dec => self.acc -= 1,
            Instruction::Add => self.acc += self.pop()?,
            Instruction::Sub => self.acc -= self.pop()?,
            Instruction::Mul => self.acc *= self.pop()?,
            Instruction::Div => self.acc /= self.pop()?,
            Instruction::Eq => self.acc = if self.acc == self.pop()? { 1 } else { 0 },
            Instruction::Neq =>  self.acc = if self.acc != self.pop()? { 1 } else { 0 },
            Instruction::Lt =>  self.acc = if self.acc < self.pop()? { 1 } else { 0 },
            Instruction::Lte =>  self.acc = if self.acc <= self.pop()? { 1 } else { 0 },
            Instruction::Gt =>  self.acc = if self.acc > self.pop()? { 1 } else { 0 },
            Instruction::Gte =>  self.acc = if self.acc >= self.pop()? { 1 } else { 0 },
            Instruction::Inv => self.acc = if self.acc == 1 { 0 } else { 1 },
            Instruction::Jmp(addr) => {
                self.pc = addr;
                return Ok(());
            }
            Instruction::JumpIfZero(addr) => {
                if self.acc == 0 {
                    self.pc = addr;
                    return Ok(());
                }
            }
            Instruction::JumpIfNotZero(addr) => {
                if self.acc != 0 {
                    self.pc = addr;
                    return Ok(());
                }
            }
            Instruction::Call(addr) => {
                self.push_frame(self.pc)?;
                self.pc = addr;
                return Ok(());
            }
            Instruction::Ret => {
                self.pc = self.pop_frame()?;
            }
        }
        self.pc += 1;
        Ok(())
    }

    pub fn run(&mut self, code: &[Instruction]) -> Result<(), MachineError> {
        self.pc = 0;
        while (self.pc as usize) < code.len() && code[self.pc as usize] != Instruction::Halt {
            self.execute_instruction(code[self.pc as usize])?
        }
        Ok(())
    }
}