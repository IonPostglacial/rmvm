use vm::Instruction;

mod vm;

const CODE_SIZE: usize = 128_000;

fn main() {
    let src = std::fs::read_to_string("sample.maf").expect("unable to open file");
    let mut code = [Instruction::Halt; CODE_SIZE];
    vm::asm::code_from_str(&src, &mut code).expect("could not parse code");
    let mut machine = vm::Machine::new();
    let res = machine.run(&code);
    println!(
        "pc: {}, sp: {}, fp: {}, acc: {}",
        machine.pc, machine.sp, machine.fp, machine.acc
    );
    println!("res: {:?}", res);
}
