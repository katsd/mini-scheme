use crate::lexer::{Meta, Token, TokenKind};
use crate::syntax::*;
use anyhow::{bail, ensure, Result};
use ctx::*;

macro_rules! ensure_paren_open {
    ($ctx:expr) => {
        if $ctx.read()?.kind != TokenKind::ParenOpen {
            bail!("'(' expected");
        }
    };
}

macro_rules! ensure_paren_close {
    ($ctx:expr) => {
        if $ctx.read()?.kind != TokenKind::ParenClose {
            bail!("')' expected");
        }
    };
}

macro_rules! ensure_symbol {
    ($ctx:expr, $kind:expr, $str:expr) => {
        if $ctx.read()?.kind != $kind {
            bail!("'{}' expected", $str);
        }
    };
}

pub fn parse(tokens: Vec<Token>) -> Result<AST> {
    let mut ctx = Context::new(tokens);

    let mut body = vec![];

    while ctx.has_token() {
        body.push(Parse::parse(&mut ctx)?);
    }

    Ok(AST { body })
}

trait Parse
where
    Self: Sized,
{
    fn parse(ctx: &mut Context) -> Result<Self>;
}

impl Parse for Toplevel {
    fn parse(ctx: &mut Context) -> Result<Self> {
        match ctx.peek(0)?.kind {
            TokenKind::ParenOpen => match ctx.peek(1)?.kind {
                TokenKind::Define => Ok(Self::Define(Parse::parse(ctx)?)),
                TokenKind::Load => Ok(Self::Load(Parse::parse(ctx)?)),
                _ => Ok(Self::Exp(Parse::parse(ctx)?)),
            },
            _ => Ok(Self::Exp(Parse::parse(ctx)?)),
        }
    }
}

impl Parse for Load {
    fn parse(ctx: &mut Context) -> Result<Self> {
        ctx.start();

        ensure_paren_open!(ctx);
        ensure_symbol!(ctx, TokenKind::Load, "load");

        let src = Parse::parse(ctx)?;

        ensure_paren_close!(ctx);

        Ok(Self {
            meta: ctx.meta(),
            src,
        })
    }
}

impl Parse for Define {
    fn parse(ctx: &mut Context) -> Result<Self> {
        if ctx.peek(2)?.kind == TokenKind::ParenOpen {
            Ok(Self::Func(Parse::parse(ctx)?))
        } else {
            Ok(Self::Var(Parse::parse(ctx)?))
        }
    }
}

impl Parse for DefVar {
    fn parse(ctx: &mut Context) -> Result<Self> {
        ctx.start();

        ensure_paren_open!(ctx);
        ensure_symbol!(ctx, TokenKind::Define, "define");

        let id = Parse::parse(ctx)?;
        let exp = Parse::parse(ctx)?;

        ensure_paren_close!(ctx);

        Ok(Self {
            meta: ctx.meta(),
            id,
            exp,
        })
    }
}

impl Parse for DefFunc {
    fn parse(ctx: &mut Context) -> Result<Self> {
        ctx.start();

        ensure_paren_open!(ctx);
        ensure_symbol!(ctx, TokenKind::Define, "define");
        ensure_paren_open!(ctx);

        let id = Parse::parse(ctx)?;

        let mut args = vec![];

        while ctx.peek(0)?.kind != TokenKind::Period && ctx.peek(0)?.kind != TokenKind::ParenClose {
            args.push(Parse::parse(ctx)?);
        }

        let varg = if ctx.peek(0)?.kind == TokenKind::Period {
            Some(Parse::parse(ctx)?)
        } else {
            None
        };

        ensure_paren_close!(ctx);

        let exp = Parse::parse(ctx)?;

        ensure_paren_close!(ctx);

        Ok(Self {
            meta: ctx.meta(),
            id,
            args,
            varg,
            exp,
        })
    }
}

