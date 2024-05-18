use std::collections::HashMap;
use std::fs::read_to_string;
use std::sync::{Arc, Mutex};
use crate::obj::*;

#[derive(Debug, Clone)]
pub struct Frame {
    parent: Option<u32>,
    table: HashMap<Id, Obj>,
    ref_cnt: u32,
}

#[derive(Debug, Clone)]
pub enum Inst {
    Push(Obj),
    Pop,
    Set(Id),
    Get(Id),
    Def(Id),
    CollectVArg(Id),
    Jump(u32),
    JumpIf(u32),
    Call,
    Ret,
    PushReturnContext(u32),
    CreateClosure(u32),
    Load,
    Exit,

    Display,

    Add, // +
    Sub, // -
    Mul, // *
    Div, // /
    Eq,  // =
    Lt,  // <
    Le,  // <=
    Gt,  // >
    Ge,  // >=

    And,
    Or,
    Not,

    Cons,
    Car,
    Cdr,
    SetCar,
    SetCdr,
    ExpandList,

    IsNull,
    IsPair,
    IsNumber,
    IsBool,
    IsString,
    IsProc,
    IsSymbol,

    IsEq,
    IsEqual,

    SymToStr,
    StrToSym,
    StrToNum,
    NumToStr,
    StringAppend,
}

