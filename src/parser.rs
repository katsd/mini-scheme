use std::collections::HashMap;
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
            panic!("')' expected");
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

pub struct Parser {
    ctx: Context,
}

impl Parser {
    pub fn new() -> Self {
        Self {
            ctx: Context::new(vec![]),
        }
    }

    pub fn parse(&mut self, src: String) -> Result<AST> {
        let tokens = crate::lexer::get_tokens(src);
        self.ctx.add(tokens);

        let mut body = vec![];

        while self.ctx.has_token() {
            body.push(Parse::parse(&mut self.ctx)?);
        }

        Ok(AST { body })
    }
}

trait Parse
where
    Self: Sized,
{
    fn parse(ctx: &mut Context) -> Result<Self>;
}

impl<T> Parse for Vec<T>
where
    T: Parse,
{
    fn parse(ctx: &mut Context) -> Result<Self> {
        let mut res = vec![];

        while ctx.peek(0)?.kind != TokenKind::ParenClose {
            res.push(T::parse(ctx)?);
        }

        Ok(res)
    }
}

impl Parse for Toplevel {
    fn parse(ctx: &mut Context) -> Result<Self> {
        match ctx.peek(0)?.kind {
            TokenKind::ParenOpen => match ctx.peek(1)?.kind {
                TokenKind::DefineSyntax => {
                    let syntax_def = DefineSyntax::parse(ctx)?;

                    ctx.add_syntax_def(syntax_def);

                    Ok(Self::DefineSyntax)
                }
                TokenKind::Define => Ok(Self::Define(Parse::parse(ctx)?)),
                TokenKind::Load => Ok(Self::Load(Parse::parse(ctx)?)),
                _ => Ok(Self::Exp(Parse::parse(ctx)?)),
            },
            _ => Ok(Self::Exp(Parse::parse(ctx)?)),
        }
    }
}

impl Parse for DefineSyntax {
    fn parse(ctx: &mut Context) -> Result<Self> {
        ctx.start();

        ensure_paren_open!(ctx);
        ensure_symbol!(ctx, TokenKind::DefineSyntax, "define-syntax");

        let id = Parse::parse(ctx)?;

        ensure_paren_open!(ctx);
        ensure_symbol!(ctx, TokenKind::SyntaxRules, "syntax-rules");

        ensure_paren_open!(ctx);
        let keywords = Parse::parse(ctx)?;
        ensure_paren_close!(ctx);

        let syntax_rules = Parse::parse(ctx)?;

        ensure_paren_close!(ctx);

        ensure_paren_close!(ctx);

        Ok(Self {
            meta: ctx.meta(),
            id,
            keywords,
            syntax_rules,
        })
    }
}

impl Parse for SyntaxRule {
    fn parse(ctx: &mut Context) -> Result<Self> {
        ctx.start();

        ensure_paren_open!(ctx);
        let syntax = ctx.read_next_chunk()?;
        let template = ctx.read_next_chunk()?;
        ensure_paren_close!(ctx);

        Ok(Self {
            meta: ctx.meta(),
            syntax,
            template,
        })
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
            ensure_symbol!(ctx, TokenKind::Period, ".");
            Some(Parse::parse(ctx)?)
        } else {
            None
        };

        ensure_paren_close!(ctx);

        let body = Parse::parse(ctx)?;

        ensure_paren_close!(ctx);

