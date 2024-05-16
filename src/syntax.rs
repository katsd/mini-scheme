use crate::lexer::Meta;
use crate::obj::Number;

#[derive(Debug, Clone)]
pub struct AST {
    pub body: Vec<Toplevel>,
}

#[derive(Debug, Clone)]
pub enum Toplevel {
    Exp(Exp),
    Define(Define),
    Load(Load),
}

#[derive(Debug, Clone)]
pub struct Load {
    pub meta: Meta,
    pub src: Str,
}

#[derive(Debug, Clone)]
pub enum Define {
    Var(DefVar),
    Func(DefFunc),
}

#[derive(Debug, Clone)]
pub struct DefVar {
    pub meta: Meta,
    pub id: Id,
    pub exp: Exp,
}

#[derive(Debug, Clone)]
pub struct DefFunc {
    pub meta: Meta,
    pub id: Id,
    pub args: Vec<Id>,
    pub varg: Option<Id>,
    pub body: Body,
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct Lambda {
    pub meta: Meta,
    pub arg: Arg,
    pub body: Body,
}

#[derive(Debug, Clone)]
pub struct Apply {
    pub meta: Meta,
    pub func: Exp,
    pub exps: Vec<Exp>,
}

#[derive(Debug, Clone)]
pub struct Quote {
    pub meta: Meta,
    pub s_exp: SExp,
}

#[derive(Debug, Clone)]
pub struct Set {
    pub meta: Meta,
    pub id: Id,
    pub exp: Exp,
}

#[derive(Debug, Clone)]
pub struct Let {
    pub meta: Meta,
    pub id: Option<Id>,
    pub bindings: Bindings,
    pub body: Body,
}

#[derive(Debug, Clone)]
pub struct LetAster {
    pub meta: Meta,
    pub bindings: Bindings,
    pub body: Body,
}

#[derive(Debug, Clone)]
pub struct LetRec {
    pub meta: Meta,
    pub bindings: Bindings,
    pub body: Body,
}

#[derive(Debug, Clone)]
pub struct If {
    pub meta: Meta,
    pub cond: Exp,
    pub then: Exp,
    pub el: Option<Exp>,
}

#[derive(Debug, Clone)]
pub struct Cond {
    pub meta: Meta,
    pub matches: Vec<Match>,
    pub el: Option<NonEmptyVec<Exp>>,
}

#[derive(Debug, Clone)]
pub struct Match {
    pub meta: Meta,
    pub cond: Exp,
    pub then: NonEmptyVec<Exp>,
}

#[derive(Debug, Clone)]
pub struct And {
    pub meta: Meta,
    pub exps: Vec<Exp>,
}

#[derive(Debug, Clone)]
pub struct Or {
    pub meta: Meta,
    pub exps: Vec<Exp>,
}

#[derive(Debug, Clone)]
pub struct Begin {
    pub meta: Meta,
    pub exps: Vec<Exp>,
}

#[derive(Debug, Clone)]
pub struct Do {
    pub meta: Meta,
    pub bindings: Vec<DoBinding>,
    pub cond: Exp,
    pub value: Vec<Exp>,
    pub body: Body,
}

#[derive(Debug, Clone)]
pub struct DoBinding {
    pub meta: Meta,
    pub id: Id,
    pub i: Exp,
    pub u: Exp,
}

#[derive(Debug, Clone)]
pub struct Body {
    pub meta: Meta,
    pub defs: Vec<Define>,
    pub exps: NonEmptyVec<Exp>,
}

#[derive(Debug, Clone)]
pub enum Arg {
    Id(Id),
    Args(Args),
}

#[derive(Debug, Clone)]
pub struct Args {
    pub meta: Meta,
    pub args: Vec<Id>,
    pub varg: Option<Id>,
}

#[derive(Debug, Clone)]
pub struct Bindings {
    pub meta: Meta,
    pub bindings: Vec<Binding>,
}

#[derive(Debug, Clone)]
pub struct Binding {
    pub meta: Meta,
    pub id: Id,
    pub exp: Exp,
}

#[derive(Debug, Clone)]
pub enum SExp {
    Const(Const),
    Id(Id),
    List(List),
}

#[derive(Debug, Clone)]
pub enum List {
    // TODO
}

#[derive(Debug, Clone)]
pub enum Const {
    Num(Num),
    Bool(Bool),
    String(Str),
    Null(Null),
}

#[derive(Debug, Clone)]
pub struct Id {
    pub meta: Meta,
    pub v: String,
}

#[derive(Debug, Clone)]
pub struct Num {
    pub meta: Meta,
    pub v: Number,
}

#[derive(Debug, Clone)]
pub struct Bool {
    pub meta: Meta,
    pub v: bool,
}

#[derive(Debug, Clone)]
pub struct Str {
    pub meta: Meta,
    pub v: String,
}

#[derive(Debug, Clone)]
pub struct Null {
    pub meta: Meta,
}

#[derive(Debug, Clone)]
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

    pub fn get(&self) -> &Vec<T> {
        &self.inner
    }
}
