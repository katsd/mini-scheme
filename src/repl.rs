use std::io::{BufRead, Write};
use std::process::exit;
use std::sync::mpsc::channel;

pub fn run() {
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();

    let mut vm = crate::vm::VM::new();
    let _ = vm.exec(crate::prelude(), None, None);

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

        use crate::vm::Inst;
        use crate::obj;

        let var = obj::Id(format!("${}", var_cnt));
        var_cnt += 1;

        let ret = vm.exec(
            input.into(),
            Some(&rx),
            Some(vec![
                Inst::Def(var.clone()),
                Inst::Set(var.clone()),
                Inst::Get(var.clone()),
                Inst::Exit,
            ]),
        );

        match ret {
            Ok(ret) => println!("{} = {}", var.0, ret),
            Err(e) => println!("Error: {}", e),
        }
    }
}