        Ok(Self {
            meta: ctx.meta(),
            id,
            args,
            varg,
            body,
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
                _ => {
                    let id = ctx.peek(1)?;

                    if let TokenKind::Id(id) = &id.kind {
                        if ctx.is_macro(id) {
                            return Self::expand_macro(ctx);
                        }
                    }

                    Ok(Self::Apply(Box::new(Parse::parse(ctx)?)))
                }
            },
            TokenKind::SingleQuote => Ok(Self::Quote(Box::new(Parse::parse(ctx)?))),

            TokenKind::Id(_) => Ok(Self::Id(Parse::parse(ctx)?)),

            TokenKind::Num(_) | TokenKind::Bool(_) | TokenKind::Str(_) => {
                Ok(Self::Const(Parse::parse(ctx)?))
            }
            _ => panic!("Not Exp"), //bail!("Not Exp"),
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
        ctx.enter_quote();

        let s_exp = if ctx.peek(0)?.kind == TokenKind::SingleQuote {
            ensure_symbol!(ctx, TokenKind::SingleQuote, "'");

            Parse::parse(ctx)?
        } else {
            ensure_paren_open!(ctx);
            ensure_symbol!(ctx, TokenKind::Quote, "quote");

            let s_exp = Parse::parse(ctx)?;

            ensure_paren_close!(ctx);

            s_exp
        };

        ctx.exit_quote();

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
        ensure_symbol!(ctx, TokenKind::Set, "set!");

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
        ensure_symbol!(ctx, TokenKind::Let, "let");

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
        ensure_symbol!(ctx, TokenKind::LetAster, "let*");

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
        ensure_symbol!(ctx, TokenKind::LetRec, "letrec");

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
        ctx.start();

        ensure_paren_open!(ctx);
        ensure_symbol!(ctx, TokenKind::Cond, "cond");

        let mut matches = vec![];
        let mut el = None;

        while ctx.peek(0)?.kind != TokenKind::ParenClose {
            if ctx.peek(1)?.kind == TokenKind::Else {
                ensure_paren_open!(ctx);
                ensure_symbol!(ctx, TokenKind::Else, "else");

                let mut exps = NonEmptyVec::new(Parse::parse(ctx)?);

                while ctx.peek(0)?.kind != TokenKind::ParenClose {
                    exps.push(Parse::parse(ctx)?);
                }

                el = Some(exps);

                ensure_paren_close!(ctx);

                break;
            }

            matches.push(Parse::parse(ctx)?);
        }

        ensure_paren_close!(ctx);

        Ok(Self {
            meta: ctx.meta(),
            matches,
            el,
        })
    }
}

impl Parse for Match {
    fn parse(ctx: &mut Context) -> Result<Self> {
        ctx.start();

        ensure_paren_open!(ctx);

        let cond = Parse::parse(ctx)?;

        let mut then = NonEmptyVec::new(Parse::parse(ctx)?);

        while ctx.peek(0)?.kind != TokenKind::ParenClose {
            then.push(Parse::parse(ctx)?);
        }

        ensure_paren_close!(ctx);

        Ok(Self {
            meta: ctx.meta(),
            cond,
            then,
        })
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
        ctx.start();

        ensure_paren_open!(ctx);
        ensure_symbol!(ctx, TokenKind::Do, "do");

        let mut bindings = vec![];

        ensure_paren_open!(ctx);
        while ctx.peek(0)?.kind != TokenKind::ParenClose {
            bindings.push(Parse::parse(ctx)?);
        }
        ensure_paren_close!(ctx);

        ensure_paren_open!(ctx);
        let cond = Parse::parse(ctx)?;

        let mut value = vec![];
        while ctx.peek(0)?.kind != TokenKind::ParenClose {
            value.push(Parse::parse(ctx)?);
        }
        ensure_paren_close!(ctx);

        let body = Parse::parse(ctx)?;

        ensure_paren_close!(ctx);

        Ok(Self {
            meta: ctx.meta(),
            bindings,
            cond,
            value,
            body,
        })
    }
}

impl Parse for DoBinding {
    fn parse(ctx: &mut Context) -> Result<Self> {
        ctx.start();

        ensure_paren_open!(ctx);

        let id = Parse::parse(ctx)?;
        let i = Parse::parse(ctx)?;
        let u = Parse::parse(ctx)?;

        ensure_paren_close!(ctx);

        Ok(Self {
            meta: ctx.meta(),
            id,
            i,
            u,
        })
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
            Ok(Self::Args(Parse::parse(ctx)?))
        } else {
            Ok(Self::VArg(Parse::parse(ctx)?))
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
            ensure_symbol!(ctx, TokenKind::Period, ".");
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
                    Ok(Self::Pair(Box::new(Parse::parse(ctx)?)))
                }
            }
            _ => bail!("Not S-Exp"),
        }
    }
}

