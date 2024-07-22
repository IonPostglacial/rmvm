pub type ProgramAddress = u16;
pub type Value = i32;
pub type Immediate = i16;

pub const STACK_SIZE: usize = 16000;
pub const CALLSTACK_SIZE: usize = 100;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Instruction {
    Halt,
    Noop,
    LoadImmediate(Immediate),
    Push,
    Pop,
    Dup,
    Swap,
    LoadTop,
    Over,
    Inc,
    Dec,
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Neq,
    Lt,
    Lte,
    Gt,
    Gte,
    Inv,
    Jmp(ProgramAddress),
    JumpIfZero(ProgramAddress),
    JumpIfNotZero(ProgramAddress),
    Call(ProgramAddress),
    Ret,
}
