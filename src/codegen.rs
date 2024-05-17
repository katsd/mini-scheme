use crate::syntax;
use crate::obj::*;
use crate::vm::Inst;
use builder::Builder;

pub fn generate(ast: &syntax::AST) -> Vec<Inst> {
    let mut builder = Builder::new();

    for t in &ast.body {
        t.gen(&mut builder);
    }

    builder.push(Inst::Exit);

    builder.build()
}

trait Gen {
    fn gen(&self, builder: &mut Builder);
}

impl Gen for syntax::Toplevel {
    fn gen(&self, builder: &mut Builder) {
        match &self {
            syntax::Toplevel::Exp(t) => t.gen(builder),
            syntax::Toplevel::Define(t) => t.gen(builder),
            syntax::Toplevel::Load(t) => t.gen(builder),
        }
    }
}

impl Gen for syntax::Load {
    fn gen(&self, builder: &mut Builder) {
        todo!()
    }
}

impl Gen for syntax::Define {
    fn gen(&self, builder: &mut Builder) {
        match self {
            Self::Var(t) => t.gen(builder),
            Self::Func(t) => t.gen(builder),
        }
    }
}

impl Gen for syntax::DefVar {
    fn gen(&self, builder: &mut Builder) {
        builder.push(Inst::Def(Id(self.id.v.clone())));
        self.exp.gen(builder);
        builder.push(Inst::Set(Id(self.id.v.clone())));
        builder.push(Inst::Push(Obj::Null));
    }
}

impl Gen for syntax::DefFunc {
    fn gen(&self, builder: &mut Builder) {
        let lambda = syntax::Lambda {
            meta: self.meta.clone(),
            arg: syntax::Arg::Args(syntax::Args {
                meta: self.meta.clone(),
                args: self.args.clone(),
                varg: self.varg.clone(),
            }),
            body: self.body.clone(),
        };

        let set = syntax::DefVar {
            meta: self.meta.clone(),
            id: self.id.clone(),
            exp: syntax::Exp::Lambda(Box::new(lambda)),
        };

        set.gen(builder);
        builder.push(Inst::Push(Obj::Null));
    }
}

impl Gen for syntax::Exp {
    fn gen(&self, builder: &mut Builder) {
        match self {
            Self::Const(t) => t.gen(builder),
            Self::Id(t) => t.gen(builder),
            Self::Lambda(t) => t.gen(builder),
            Self::Apply(t) => t.gen(builder),
            Self::Quote(t) => t.gen(builder),
            Self::Set(t) => t.gen(builder),
            Self::Let(t) => t.gen(builder),
            Self::LetAster(t) => t.gen(builder),
            Self::LetRec(t) => t.gen(builder),
            Self::If(t) => t.gen(builder),
            Self::Cond(t) => t.gen(builder),
            Self::And(t) => t.gen(builder),
            Self::Or(t) => t.gen(builder),
            Self::Begin(t) => t.gen(builder),
            Self::Do(t) => t.gen(builder),
        }
    }
}

impl Gen for syntax::Lambda {
    fn gen(&self, builder: &mut Builder) {
        let lambda_id = builder.get_label();
        let label = builder.get_label();

        builder.push_closure(lambda_id);
        builder.push_jump(label);

        builder.push_label(lambda_id);

        match &self.arg {
            syntax::Arg::Args(args) => {
                for id in args.args.iter().rev() {
                    builder.push(Inst::Def(Id(id.v.clone())));
                    builder.push(Inst::Set(Id(id.v.clone())));
                }

                // TODO: support varg
            }
            syntax::Arg::Id(id) => {
                builder.push(Inst::Def(Id(id.v.clone())));
                builder.push(Inst::Set(Id(id.v.clone())));
            }
        }

        self.body.gen(builder);

        builder.push(Inst::Ret);

        builder.push_label(label);
    }
}