impl Parse for Exp {
    fn parse(ctx: &mut Context) -> Result<Self> {
        match ctx.peek(0)?.kind {
            TokenKind::ParenOpen => match ctx.peek(1)?.kind {
                TokenKind::ParenClose => Ok(Self::Const(Parse::parse(ctx)?)),
                TokenKind::Lambda => Ok(Self::Lambda(Box::new(Parse::parse(ctx)?))),
                TokenKind::Quote => Ok(Self::Quote(Box::new(Parse::parse(ctx)?))),
                TokenKind::Set => Ok(Self::Set(Box::new(Parse::parse(ctx)?))),
                TokenKind::Let => Ok(Self::Let(Box::new(Parse::parse(ctx)?))),
                TokenKind::LetAster => Ok(Self::LetAster(Box::new(Parse::parse(ctx)?))),
                TokenKind::LetRec => Ok(Self::LetRec(Box::new(Parse::parse(ctx)?))),
                TokenKind::If => Ok(Self::If(Box::new(Parse::parse(ctx)?))),
                TokenKind::Cond => Ok(Self::Cond(Box::new(Parse::parse(ctx)?))),
                TokenKind::And => Ok(Self::And(Box::new(Parse::parse(ctx)?))),
                TokenKind::Or => Ok(Self::Or(Box::new(Parse::parse(ctx)?))),
                TokenKind::Begin => Ok(Self::Begin(Box::new(Parse::parse(ctx)?))),
                TokenKind::Do => Ok(Self::Do(Box::new(Parse::parse(ctx)?))),
                _ => Ok(Self::Apply(Box::new(Parse::parse(ctx)?))),
            },
            TokenKind::Id(_) => Ok(Self::Id(Parse::parse(ctx)?)),

            TokenKind::Num(_) | TokenKind::Bool(_) | TokenKind::Str(_) => {
                Ok(Self::Const(Parse::parse(ctx)?))
            }
            _ => bail!("Not Exp"),
        }
    }
}

impl Parse for Lambda {
    fn parse(ctx: &mut Context) -> Result<Self> {
        ctx.start();

        ensure_paren_open!(ctx);
        ensure_symbol!(ctx, TokenKind::Lambda, "lambda");

        let arg = Parse::parse(ctx)?;
        let body = Parse::parse(ctx)?;

        ensure_paren_close!(ctx);

        Ok(Self {
            meta: ctx.meta(),
            arg,
            body,
        })
    }
}

impl Parse for Apply {
    fn parse(ctx: &mut Context) -> Result<Self> {
        ctx.start();

        ensure_paren_open!(ctx);

        let func = Parse::parse(ctx)?;

        let mut exps = vec![];

        while ctx.peek(0)?.kind != TokenKind::ParenClose {
            exps.push(Parse::parse(ctx)?);
        }

        ensure_paren_close!(ctx);

        Ok(Self {
            meta: ctx.meta(),
            func,
            exps,
        })
    }
}

impl Parse for Quote {
    fn parse(ctx: &mut Context) -> Result<Self> {
        ctx.start();

        ensure_paren_open!(ctx);
        ensure_symbol!(ctx, TokenKind::Lambda, "quote");

        let s_exp = Parse::parse(ctx)?;

        ensure_paren_close!(ctx);

        Ok(Self {
            meta: ctx.meta(),
            s_exp,
        })
    }
}

impl Parse for Set {
    fn parse(ctx: &mut Context) -> Result<Self> {
        ctx.start();

        ensure_paren_open!(ctx);
        ensure_symbol!(ctx, TokenKind::Lambda, "set");

        let id = Parse::parse(ctx)?;
        let exp = Parse::parse(ctx)?;

        ensure_paren_close!(ctx);

        Ok(Self {
            meta: ctx.meta(),
            id,
            exp,
        })
    }
}

impl Parse for Let {
    fn parse(ctx: &mut Context) -> Result<Self> {
        ctx.start();

        ensure_paren_open!(ctx);
        ensure_symbol!(ctx, TokenKind::Lambda, "let");

        let id = if let TokenKind::Id(_) = ctx.peek(0)?.kind {
            Some(Parse::parse(ctx)?)
        } else {
            None
        };

        let bindings = Parse::parse(ctx)?;
        let body = Parse::parse(ctx)?;

        ensure_paren_close!(ctx);

        Ok(Self {
            meta: ctx.meta(),
            id,
            bindings,
            body,
        })
    }
}

impl Parse for LetAster {
    fn parse(ctx: &mut Context) -> Result<Self> {
        ctx.start();

        ensure_paren_open!(ctx);
        ensure_symbol!(ctx, TokenKind::Lambda, "let*");

        let bindings = Parse::parse(ctx)?;
        let body = Parse::parse(ctx)?;

        ensure_paren_close!(ctx);

        Ok(Self {
            meta: ctx.meta(),
            bindings,
            body,
        })
    }
}

impl Parse for LetRec {
    fn parse(ctx: &mut Context) -> Result<Self> {
        ctx.start();

        ensure_paren_open!(ctx);
        ensure_symbol!(ctx, TokenKind::Lambda, "letrec");

        let bindings = Parse::parse(ctx)?;
        let body = Parse::parse(ctx)?;

        ensure_paren_close!(ctx);

        Ok(Self {
            meta: ctx.meta(),
            bindings,
            body,
        })
    }
}