impl Parse for Pair {
    fn parse(ctx: &mut Context) -> Result<Self> {
        ctx.start();

        ensure_paren_open!(ctx);

        let mut exps = vec![];

        while ctx.peek(0)?.kind != TokenKind::Period && ctx.peek(0)?.kind != TokenKind::ParenClose {
            exps.push(Parse::parse(ctx)?);
        }

        let last = if ctx.peek(0)?.kind == TokenKind::Period {
            if exps.len() == 0 {
                panic!("Invalid S-Exp")
            }

            let _ = ctx.read()?;
            Some(Parse::parse(ctx)?)
        } else {
            None
        };

        ensure_paren_close!(ctx);

        Ok(Self {
            meta: ctx.meta(),
            exps,
            last,
        })
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
            panic!("Not Id")
            //bail!("Not Id")
        };

        let id_ctx = if ctx.is_quoted() { 0 } else { t.meta.id_ctx };

        Ok(Id {
            meta: t.meta,
            id_ctx,
            v: id,
        })
    }
}

impl Exp {
    fn expand_macro(ctx: &mut Context) -> Result<Self> {
        let macro_id = {
            let TokenKind::Id(macro_id) = &ctx.peek(1)?.kind else {
                bail!("Not Macro")
            };

            macro_id.clone()
        };

        let expanded = ctx.expand_macro(macro_id)?;

        ctx.insert(expanded);

        Parse::parse(ctx)
    }
}

impl DefineSyntax {
    fn is_keyword(&self, id: &String) -> bool {
        self.keywords.iter().find(|k| &k.v == id).is_some()
    }
}

impl SyntaxRule {
    fn try_match(
        &self,
        def: &DefineSyntax,
        ctx: &mut Context,
    ) -> Result<HashMap<String, Vec<Token>>> {
        let mut reps = HashMap::<String, Vec<Token>>::new();

        for t in &self.syntax {
            match &t.kind {
                TokenKind::ParenOpen | TokenKind::ParenClose | TokenKind::Period => {
                    if ctx.read()?.kind != t.kind {
                        bail!("Invalid syntax");
                    }
                }
                TokenKind::Id(id) => {
                    if id == "_" {
                        ctx.read_next_chunk()?;
                        continue;
                    }

                    if def.is_keyword(id) {
                        if let TokenKind::Id(t) = ctx.read()?.kind {
                            if id != &t {
                                bail!("Invalid syntax");
                            }
                        } else {
                            bail!("Invalid syntax");
                        }

                        continue;
                    }

                    reps.insert(id.clone(), ctx.read_next_chunk()?);
                }
                TokenKind::Ellipsis => {
                    let mut rep = vec![];

                    while ctx.peek(0)?.kind != TokenKind::ParenClose {
                        rep.extend(ctx.read_next_chunk()?);
                    }

                    reps.insert("...".into(), rep);
                }
                _ => {
                    bail!("Invalid syntax");
                }
            }
        }

        Ok(reps)
    }
}

mod ctx {
    use std::collections::HashMap;
    use anyhow::{Context as _, ensure, Result};
    use crate::lexer::Meta;
    use super::*;

    #[derive(Clone)]
    pub struct Context {
        tokens: Vec<Token>,
        i: usize,
        parse_origins: Vec<usize>,

        syntax_defs: HashMap<String, DefineSyntax>,

        id_ctxs: Vec<u32>,
        id_ctx_cnt: u32,

        is_quoted: bool,
    }

    impl Context {
        pub fn new(tokens: Vec<Token>) -> Self {
            Self {
                tokens,
                i: 0,
                parse_origins: vec![],

                syntax_defs: Default::default(),

                id_ctxs: vec![0],
                id_ctx_cnt: 0,

                is_quoted: false,
            }
        }

