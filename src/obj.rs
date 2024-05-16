use crate::vm::Env;

#[derive(Debug, Clone)]
pub enum Obj {
    Bool(bool),
    Number(Number),
    String(String),
    Id(Id),
    Pair { l: Box<Obj>, r: Box<Obj> },
    Closure { addr: u32, env: Env },
    Null,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Number {
    Int(i64),
    Float(f64),
}

impl From<i64> for Number {
    fn from(value: i64) -> Self {
        Self::Int(value)
    }
}

impl From<f64> for Number {
    fn from(value: f64) -> Self {
        Self::Float(value)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Id(pub String);
