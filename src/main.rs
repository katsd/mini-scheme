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

    let mut vm = vm::VM::new();

    let _ = vm.exec(prelude(), None, None);
    let _ = vm.exec(src, None, None);
}

fn prelude() -> String {
    include_str!("prelude.scm").into()
}