        pub fn add(&mut self, tokens: Vec<Token>) {
            self.tokens.extend(tokens);
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

        pub fn read_next_chunk(&mut self) -> Result<Vec<Token>> {
            match self.peek(0)?.kind {
                TokenKind::ParenOpen => (),
                TokenKind::SingleQuote => {
                    return Ok(vec![vec![self.read()?], self.read_next_chunk()?].concat())
                }
                _ => return Ok(vec![self.read()?]),
            };

            let mut paren_stack = 0;

            let mut res = vec![];

            loop {
                let t = self.read()?;

                match &t.kind {
                    TokenKind::ParenOpen => paren_stack += 1,
                    TokenKind::ParenClose => paren_stack -= 1,
                    _ => (),
                };

                res.push(t);

                if paren_stack == 0 {
                    break;
                }
            }

            Ok(res)
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

        pub fn insert(&mut self, tokens: Vec<Token>) {
            self.tokens.splice(self.i..self.i, tokens);
        }

        pub fn add_syntax_def(&mut self, def: DefineSyntax) {
            self.syntax_defs.insert(def.id.v.clone(), def);
        }

        pub fn is_macro(&self, id: &String) -> bool {
            self.syntax_defs.iter().find(|i| i.0 == id).is_some()
        }

        pub fn expand_macro(&mut self, id: String) -> Result<Vec<Token>> {
            let def = {
                self.syntax_defs
                    .iter()
                    .find(|i| i.0 == &id)
                    .context(format!("Macro {} is not defined", id))?
                    .1
                    .clone()
            };

            self.enter_new_id_ctx();

            for rule in &def.syntax_rules {
                let mut ctx = self.clone();

                let Ok(reps) = rule.try_match(&def, &mut ctx) else {
                    continue;
                };

                *self = ctx;

                let mut expanded = vec![];

                let mut quote_paren_stack = 0;
                let mut is_quote = false;

                for t in &rule.template {
                    if t.kind == TokenKind::Quote {
                        is_quote = true;
                        quote_paren_stack = 1;
                    }

                    let is_single_quote = t.kind == TokenKind::SingleQuote;

                    if is_single_quote {
                        is_quote = true;
                        quote_paren_stack = 0;
                    }

                    if is_quote {
                        if t.kind == TokenKind::ParenOpen {
                            quote_paren_stack += 1;
                        } else if t.kind == TokenKind::ParenClose {
                            quote_paren_stack -= 1;
                        }
                    }

                    if !is_single_quote && quote_paren_stack == 0 {
                        is_quote = false;
                    }

                    if !is_quote {
                        if let TokenKind::Id(id) = &t.kind {
                            if let Some(rep) = reps.get(id) {
                                expanded.extend(rep.clone());
                                continue;
                            }
                        }

                        if &TokenKind::Ellipsis == &t.kind {
                            if let Some(rep) = reps.get("...") {
                                expanded.extend(rep.clone());
                                continue;
                            }
                        }
                    }

                    let mut t = t.clone();
                    t.meta.id_ctx = self.get_id_ctx();

                    expanded.push(t);
                }

                self.exit_cur_id_ctx();

                return Ok(expanded);
            }

            bail!("Invalid syntax")
        }

        pub fn get_id_ctx(&self) -> u32 {
            *self.id_ctxs.last().unwrap()
        }

        pub fn enter_new_id_ctx(&mut self) {
            self.id_ctx_cnt += 1;
            self.id_ctxs.push(self.id_ctx_cnt);
        }

        pub fn exit_cur_id_ctx(&mut self) {
            let _ = self.id_ctxs.pop();
        }

        pub fn enter_quote(&mut self) {
            self.is_quoted = true;
        }

        pub fn exit_quote(&mut self) {
            self.is_quoted = false;
        }

        pub fn is_quoted(&self) -> bool {
            self.is_quoted
        }
    }
}
