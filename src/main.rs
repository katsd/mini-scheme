use std::env;
use std::fs::{File, read_to_string};
use std::io::BufReader;
use crate::lexer::get_tokens;

mod lexer;
mod number;
mod parser;
mod syntax;

fn main() {
    let args: Vec<String> = env::args().collect();

    let src = if args.len() < 2 {
        "test/yay.ss"
    } else {
        &args[1]
    };

    let src = read_to_string(src).expect("Failed to open file");

    let tokens = get_tokens(src);

    println!("{:?}", tokens);

    let ast = parser::parse(tokens).unwrap();

    println!("{:?}", ast);
}
