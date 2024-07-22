use std::collections::{hash_map::Entry, HashMap};

use super::archi::{Immediate, Instruction, ProgramAddress};

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

fn assert_operand_exists(op: Option<&str>, linum: usize) -> Result<&str, AssemblyError> {
    op.ok_or(AssemblyError {
        kind: AssemblyErrorKind::WrongArity {
            expected: 1,
            got: 0,
        },
        line: linum,
    })
}

fn parse_immediate(op: Option<&str>, linum: usize) -> Result<Immediate, AssemblyError> {
    let operand = assert_operand_exists(op, linum)?;
    operand.parse::<Immediate>().map_err(|_| AssemblyError {
        kind: AssemblyErrorKind::InvalidNumber(operand.to_string()),
        line: linum,
    })
}

fn parse_address(
    op: Option<&str>,
    linum: usize,
    inst_count: ProgramAddress,
    addresses_by_label: &mut HashMap<String, Fix>,
) -> Result<ProgramAddress, AssemblyError> {
    let operand = assert_operand_exists(op, linum)?;
    if operand.starts_with('@') {
        let label = &operand[1..operand.len()];
        match addresses_by_label.get_mut(label) {
            None => {
                addresses_by_label.insert(label.to_string(), Fix::new_with_fix(None, inst_count));
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
                line: linum,
            })
    }
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

pub fn code_from_str(src: &str, dst: &mut [Instruction]) -> Result<ProgramAddress, AssemblyError> {
    let mut fixes_by_label: HashMap<String, Fix> = HashMap::new();
    let mut inst_count: ProgramAddress = 0;
    for (linum, line) in src.lines().enumerate() {
        let line = line.trim();
        if line.starts_with(';') {
            continue;
        }
        if inst_count >= (dst.len() as ProgramAddress) {
            return Err(AssemblyError {
                kind: AssemblyErrorKind::CodeTooBig,
                line: linum,
            });
        }
        let mut words = line.split_whitespace();
        match words.nth(0) {
            None => {}
            Some(s) => {
                dst[inst_count as usize] = match s {
                    "halt" => Instruction::Halt,
                    "noop" => Instruction::Noop,
                    "load" => Instruction::LoadImmediate(parse_immediate(words.nth(0), linum)?),
                    "push" => Instruction::Push,
                    "pop" => Instruction::Pop,
                    "dup" => Instruction::Dup,
                    "swap" => Instruction::Swap,
                    "ldt" => Instruction::LoadTop,
                    "over" => Instruction::Over,
                    "inc" => Instruction::Inc,
                    "dec" => Instruction::Dec,
                    "add" => Instruction::Add,
                    "sub" => Instruction::Sub,
                    "mul" => Instruction::Mul,
                    "div" => Instruction::Div,
                    "eq" => Instruction::Eq,
                    "neq" => Instruction::Neq,
                    "lt" => Instruction::Lt,
                    "lte" => Instruction::Lte,
                    "gt" => Instruction::Gt,
                    "gte" => Instruction::Gte,
                    "inv" => Instruction::Inv,
                    "jmp" => Instruction::Jmp(parse_address(
                        words.nth(0),
                        linum,
                        inst_count,
                        &mut fixes_by_label,
                    )?),
                    "jz" => Instruction::JumpIfZero(parse_address(
                        words.nth(0),
                        linum,
                        inst_count,
                        &mut fixes_by_label,
                    )?),
                    "jnz" => Instruction::JumpIfNotZero(parse_address(
                        words.nth(0),
                        linum,
                        inst_count,
                        &mut fixes_by_label,
                    )?),
                    "call" => Instruction::Call(parse_address(
                        words.nth(0),
                        linum,
                        inst_count,
                        &mut fixes_by_label,
                    )?),
                    "ret" => Instruction::Ret,
                    label if label.ends_with(':') => {
                        let entry = fixes_by_label.entry(label[0..label.len() - 1].to_string());
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
                        kind: AssemblyErrorKind::UnknownInstruction(inst.to_string()),
                        line: linum,
                    })?,
                };
            }
        }
        inst_count += 1;
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
