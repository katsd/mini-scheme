use std::fmt::{Display, Formatter};

#[derive(Debug, Clone)]
pub enum Obj {
    Bool(bool),
    Number(Number),
    String(String),
    Id(Id),
    Pair { l: Box<Obj>, r: Box<Obj> },
    Closure { addr: u32, fp: u32 },
    Context { pc: u32, fp: u32 },
    Null,
}

impl Display for Obj {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Obj::Bool(v) => write!(f, "{}", v),
            Obj::Number(v) => write!(f, "{}", v),
            Obj::String(v) => write!(f, "{}", v),
            Obj::Id(v) => write!(f, "id({})", v.0),
            Obj::Pair { l, r } => write!(f, "({}, {})", l, r),
            Obj::Closure { addr, fp } => write!(f, "closure({}, {})", addr, fp),
            Obj::Context { pc, fp } => write!(f, "context({}, {})", pc, fp),
            Obj::Null => write!(f, "null"),
        }
    }
}

impl Obj {
    pub fn bool(self) -> bool {
        let Self::Bool(n) = self else {
            panic!("Not Bool")
        };

        n
    }

    pub fn number(self) -> Number {
        let Self::Number(n) = self else {
            panic!("Not Number")
        };

        n
    }

    pub fn string(self) -> String {
        let Self::String(n) = self else {
            panic!("Not String")
        };

        n
    }

    pub fn id(self) -> Id {
        let Self::Id(n) = self else { panic!("Not Id") };

        n
    }
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

impl Display for Number {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Number::Int(v) => write!(f, "{}", v),
            Number::Float(v) => write!(f, "{}", v),
        }
    }
}

impl Number {
    pub fn int(&self) -> i64 {
        match self {
            Number::Int(n) => *n,
            Number::Float(n) => *n as i64,
        }
    }

    pub fn float(&self) -> f64 {
        match self {
            Number::Int(n) => *n as f64,
            Number::Float(n) => *n,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct Id(pub String);