impl Gen for syntax::Apply {
    fn gen(&self, builder: &mut Builder) {
        let builtin_inst = if let syntax::Exp::Id(id) = &self.func {
            match id.v.as_str() {
                "display" => Some(Inst::Display),
                "newline" => Some(Inst::Newline),
                "+" => Some(Inst::Add),
                "-" => Some(Inst::Sub),
                "*" => Some(Inst::Mul),
                "/" => Some(Inst::Div),
                "=" => Some(Inst::Eq),
                "<" => Some(Inst::Lt),
                "<=" => Some(Inst::Le),
                ">" => Some(Inst::Gt),
                ">=" => Some(Inst::Ge),
                "cons" => Some(Inst::Cons),
                "car" => Some(Inst::Car),
                "cdr" => Some(Inst::Cdr),
                "set-car!" => Some(Inst::SetCar),
                "set-cdr!" => Some(Inst::SetCdr),
                _ => None,
            }
        } else {
            None
        };

        let label = builder.get_label();

        if builtin_inst.is_none() {
            builder.push_return_context(label);
        }

        for exp in &self.exps {
            exp.gen(builder);
        }

        if let Some(i) = builtin_inst {
            builder.push(i);
            return;
        }

        self.func.gen(builder);
        builder.push(Inst::Call);

        builder.push_label(label);
    }
}

impl Gen for syntax::Quote {
    fn gen(&self, builder: &mut Builder) {
        self.s_exp.gen(builder);
    }
}

impl Gen for syntax::Set {
    fn gen(&self, builder: &mut Builder) {
        self.exp.gen(builder);
        builder.push(Inst::Set(Id(self.id.v.clone())));
        builder.push(Inst::Push(Obj::Null));
    }
}

impl Gen for syntax::Let {
    fn gen(&self, builder: &mut Builder) {
        let arg = syntax::Arg::Args(syntax::Args {
            meta: self.meta.clone(),
            args: self.bindings.bindings.iter().map(|b| b.id.clone()).collect(),
            varg: None,
        });

        let lambda = if let Some(id) = &self.id {
            syntax::Lambda {
                meta: self.meta.clone(),
                arg,
                body: syntax::Body {
                    meta: self.meta.clone(),
                    defs: vec![syntax::Define::Func(syntax::DefFunc {
                        meta: self.meta.clone(),
                        id: id.clone(),
                        args: self.bindings.bindings.iter().map(|b| b.id.clone()).collect(),
                        varg: None,
                        body: self.body.clone(),
                    })],
                    exps: syntax::NonEmptyVec::new(syntax::Exp::Apply(Box::new(syntax::Apply {
                        meta: self.meta.clone(),
                        func: syntax::Exp::Id(id.clone()),
                        exps: self
                            .bindings
                            .bindings
                            .iter()
                            .map(|b| syntax::Exp::Id(b.id.clone()))
                            .collect(),
                    }))),
                },
            }
        } else {
            syntax::Lambda {
                meta: self.meta.clone(),
                arg,
                body: self.body.clone(),
            }
        };

        let apply = syntax::Apply {
            meta: self.meta.clone(),
            func: syntax::Exp::Lambda(Box::new(lambda)),
            exps: self.bindings.bindings.iter().map(|b| b.exp.clone()).collect(),
        };

        apply.gen(builder);
    }
}

impl Gen for syntax::LetAster {
    fn gen(&self, builder: &mut Builder) {
        todo!()
    }
}

impl Gen for syntax::LetRec {
    fn gen(&self, builder: &mut Builder) {
        todo!()
    }
}

impl Gen for syntax::If {
    fn gen(&self, builder: &mut Builder) {
        todo!()
    }
}

impl Gen for syntax::Cond {
    fn gen(&self, builder: &mut Builder) {
        todo!()
    }
}

impl Gen for syntax::And {
    fn gen(&self, builder: &mut Builder) {
        todo!()
    }
}

impl Gen for syntax::Or {
    fn gen(&self, builder: &mut Builder) {
        todo!()
    }
}

impl Gen for syntax::Begin {
    fn gen(&self, builder: &mut Builder) {
        for (i, exp) in self.exps.iter().enumerate() {
            exp.gen(builder);

            if i < self.exps.len() - 1 {
                builder.push(Inst::Pop);
            }
        }
    }
}

impl Gen for syntax::Do {
    fn gen(&self, builder: &mut Builder) {
        todo!()
    }
}

impl Gen for syntax::Body {
    fn gen(&self, builder: &mut Builder) {
        for def in &self.defs {
            def.gen(builder);
        }

        for exp in self.exps.get() {
            exp.gen(builder);
        }
    }
}

impl Gen for syntax::Arg {
    fn gen(&self, builder: &mut Builder) {
        todo!()
    }
}

impl Gen for syntax::Args {
    fn gen(&self, builder: &mut Builder) {
        todo!()
    }
}

