use crate::obj::*;

#[derive(Debug)]
struct Frame {}

#[derive(Debug, Clone)]
pub struct Env {}

#[derive(Debug, Clone)]
pub enum Inst {
    Push(Obj),
    Pop,
    Set(Id),
    Get(Id),
    Def(Id),
    Jump(u32),
    Call,
    Ret,
    CreateFrame,
    PushFp,
    CreateClosure(u32),

    Display,
    Newline,

    Add, // +
    Sub, // -
    Mul, // *
    Div, // /
    Eq,  // =
    Lt,  // <
    Le,  // <=
    Gt,  // >
    Ge,  // >=
}

pub fn exec(insts: Vec<Inst>) {}
