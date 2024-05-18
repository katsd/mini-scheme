use std::env;
use std::fs::read_to_string;

use crate::vm::Inst;

mod lexer;
mod parser;
mod syntax;
mod vm;
mod codegen;
mod obj;
mod repl;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        repl::run();
    }

    let src = &args[1];

    let src = read_to_string(src).expect("Failed to open file");

    let tokens = lexer::get_tokens(src);

    println!("{:?}\n", tokens);

    let ast = parser::parse(tokens).unwrap();

    println!("{:?}\n", ast);

    let insts = codegen::generate(&ast, true);

    for (idx, inst) in insts.iter().enumerate() {
        println!("{:2} {:?}", idx, inst);
    }

    let mut vm = vm::VM::new();

    vm.exec(prelude(), None);

    println!("\n==================\n");

    let _ = vm.exec(insts, None);
}

fn prelude() -> Vec<Inst> {
    let src = include_str!("prelude.scm");
    let tokens = lexer::get_tokens(src.into());
    let ast = parser::parse(tokens).unwrap();
    codegen::generate(&ast, true)
}