impl Parse for If {
    fn parse(ctx: &mut Context) -> Result<Self> {
        ctx.start();

        ensure_paren_open!(ctx);
        ensure_symbol!(ctx, TokenKind::If, "if");

        let cond = Parse::parse(ctx)?;
        let then = Parse::parse(ctx)?;

        let el = if ctx.peek(0)?.kind == TokenKind::ParenClose {
            None
        } else {
            Some(Parse::parse(ctx)?)
        };

        ensure_paren_close!(ctx);

        Ok(Self {
            meta: ctx.meta(),
            cond,
            then,
            el,
        })
    }
}

impl Parse for Cond {
    fn parse(ctx: &mut Context) -> Result<Self> {
        todo!()
    }
}

impl Parse for And {
    fn parse(ctx: &mut Context) -> Result<Self> {
        ctx.start();

        ensure_paren_open!(ctx);
        ensure_symbol!(ctx, TokenKind::And, "and");

        let mut exps = vec![];

        while ctx.peek(0)?.kind != TokenKind::ParenClose {
            exps.push(Parse::parse(ctx)?);
        }

        ensure_paren_close!(ctx);

        Ok(Self {
            meta: ctx.meta(),
            exps,
        })
    }
}

impl Parse for Or {
    fn parse(ctx: &mut Context) -> Result<Self> {
        ctx.start();

        ensure_paren_open!(ctx);
        ensure_symbol!(ctx, TokenKind::Or, "or");

        let mut exps = vec![];

        while ctx.peek(0)?.kind != TokenKind::ParenClose {
            exps.push(Parse::parse(ctx)?);
        }

        ensure_paren_close!(ctx);

        Ok(Self {
            meta: ctx.meta(),
            exps,
        })
    }
}

impl Parse for Begin {
    fn parse(ctx: &mut Context) -> Result<Self> {
        ctx.start();

        ensure_paren_open!(ctx);
        ensure_symbol!(ctx, TokenKind::Begin, "begin");

        let mut exps = vec![];

        while ctx.peek(0)?.kind != TokenKind::ParenClose {
            exps.push(Parse::parse(ctx)?);
        }

        ensure_paren_close!(ctx);

        Ok(Self {
            meta: ctx.meta(),
            exps,
        })
    }
}

impl Parse for Do {
    fn parse(ctx: &mut Context) -> Result<Self> {
        todo!()
    }
}

impl Parse for Body {
    fn parse(ctx: &mut Context) -> Result<Self> {
        ctx.start();

        let mut defs = vec![];

        while ctx.peek(0).map_or(false, |t| t.kind == TokenKind::ParenOpen)
            && ctx.peek(1).map_or(false, |t| t.kind == TokenKind::Define)
        {
            defs.push(Parse::parse(ctx)?);
        }

        let mut exps = NonEmptyVec::new(Parse::parse(ctx)?);

        while !ctx.peek(0).map_or(true, |t| t.kind == TokenKind::ParenClose) {
            exps.push(Parse::parse(ctx)?);
        }

        Ok(Self {
            meta: ctx.meta(),
            defs,
            exps,
        })
    }
}

impl Parse for Arg {
    fn parse(ctx: &mut Context) -> Result<Self> {
        if ctx.peek(0)?.kind == TokenKind::ParenOpen {
            Ok(Self::Id(Parse::parse(ctx)?))
        } else {
            Ok(Self::Args(Parse::parse(ctx)?))
        }
    }
}

impl Parse for Args {
    fn parse(ctx: &mut Context) -> Result<Self> {
        ctx.start();

        ensure_paren_open!(ctx);

        let mut args = vec![];

        while ctx.peek(0)?.kind != TokenKind::Period && ctx.peek(0)?.kind != TokenKind::ParenClose {
            args.push(Parse::parse(ctx)?);
        }

        let varg = if ctx.peek(0)?.kind == TokenKind::Period {
            Some(Parse::parse(ctx)?)
        } else {
            None
        };

        ensure_paren_close!(ctx);

        Ok(Self {
            meta: ctx.meta(),
            args,
            varg,
        })
    }
}

impl Parse for Bindings {
    fn parse(ctx: &mut Context) -> Result<Self> {
        ctx.start();

        ensure_paren_open!(ctx);

        let mut bindings = vec![];

        while ctx.peek(0)?.kind != TokenKind::ParenClose {
            bindings.push(Parse::parse(ctx)?);
        }

        ensure_paren_close!(ctx);

        Ok(Self {
            meta: ctx.meta(),
            bindings,
        })
    }
}