pub fn exec(insts: Vec<Inst>) {
    let mut insts = insts;

    let mut pc: u32 = 0;

    let mut sp = 0;
    let mut stack = vec![Obj::Null; 1000];

    let mut fp: u32 = 0;
    let mut frame_stack = vec![Option::<Frame>::None; 1000];
    frame_stack[0] = Some(Frame {
        parent: None,
        table: Default::default(),
        ref_cnt: 1,
    });

    macro_rules! pop {
        () => {{
            let v = std::mem::replace(&mut stack[sp], Obj::Null);
            update_ref_cnt(&v, &mut frame_stack, false);
            sp -= 1;
            v
        }};
    }

    macro_rules! push {
        ($obj:expr) => {{
            let v = $obj;
            update_ref_cnt(&v, &mut frame_stack, true);
            sp += 1;
            stack[sp] = v;
        }};
    }

    macro_rules! pop_retaining_ref {
        () => {{
            let v = std::mem::replace(&mut stack[sp], Obj::Null);
            sp -= 1;
            v
        }};
    }

    macro_rules! push_retaining_ref {
        ($obj:expr) => {{
            sp += 1;
            stack[sp] = $obj;
        }};
    }

    loop {
        let inst = insts[pc as usize].clone();

        match &inst {
            Inst::Push(obj) => {
                push!(obj.clone());
            }
            Inst::Pop => {
                pop!();
            }
            Inst::Set(id) => {
                let v = pop_retaining_ref!();

                let prev = find_var(&id, &fp, &mut frame_stack, |obj| std::mem::replace(obj, v))
                    .expect(&format!("{} is not defined", id.0));

                update_ref_cnt(&prev, &mut frame_stack, false);
            }
            Inst::CollectVArg(id) => {
                let mut args = vec![];

                loop {
                    let v = pop_retaining_ref!();

                    if let Obj::Context { pc: _, fp: _ } = v {
                        push_retaining_ref!(v);
                        break;
                    }

                    args.push(v);
                }

                let mut list = Obj::Null;

                for arg in args.into_iter().rev() {
                    list = Obj::Pair(Arc::new(Mutex::new(Box::new((arg, list)))))
                }

                push_retaining_ref!(list);
            }
            Inst::Get(id) => {
                let v = find_var(&id, &fp, &mut frame_stack, |obj| obj.clone())
                    .expect(&format!("{} is not defined", id.0));

                push!(v);
            }
            Inst::Def(id) => {
                let frame = frame_stack[fp as usize].as_mut().unwrap();
                frame.table.insert(id.clone(), Obj::Null);
            }
            Inst::Jump(pc_next) => {
                pc = *pc_next;
                continue;
            }
            Inst::JumpIf(pc_next) => {
                if pop!().bool() {
                    pc = *pc_next;
                    continue;
                };
            }
            Inst::Call => {
                let Obj::Closure {
                    addr,
                    fp: fp_parent,
                } = pop_retaining_ref!()
                else {
                    panic!("Not closure")
                };

                let new_frame = Frame {
                    parent: Some(fp_parent),
                    table: Default::default(),
                    ref_cnt: 1,
                };

                while frame_stack[fp as usize].is_some() {
                    fp += 1;
                }

                frame_stack[fp as usize] = Some(new_frame);

                pc = addr;

                continue;
            }
            Inst::Ret => {
                for (_, obj) in
                    frame_stack.get(fp as usize).unwrap().as_ref().unwrap().table.clone()
                {
                    update_ref_cnt(&obj, &mut frame_stack, false);
                }

                let v = pop_retaining_ref!();

                loop {
                    let v = pop!();

                    let Obj::Context {
                        pc: pc_prev,
                        fp: fp_prev,
                    } = v
                    else {
                        continue;
                    };

                    update_ref_cnt(&Obj::Closure { addr: 0, fp }, &mut frame_stack, false);

                    pc = pc_prev;
                    fp = fp_prev;

                    break;
                }

                push_retaining_ref!(v);

                continue;
            }
            Inst::Load => {
                let src = pop!().string();
                let src = read_to_string(&src).expect(&format!("Failed to open {}", src));

                let tokens = crate::lexer::get_tokens(src);
                let ast = crate::parser::parse(tokens).expect("Failed to parse");

                let next_pc = insts.len() as u32;

                insts = crate::codegen::join(insts, crate::codegen::generate(&ast, false));

                insts.push(Inst::Jump(pc + 1));

                pc = next_pc;

                continue;
            }
            Inst::Exit => {
                return;
            }
            Inst::PushReturnContext(pc) => {
                push!(Obj::Context { pc: *pc, fp });
            }
            Inst::CreateClosure(pc) => {
                let v = Obj::Closure { addr: *pc, fp };
                push!(v);
            }
            Inst::Display => {
                let v = pop!();

                print!("{}", v);
                push!(Obj::Null);
            }
            Inst::Add
            | Inst::Sub
            | Inst::Mul
            | Inst::Div
            | Inst::Eq
            | Inst::Lt
            | Inst::Le
            | Inst::Gt
            | Inst::Ge => {
                let l = pop!().number();
                let r = pop!().number();

                let obj = if let (Number::Int(l), Number::Int(r)) = (l, r) {
                    match &inst {
                        Inst::Add => Obj::Number(Number::Int(l + r)),
                        Inst::Sub => Obj::Number(Number::Int(l - r)),
                        Inst::Mul => Obj::Number(Number::Int(l * r)),
                        Inst::Div => Obj::Number(Number::Int(l / r)),
                        Inst::Eq => Obj::Bool(l == r),
                        Inst::Lt => Obj::Bool(l < r),
                        Inst::Le => Obj::Bool(l <= r),
                        Inst::Gt => Obj::Bool(l > r),
                        Inst::Ge => Obj::Bool(l >= r),
                        _ => unreachable!(),
                    }
                } else {
                    let l = l.float();
                    let r = r.float();

                    match &inst {
                        Inst::Add => Obj::Number(Number::Float(l + r)),
                        Inst::Sub => Obj::Number(Number::Float(l - r)),
                        Inst::Mul => Obj::Number(Number::Float(l * r)),
                        Inst::Div => Obj::Number(Number::Float(l / r)),
                        Inst::Eq => Obj::Bool(l == r),
                        Inst::Lt => Obj::Bool(l < r),
                        Inst::Le => Obj::Bool(l <= r),
                        Inst::Gt => Obj::Bool(l > r),
                        Inst::Ge => Obj::Bool(l >= r),
                        _ => unreachable!(),
                    }
                };

                push!(obj);
            }
            Inst::And => {
                push!(Obj::Bool(pop!().bool() && pop!().bool()));
            }
            Inst::Or => {
                push!(Obj::Bool(pop!().bool() || pop!().bool()));
            }
            Inst::Not => {
                push!(Obj::Bool(!pop!().bool()));
            }
            Inst::Cons => {
                let l = pop_retaining_ref!();
                let r = pop_retaining_ref!();

                let v = Obj::Pair(Arc::new(Mutex::new(Box::new((l, r)))));
                push_retaining_ref!(v);
            }
            Inst::Car => {
                let Obj::Pair(v) = pop_retaining_ref!() else {
                    panic!("Not Pair")
                };

                let v = v.lock().unwrap();

                let l = &v.0;
                let r = &v.1;

                update_ref_cnt(&r, &mut frame_stack, false);
                push_retaining_ref!(l.clone());
            }
            Inst::Cdr => {
                let Obj::Pair(v) = pop_retaining_ref!() else {
                    panic!("Not Pair")
                };

                let v = v.lock().unwrap();

                let l = &v.0;
                let r = &v.1;

                update_ref_cnt(&l, &mut frame_stack, false);
                push_retaining_ref!(r.clone());
            }
            Inst::SetCar => {
                let v = pop_retaining_ref!();
                let l = pop_retaining_ref!();

                let Obj::Pair(v) = v else { panic!("Not Pair") };

                let mut v = v.lock().unwrap();

                let l = std::mem::replace(&mut v.0, l);

                update_ref_cnt(&l, &mut frame_stack, false);
            }
            Inst::SetCdr => {
                let v = pop_retaining_ref!();
                let r = pop_retaining_ref!();

                let Obj::Pair(v) = v else { panic!("Not Pair") };

                let mut v = v.lock().unwrap();

                let r = std::mem::replace(&mut v.1, r);

                update_ref_cnt(&r, &mut frame_stack, false);
            }
            Inst::ExpandList => {
                let v = pop_retaining_ref!();

                for e in v.list_elems().into_iter().rev() {
                    push!(e);
                }
            }
            Inst::IsNull => {
                push!(Obj::Bool(match pop!() {
                    Obj::Null => true,
                    _ => false,
                }));
            }
            Inst::IsPair => {
                push!(Obj::Bool(match pop!() {
                    Obj::Pair(_) => true,
                    _ => false,
                }));
            }
            Inst::IsNumber => {
                push!(Obj::Bool(match pop!() {
                    Obj::Number(_) => true,
                    _ => false,
                }));
            }
            Inst::IsBool => {
                push!(Obj::Bool(match pop!() {
                    Obj::Bool(_) => true,
                    _ => false,
                }));
            }
            Inst::IsString => {
                push!(Obj::Bool(match pop!() {
                    Obj::String(_) => true,
                    _ => false,
                }));
            }
            Inst::IsProc => {
                push!(Obj::Bool(match pop!() {
                    Obj::Closure { addr: _, fp: _ } => true,
                    _ => false,
                }));
            }
            Inst::IsSymbol => {
                push!(Obj::Bool(match pop!() {
                    Obj::Id(_) => true,
                    _ => false,
                }));
            }
            Inst::IsEq => {
                let l = pop!();
                let r = pop!();

                push!(Obj::Bool(match (&l, &r) {
                    (Obj::Pair(l), Obj::Pair(r)) => {
                        Arc::ptr_eq(l, r)
                    }
                    _ => l == r,
                }));
            }
            Inst::IsEqual => {
                push!(Obj::Bool(pop!() == pop!()))
            }
            Inst::SymToStr => {
                let v = pop!().id();
                push!(Obj::String(v.0));
            }
            Inst::StrToSym => {
                let v = pop!().string();
                push!(Obj::Id(Id(v)));
            }
            Inst::StrToNum => {
                let v = pop!().string();
                if let Ok(n) = v.parse::<i64>() {
                    push!(Obj::Number(Number::Int(n)));
                } else if let Ok(n) = v.parse::<f64>() {
                    push!(Obj::Number(Number::Float(n)));
                } else {
                    push!(Obj::Number(Number::Int(0)));
                }
            }
            Inst::NumToStr => {
                let v = pop!().number();
                push!(Obj::String(format!("{}", v)));
            }
            Inst::StringAppend => {
                let l = pop!().string();
                let r = pop!().string();

                push!(Obj::String(format!("{}{}", l, r)));
            }
        }

        pc += 1;
    }
}

fn find_var<T, F>(
    id: &Id,
    fp: &u32,
    frame_stack: &mut Vec<Option<Frame>>,
    mut action: F,
) -> Option<T>
where
    F: FnOnce(&mut Obj) -> T,
{
    let mut fp = *fp;

    loop {
        let frame = frame_stack.get_mut(fp as usize)?;
        let frame = frame.as_mut().unwrap();

        if let Some(obj) = frame.table.get_mut(id) {
            return Some(action(obj));
        }

        fp = frame.parent?;
    }
}

fn update_ref_cnt(obj: &Obj, frame_stack: &mut Vec<Option<Frame>>, increment: bool) {
    let Obj::Closure { addr, fp } = obj else {
        return;
    };

    let mut fp = *fp;

    loop {
        let frame = frame_stack.get_mut(fp as usize).unwrap();
        let parent_fp = frame.as_ref().unwrap().parent;

        if increment {
            frame.as_mut().unwrap().ref_cnt += 1;
        } else {
            frame.as_mut().unwrap().ref_cnt -= 1;

            if frame.as_mut().unwrap().ref_cnt == 0 {
                *frame = None;
            }
        }

        let Some(parent_fp) = parent_fp else {
            break;
        };

        fp = parent_fp;
    }
}
