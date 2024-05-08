use crate::lexer::Meta;
use crate::number::Number;

#[derive(Debug)]
pub struct AST {
    pub body: Vec<Toplevel>,
}

#[derive(Debug)]
pub enum Toplevel {
    Exp(Exp),
    Define(Define),
    Load(Load),
}

#[derive(Debug)]
pub struct Load {
    pub meta: Meta,
    pub src: Str,
}

#[derive(Debug)]
pub enum Define {
    Var(DefVar),
    Func(DefFunc),
}

#[derive(Debug)]
pub struct DefVar {
    pub meta: Meta,
    pub id: Id,
    pub exp: Exp,
}

#[derive(Debug)]
pub struct DefFunc {
    pub meta: Meta,
    pub id: Id,
    pub args: Vec<Id>,
    pub varg: Option<Id>,
    pub exp: Exp,
}

#[derive(Debug)]
pub enum Exp {
    Const(Const),
    Id(Id),
    Lambda(Box<Lambda>),
    Apply(Box<Apply>),
    Quote(Box<Quote>),
    Set(Box<Set>),
    Let(Box<Let>),
    LetAster(Box<LetAster>),
    LetRec(Box<LetRec>),
    If(Box<If>),
    Cond(Box<Cond>),
    And(Box<And>),
    Or(Box<Or>),
    Begin(Box<Begin>),
    Do(Box<Do>),
}

#[derive(Debug)]
pub struct Lambda {
    pub meta: Meta,
    pub arg: Arg,
    pub body: Body,
}

#[derive(Debug)]
pub struct Apply {
    pub meta: Meta,
    pub func: Exp,
    pub exps: Vec<Exp>,
}

#[derive(Debug)]
pub struct Quote {
    pub meta: Meta,
    pub s_exp: SExp,
}

#[derive(Debug)]
pub struct Set {
    pub meta: Meta,
    pub id: Id,
    pub exp: Exp,
}

#[derive(Debug)]
pub struct Let {
    pub meta: Meta,
    pub id: Option<Id>,
    pub bindings: Bindings,
    pub body: Body,
}

#[derive(Debug)]
pub struct LetAster {
    pub meta: Meta,
    pub bindings: Bindings,
    pub body: Body,
}

#[derive(Debug)]
pub struct LetRec {
    pub meta: Meta,
    pub bindings: Bindings,
    pub body: Body,
}

#[derive(Debug)]
pub struct If {
    pub meta: Meta,
    pub cond: Exp,
    pub then: Exp,
    pub el: Option<Exp>,
}

#[derive(Debug)]
pub struct Cond {
    pub meta: Meta,
    pub matches: Vec<Match>,
    pub el: Option<NonEmptyVec<Exp>>,
}

#[derive(Debug)]
pub struct Match {
    pub meta: Meta,
    pub cond: Exp,
    pub then: NonEmptyVec<Exp>,
}

#[derive(Debug)]
pub struct And {
    pub meta: Meta,
    pub exps: Vec<Exp>,
}

#[derive(Debug)]
pub struct Or {
    pub meta: Meta,
    pub exps: Vec<Exp>,
}

#[derive(Debug)]
pub struct Begin {
    pub meta: Meta,
    pub exps: Vec<Exp>,
}

#[derive(Debug)]
pub struct Do {
    pub meta: Meta,
    pub bindings: Vec<DoBinding>,
    pub cond: Exp,
    pub value: Vec<Exp>,
    pub body: Body,
}

#[derive(Debug)]
pub struct DoBinding {
    pub meta: Meta,
    pub id: Id,
    pub i: Exp,
    pub u: Exp,
}

#[derive(Debug)]
pub struct Body {
    pub meta: Meta,
    pub defs: Vec<Define>,
    pub exps: NonEmptyVec<Exp>,
}

#[derive(Debug)]
pub enum Arg {
    Id(Id),
    Args(Args),
}

#[derive(Debug)]
pub struct Args {
    pub meta: Meta,
    pub args: Vec<Id>,
    pub varg : Option<Id>,
}

#[derive(Debug)]
pub struct Bindings {
    pub meta: Meta,
    pub bindings: Vec<Binding>,
}

#[derive(Debug)]
pub struct Binding {
    pub meta: Meta,
    pub id: Id,
    pub exp: Exp,
}

#[derive(Debug)]
pub enum SExp {
    Const(Const),
    Id(Id),
    List(List),
}

#[derive(Debug)]
pub enum List {
    // TODO
}

#[derive(Debug)]
pub enum Const {
    Num(Num),
    Bool(Bool),
    String(Str),
    Null(Null),
}

#[derive(Debug)]
pub struct Id {
    pub meta: Meta,
    pub v: String,
}

#[derive(Debug)]
pub struct Num {
    pub meta: Meta,
    pub v: Number,
}

#[derive(Debug)]
pub struct Bool {
    pub meta: Meta,
    pub v: bool,
}

#[derive(Debug)]
pub struct Str {
    pub meta: Meta,
    pub v: String,
}

#[derive(Debug)]
pub struct Null {
    pub meta: Meta,
}

#[derive(Debug)]
pub struct NonEmptyVec<T> {
    inner: Vec<T>,
}

impl<T> NonEmptyVec<T> {
    pub fn new(t: T) -> Self {
        Self { inner: vec![t] }
    }

    pub fn push(&mut self, t: T) {
        self.inner.push(t);
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn first(&self) -> &T {
        self.inner.first().unwrap()
    }

    pub fn last(&self) -> &T {
        self.inner.last().unwrap()
    }
}
