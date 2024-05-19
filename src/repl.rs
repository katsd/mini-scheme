use std::io::{BufRead, Write};
use std::process::exit;
use std::sync::mpsc::channel;

pub fn run() {
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();

    let mut vm = crate::vm::VM::new();
    let _ = vm.exec(crate::prelude(), None);

    let mut var_cnt = 0;

    let (tx, rx) = channel();
    ctrlc::set_handler(move || tx.send(()).unwrap()).expect("Failed to set Ctrl-C handler");

    loop {
        print!("> ");
        stdout.flush().unwrap();

        let mut input = "".into();
        stdin.lock().read_line(&mut input).unwrap();
        let input = input.trim_end();

        if input == ":q" {
            exit(0);
        }

        let tokens = crate::lexer::get_tokens(input.into());
        let Ok(ast) = crate::parser::parse(tokens) else {
            println!("syntax error");
            continue;
        };

        let mut insts = crate::codegen::generate(&ast, true);
        let _ = insts.pop();

        use crate::vm::Inst;
        use crate::obj;

        let var = obj::Id(format!("${}", var_cnt));
        var_cnt += 1;

        insts.push(Inst::Def(var.clone()));
        insts.push(Inst::Set(var.clone()));
        insts.push(Inst::Get(var.clone()));
        insts.push(Inst::Exit);

        /*
        for (idx, inst) in insts.iter().enumerate() {
            println!("{:3} {:?}", idx, inst);
        }
        */

        let ret = vm.exec(insts, Some(&rx));

        println!("{} = {}", var.0, ret);
    }
}
