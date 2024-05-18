use std::env;
use std::fs::read_to_string;

use crate::vm::Inst;

mod lexer;
mod parser;
mod syntax;
mod vm;
mod codegen;
mod obj;

fn main() {
    let args: Vec<String> = env::args().collect();

    let src = if args.len() < 2 {
        "test/yay.scm"
    } else {
        &args[1]
    };

    let src = read_to_string(src).expect("Failed to open file");

    let tokens = lexer::get_tokens(src);

    println!("{:?}\n", tokens);

    let ast = parser::parse(tokens).unwrap();

    println!("{:?}\n", ast);

    let insts = codegen::generate(&ast, true);

    for (idx, inst) in insts.iter().enumerate() {
        println!("{:2} {:?}", idx, inst);
    }

    let insts = codegen::join(prelude(), insts);

    println!("\n==================\n");

    vm::exec(insts);
}

fn prelude() -> Vec<Inst> {
    let src = include_str!("prelude.scm");
    let tokens = lexer::get_tokens(src.into());
    let ast = parser::parse(tokens).unwrap();
    codegen::generate(&ast, false)
}