impl Parse for Binding {
    fn parse(ctx: &mut Context) -> Result<Self> {
        ctx.start();

        ensure_paren_open!(ctx);

        let id = Parse::parse(ctx)?;
        let exp = Parse::parse(ctx)?;

        ensure_paren_close!(ctx);

        Ok(Self {
            meta: ctx.meta(),
            id,
            exp,
        })
    }
}

impl Parse for SExp {
    fn parse(ctx: &mut Context) -> Result<Self> {
        match ctx.peek(0)?.kind {
            TokenKind::Id(_) => Ok(Self::Id(Parse::parse(ctx)?)),
            TokenKind::Num(_) | TokenKind::Bool(_) | TokenKind::Str(_) => {
                Ok(Self::Const(Parse::parse(ctx)?))
            }
            TokenKind::ParenOpen => {
                if ctx.peek(1)?.kind == TokenKind::ParenClose {
                    Ok(Self::Const(Parse::parse(ctx)?))
                } else {
                    todo!()
                }
            }
            _ => bail!("Not S-Exp"),
        }
    }
}

impl Parse for Const {
    fn parse(ctx: &mut Context) -> Result<Self> {
        match ctx.peek(0)?.kind {
            TokenKind::ParenOpen => Ok(Self::Null(Parse::parse(ctx)?)),
            TokenKind::Num(_) => Ok(Self::Num(Parse::parse(ctx)?)),
            TokenKind::Bool(_) => Ok(Self::Bool(Parse::parse(ctx)?)),
            TokenKind::Str(_) => Ok(Self::String(Parse::parse(ctx)?)),
            _ => bail!("Not Const"),
        }
    }
}

impl Parse for Num {
    fn parse(ctx: &mut Context) -> Result<Self> {
        let t = ctx.read()?;

        let TokenKind::Num(n) = t.kind else {
            bail!("Not Num")
        };

        Ok(Num { meta: t.meta, v: n })
    }
}

impl Parse for Bool {
    fn parse(ctx: &mut Context) -> Result<Self> {
        let t = ctx.read()?;

        let TokenKind::Bool(b) = t.kind else {
            bail!("Not Bool")
        };

        Ok(Bool { meta: t.meta, v: b })
    }
}

impl Parse for Str {
    fn parse(ctx: &mut Context) -> Result<Self> {
        let t = ctx.read()?;

        let TokenKind::Str(s) = t.kind else {
            bail!("Not String")
        };

        Ok(Str { meta: t.meta, v: s })
    }
}

impl Parse for Null {
    fn parse(ctx: &mut Context) -> Result<Self> {
        let t = ctx.read()?;
        let t1 = ctx.read()?;

        ensure!(t.kind == TokenKind::ParenOpen && t1.kind == TokenKind::ParenClose, "Not Null");

        Ok(Null { meta: t.meta })
    }
}

impl Parse for Id {
    fn parse(ctx: &mut Context) -> Result<Self> {
        let t = ctx.read()?;

        let TokenKind::Id(id) = t.kind else {
            bail!("Not Id")
        };

        Ok(Id {
            meta: t.meta,
            v: id,
        })
    }
}

mod ctx {
    use anyhow::{ensure, Result};
    use crate::lexer::Meta;
    use super::Token;

    pub struct Context {
        tokens: Vec<Token>,
        i: usize,
        parse_origins: Vec<usize>,
    }

    impl Context {
        pub fn new(tokens: Vec<Token>) -> Self {
            Self {
                tokens,
                i: 0,
                parse_origins: vec![],
            }
        }

        pub fn read(&mut self) -> Result<Token> {
            ensure!(self.i < self.tokens.len(), "");

            self.i += 1;

            Ok(self.tokens[self.i - 1].clone())
        }

        pub fn peek(&self, n: isize) -> Result<&Token> {
            ensure!(
                0 <= self.i as isize + n && self.i as isize + n < self.tokens.len() as isize,
                ""
            );

            Ok(&self.tokens[(self.i as isize + n) as usize])
        }

        pub fn start(&mut self) {
            self.parse_origins.push(self.i);
        }

        pub fn meta(&mut self) -> Meta {
            Meta::join(
                &self.tokens[self.parse_origins.pop().unwrap()].meta,
                &self.tokens[self.i - 1].meta,
            )
        }

        pub fn has_token(&self) -> bool {
            self.i < self.tokens.len()
        }
    }
}
