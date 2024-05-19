use std::cell::RefCell;
use std::fmt::{Display, Formatter};
use std::rc::Rc;
use anyhow::{bail, Result};

#[derive(Debug, Clone)]
pub enum Obj {
    Bool(bool),
    Number(Number),
    String(String),
    Id(Id),
    Pair(Rc<RefCell<(Obj, Obj)>>),
    Closure { addr: u32, fp: u32 },
    Context { pc: u32, fp: u32 },
    Null,
}

impl PartialEq for Obj {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Bool(l), Self::Bool(r)) => l == r,
            (Self::Number(l), Self::Number(r)) => l == r,
            (Self::String(l), Self::String(r)) => l == r,
            (Self::Id(l), Self::Id(r)) => l.0 == r.0,
            (
                Self::Closure {
                    addr: addr_l,
                    fp: fp_l,
                },
                Self::Closure {
                    addr: addr_r,
                    fp: fp_r,
                },
            ) => addr_l == addr_r && fp_l == fp_r,
            (Self::Context { pc: pc_l, fp: fp_l }, Self::Context { pc: pc_r, fp: fp_r }) => {
                pc_l == pc_r && fp_l == fp_r
            }
            (Self::Null, Self::Null) => true,
            (Self::Pair(l), Self::Pair(r)) => {
                let l = l.borrow();
                let r = r.borrow();

                l.0 == r.0 && l.1 == r.1
            }
            _ => false,
        }
    }
}

impl Display for Obj {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Obj::Bool(v) => write!(f, "{}", v),
            Obj::Number(v) => write!(f, "{}", v),
            Obj::String(v) => write!(f, "{}", v),
            Obj::Id(v) => write!(f, "{}", v.0),
            Obj::Pair(v) => {
                let v = v.borrow();
                write!(f, "({})", display_pair(&v))
            }
            Obj::Closure { addr, fp } => write!(f, "closure({}, {})", addr, fp),
            Obj::Context { pc, fp } => write!(f, "context({}, {})", pc, fp),
            Obj::Null => write!(f, "null"),
        }
    }
}

fn display_pair(pair: &(Obj, Obj)) -> String {
    if let Obj::Pair(v) = &pair.1 {
        format!("{} {}", pair.0, display_pair(&v.borrow()))
    } else {
        match pair.1 {
            Obj::Null => format!("{}", pair.0),
            _ => format!("{} . {}", pair.0, pair.1),
        }
    }
}

impl Obj {
    pub fn bool(self) -> Result<bool> {
        let Self::Bool(n) = self else {
            bail!("Not Bool")
        };

        Ok(n)
    }

    pub fn number(self) -> Result<Number> {
        let Self::Number(n) = self else {
            bail!("Not Number")
        };

        Ok(n)
    }

    pub fn string(self) -> Result<String> {
        let Self::String(n) = self else {
            bail!("Not String")
        };

        Ok(n)
    }

    pub fn id(self) -> Result<Id> {
        let Self::Id(n) = self else { bail!("Not Id") };

        Ok(n)
    }

    pub fn list_elems(self) -> Result<Vec<Obj>> {
        match self {
            Obj::Null => Ok(vec![]),
            Obj::Pair(p) => {
                let p = p.borrow();

                Ok(vec![vec![p.0.clone()], p.1.clone().list_elems()?]
                    .into_iter()
                    .flatten()
                    .collect())
            }
            _ => bail!("Not List"),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Number {
    Int(i64),
    Float(f64),
}

impl PartialEq for Number {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Int(l), Self::Int(r)) => l == r,
            (Self::Int(l), Self::Float(r)) => *l as f64 == *r,
            (Self::Float(l), Self::Int(r)) => *l == *r as f64,
            (Self::Float(l), Self::Float(r)) => l == r,
        }
    }
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
