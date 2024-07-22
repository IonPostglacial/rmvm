use core::str;
use std::collections::{hash_map::Entry, HashMap};

use super::archi::{Immediate, Instruction, ProgramAddress};
use std::ops::Range;

#[derive(Debug)]
pub enum AssemblyErrorKind {
    UnknownInstruction(String),
    WrongArity { expected: usize, got: usize },
    InvalidNumber(String),
    CodeTooBig,
}

#[derive(Debug)]
pub struct AssemblyError {
    pub kind: AssemblyErrorKind,
    pub line: usize,
}

fn parse_immediate(operand: &str, linum: usize) -> Result<Immediate, AssemblyError> {
    operand.parse::<Immediate>().map_err(|_| AssemblyError {
        kind: AssemblyErrorKind::InvalidNumber(operand.to_string()),
        line: linum,
    })
}

struct Fix {
    address: Option<ProgramAddress>,
    to_fix: Vec<ProgramAddress>,
}

impl Fix {
    fn new(address: Option<ProgramAddress>) -> Fix {
        Fix {
            address,
            to_fix: Vec::new(),
        }
    }

    fn new_with_fix(address: Option<ProgramAddress>, fix: ProgramAddress) -> Fix {
        Fix {
            address,
            to_fix: vec![fix],
        }
    }
}

struct Tokenizer {
    char_index: usize,
    last_token: usize,
    line_num: usize,
}

impl Tokenizer {
    fn consume_comment(&mut self, source: &[u8]) {
        while self.char_index < source.len() && source[self.char_index] != b'\n' {
            self.char_index += 1;
        }
    }

    fn next_token(&mut self, source: &[u8]) -> Range<usize> {
        while self.char_index < source.len() - 1
            && (self.char_index < self.last_token
                || (source[self.char_index] != b' ' && source[self.char_index] != b'\n'))
        {
            self.char_index += 1;
        }
        let tok = self.last_token..self.char_index;
        self.last_token = self.char_index + 1;
        if self.last_token >= source.len() {
            self.char_index = self.last_token;
        }
        tok
    }

    fn next_token_slice<'a>(&mut self, source: &'a [u8]) -> &'a str {
        str::from_utf8(&source[self.next_token(source)]).expect("valid utf8")
    }

    fn parse_address<'a>(
        &self,
        operand: &'a str,
        inst_count: ProgramAddress,
        fixes_by_label: &mut HashMap<&'a str, Fix>,
    ) -> Result<ProgramAddress, AssemblyError> {
        if operand.starts_with('@') {
            let label = &operand[1..operand.len()];
            match fixes_by_label.get_mut(label) {
                None => {
                    fixes_by_label.insert(label, Fix::new_with_fix(None, inst_count));
                    return Ok(0);
                }
                Some(fix) => {
                    fix.to_fix.push(inst_count);
                    return Ok(fix.address.unwrap_or_default());
                }
            }
        } else {
            operand
                .parse::<ProgramAddress>()
                .map_err(|_| AssemblyError {
                    kind: AssemblyErrorKind::InvalidNumber(operand.to_string()),
                    line: self.line_num,
                })
        }
    }
}

pub fn code_from_str(src: &str, dst: &mut [Instruction]) -> Result<ProgramAddress, AssemblyError> {
    let mut tokenizer = Tokenizer {
        char_index: 0,
        last_token: 0,
        line_num: 0,
    };
    let mut inst_count: ProgramAddress = 0;
    let mut fixes_by_label: HashMap<&str, Fix> = HashMap::new();
    let source = src.as_bytes();

    while tokenizer.char_index < source.len() {
        match source[tokenizer.char_index] {
            b';' => tokenizer.consume_comment(source),
            b' ' | b'\n' => {
                let cmd = &source[tokenizer.next_token(source)];
                dst[inst_count as usize] = match cmd {
                    b"" => { continue; },
                    b"halt" => Instruction::Halt,
                    b"noop" => Instruction::Noop,
                    b"load" => Instruction::LoadImmediate(parse_immediate(
                        tokenizer.next_token_slice(source),
                        tokenizer.line_num,
                    )?),
                    b"push" => Instruction::Push,
                    b"pop" => Instruction::Pop,
                    b"dup" => Instruction::Dup,
                    b"swap" => Instruction::Swap,
                    b"ldt" => Instruction::LoadTop,
                    b"over" => Instruction::Over,
                    b"inc" => Instruction::Inc,
                    b"dec" => Instruction::Dec,
                    b"add" => Instruction::Add,
                    b"sub" => Instruction::Sub,
                    b"mul" => Instruction::Mul,
                    b"div" => Instruction::Div,
                    b"eq" => Instruction::Eq,
                    b"neq" => Instruction::Neq,
                    b"lt" => Instruction::Lt,
                    b"lte" => Instruction::Lte,
                    b"gt" => Instruction::Gt,
                    b"gte" => Instruction::Gte,
                    b"inv" => Instruction::Inv,
                    b"jmp" => {
                        let addr = tokenizer.next_token_slice(source);
                        Instruction::Jmp(tokenizer.parse_address(
                            addr,
                            inst_count,
                            &mut fixes_by_label,
                        )?)
                    }
                    b"jz" => {
                        let addr = tokenizer.next_token_slice(source);
                        Instruction::JumpIfZero(tokenizer.parse_address(
                            addr,
                            inst_count,
                            &mut fixes_by_label,
                        )?)
                    }
                    b"jnz" => {
                        let addr = tokenizer.next_token_slice(source);
                        Instruction::JumpIfNotZero(tokenizer.parse_address(
                            addr,
                            inst_count,
                            &mut fixes_by_label,
                        )?)
                    }
                    b"call" => {
                        let addr = tokenizer.next_token_slice(source);
                        Instruction::Call(tokenizer.parse_address(
                            addr,
                            inst_count,
                            &mut fixes_by_label,
                        )?)
                    }
                    b"ret" => Instruction::Ret,
                    label if label.len() > 0 && label[label.len() - 1] == b':' => {
                        let entry = fixes_by_label
                            .entry(str::from_utf8(&label[0..label.len() - 1]).expect("valid utf8"));
                        match entry {
                            Entry::Occupied(ent) => {
                                ent.into_mut().address = Some(inst_count);
                            }
                            Entry::Vacant(ent) => {
                                let key = ent.into_key();
                                fixes_by_label.insert(key, Fix::new(Some(inst_count)));
                            }
                        }
                        continue;
                    }
                    inst => Err(AssemblyError {
                        kind: AssemblyErrorKind::UnknownInstruction(
                            String::from_utf8(inst.to_vec()).expect("valid utf8"),
                        ),
                        line: tokenizer.line_num,
                    })?,
                };
                inst_count += 1;
                tokenizer.line_num += 1;
            }
            _ => {
                tokenizer.char_index += 1;
            }
        }
    }
    for (_, fix) in fixes_by_label {
        match fix.address {
            None => todo!("handle error missing label definition"),
            Some(address) => {
                for to_fix in fix.to_fix {
                    let addr = to_fix as usize;
                    match dst[addr] {
                        Instruction::Jmp(_) => dst[addr] = Instruction::Jmp(address),
                        Instruction::JumpIfZero(_) => dst[addr] = Instruction::JumpIfZero(address),
                        Instruction::JumpIfNotZero(_) => {
                            dst[addr] = Instruction::JumpIfNotZero(address)
                        }
                        Instruction::Call(_) => dst[addr] = Instruction::Call(address),
                        _ => todo!("handle error instruction address unsupported"),
                    }
                }
            }
        }
    }
    Ok(inst_count)
}