impl Gen for syntax::Bindings {
    fn gen(&self, builder: &mut Builder) {
        todo!()
    }
}

impl Gen for syntax::Binding {
    fn gen(&self, builder: &mut Builder) {
        todo!()
    }
}

impl Gen for syntax::SExp {
    fn gen(&self, builder: &mut Builder) {
        match self {
            Self::Const(t) => t.gen(builder),
            Self::Id(t) => builder.push(Inst::Push(Obj::Id(Id(t.v.clone())))),
            Self::Pair(t) => t.gen(builder),
        }
    }
}

impl Gen for syntax::Pair {
    fn gen(&self, builder: &mut Builder) {
        for (i, exp) in self.exps.iter().enumerate() {
            if i == self.exps.len() - 1 {
                exp.gen(builder);

                if let Some(last) = &self.last {
                    last.gen(builder);
                } else {
                    builder.push(Inst::Push(Obj::Null));
                }
            } else {
                exp.gen(builder);
            }
        }

        for _ in 0..self.exps.len() {
            builder.push(Inst::Cons);
        }
    }
}

impl Gen for syntax::Const {
    fn gen(&self, builder: &mut Builder) {
        match self {
            Self::Num(t) => t.gen(builder),
            Self::Bool(t) => t.gen(builder),
            Self::String(t) => t.gen(builder),
            Self::Null(t) => t.gen(builder),
        }
    }
}

impl Gen for syntax::Bool {
    fn gen(&self, builder: &mut Builder) {
        builder.push(Inst::Push(Obj::Bool(self.v)));
    }
}

impl Gen for syntax::Num {
    fn gen(&self, builder: &mut Builder) {
        builder.push(Inst::Push(Obj::Number(self.v)));
    }
}

impl Gen for syntax::Str {
    fn gen(&self, builder: &mut Builder) {
        builder.push(Inst::Push(Obj::String(self.v.clone())));
    }
}

impl Gen for syntax::Null {
    fn gen(&self, builder: &mut Builder) {
        builder.push(Inst::Push(Obj::Null));
    }
}

impl Gen for syntax::Id {
    fn gen(&self, builder: &mut Builder) {
        builder.push(Inst::Get(Id(self.v.clone())));
    }
}

mod builder {
    use super::*;

    pub struct Builder {
        label: u32,
        insts: Vec<TempInst>,
    }

    #[derive(Debug)]
    pub enum TempInst {
        Raw(Inst),
        Jump(u32),
        CreateClosure(u32),
        PushReturnContext(u32),
        Label(u32),
    }

    impl Builder {
        pub fn new() -> Self {
            Self {
                label: 0,
                insts: vec![],
            }
        }

        pub fn get_label(&mut self) -> u32 {
            self.label += 1;
            self.label
        }

        pub fn push(&mut self, inst: Inst) {
            self.insts.push(TempInst::Raw(inst));
        }

        pub fn push_label(&mut self, label: u32) {
            self.insts.push(TempInst::Label(label));
        }

        pub fn push_jump(&mut self, label: u32) {
            self.insts.push(TempInst::Jump(label));
        }
        pub fn push_return_context(&mut self, label: u32) {
            self.insts.push(TempInst::PushReturnContext(label));
        }
        pub fn push_closure(&mut self, id: u32) {
            self.insts.push(TempInst::CreateClosure(id));
        }

        pub fn build(&self) -> Vec<Inst> {
            let mut pc = 0;
            let mut label_to_pc = std::collections::HashMap::<u32, u32>::new();

            for i in &self.insts {
                let TempInst::Label(l) = i else {
                    pc += 1;
                    continue;
                };

                label_to_pc.insert(*l, pc);
            }

            let mut insts = vec![];

            for i in &self.insts {
                match i {
                    TempInst::Raw(i) => insts.push(i.clone()),
                    TempInst::Jump(i) => insts.push(Inst::Jump(*label_to_pc.get(i).unwrap())),
                    TempInst::CreateClosure(i) => {
                        insts.push(Inst::CreateClosure(*label_to_pc.get(i).unwrap()))
                    }
                    TempInst::PushReturnContext(i) => {
                        insts.push(Inst::PushReturnContext(*label_to_pc.get(i).unwrap()))
                    }
                    TempInst::Label(_) => continue,
                }
            }

            insts
        }
    }
}
