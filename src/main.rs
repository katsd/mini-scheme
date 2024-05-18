use std::env;
use std::fs::{File, read_to_string};
use std::io::BufReader;
use crate::lexer::get_tokens;

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

    let src = vec![
        include_str!("std.scm"),
        &read_to_string(src).expect("Failed to open file"),
    ]
    .concat();

    let tokens = get_tokens(src);

    println!("{:?}\n", tokens);

    let ast = parser::parse(tokens).unwrap();

    println!("{:?}\n", ast);

    let insts = codegen::generate(&ast);

    for (idx, inst) in insts.iter().enumerate() {
        println!("{:2} {:?}", idx, inst);
    }

    println!("---------------");

    vm::exec(insts);
}
