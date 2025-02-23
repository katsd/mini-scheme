use crate::syntax;
use crate::obj::*;
use crate::vm::Inst;
use builder::{Builder, TempInst};

pub struct CodeGen {
    builder: Builder,
}

impl CodeGen {
    pub fn new() -> Self {
        Self {
            builder: Builder::new(),
        }
    }

    pub fn generate(&mut self, ast: &syntax::AST, is_main: bool) -> Vec<Inst> {
        self.builder.init();

        for t in &ast.body {
            t.gen(&mut self.builder, false);
        }

        if is_main {
            self.builder.push(Inst::Exit);
        }

        self.builder.build()
    }
}

pub fn join(l: Vec<Inst>, r: Vec<Inst>) -> Vec<Inst> {
    let len_l = l.len();
    let len_r = r.len();

    let mut insts = l;
    insts.extend(r);

    for i in len_l..(len_l + len_r) {
        let inst = insts.get_mut(i).unwrap();

        match inst {
            Inst::Jump(a) => *a += len_l as u32,
            Inst::JumpIf(a) => *a += len_l as u32,
            Inst::CreateClosure(a) => *a += len_l as u32,
            Inst::PushReturnContext(a) => *a += len_l as u32,
            _ => (),
        }
    }

    insts
}

impl Id {
    fn new(id: &syntax::Id, builder: &Builder) -> Self {
        let id_ctx = builder.get_true_id_ctx(&id.v, id.id_ctx);

        if id_ctx == 0 {
            Self(id.v.clone())
        } else {
            Self(format!("{}~{}", id.v, id_ctx))
        }
    }
}

trait Gen {
    fn gen(&self, builder: &mut Builder, is_tail: bool);
}

impl Gen for syntax::Toplevel {
    fn gen(&self, builder: &mut Builder, is_tail: bool) {
        match &self {
            syntax::Toplevel::DefineSyntax => (),
            syntax::Toplevel::Exp(t) => t.gen(builder, false),
            syntax::Toplevel::Define(t) => t.gen(builder, false),
            syntax::Toplevel::Load(t) => t.gen(builder, false),
        }
    }
}

impl Gen for syntax::Load {
    fn gen(&self, builder: &mut Builder, is_tail: bool) {
        self.src.gen(builder, false);
        builder.push(Inst::Load);
    }
}

impl Gen for syntax::Define {
    fn gen(&self, builder: &mut Builder, is_tail: bool) {
        match self {
            Self::Var(t) => t.gen(builder, false),
            Self::Func(t) => t.gen(builder, false),
        }
    }
}

impl Gen for syntax::DefVar {
    fn gen(&self, builder: &mut Builder, is_tail: bool) {
        builder.def(&self.id.v, self.id.id_ctx);
        builder.push(Inst::Def(Id::new(&self.id, builder)));
        self.exp.gen(builder, false);
        builder.push(Inst::Set(Id::new(&self.id, builder)));
        builder.push(Inst::Push(Obj::Null));
    }
}

impl Gen for syntax::DefFunc {
    fn gen(&self, builder: &mut Builder, is_tail: bool) {
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

        set.gen(builder, false);
        builder.push(Inst::Push(Obj::Null));
    }
}

impl Gen for syntax::Exp {
    fn gen(&self, builder: &mut Builder, is_tail: bool) {
        match self {
            Self::Const(t) => t.gen(builder, is_tail),
            Self::Id(t) => t.gen(builder, is_tail),
            Self::Lambda(t) => t.gen(builder, is_tail),
            Self::Apply(t) => t.gen(builder, is_tail),
            Self::Quote(t) => t.gen(builder, is_tail),
            Self::Set(t) => t.gen(builder, is_tail),
            Self::Let(t) => t.gen(builder, is_tail),
            Self::LetAster(t) => t.gen(builder, is_tail),
            Self::LetRec(t) => t.gen(builder, is_tail),
            Self::If(t) => t.gen(builder, is_tail),
            Self::Cond(t) => t.gen(builder, is_tail),
            Self::And(t) => t.gen(builder, is_tail),
            Self::Or(t) => t.gen(builder, is_tail),
            Self::Begin(t) => t.gen(builder, is_tail),
            Self::Do(t) => t.gen(builder, is_tail),
        }
    }
}

impl Gen for syntax::Lambda {
    fn gen(&self, builder: &mut Builder, is_tail: bool) {
        builder.enter_new_scope();

        let lambda_id = builder.get_label();
        let label = builder.get_label();

        builder.push_temp(TempInst::CreateClosure(lambda_id));
        builder.push_temp(TempInst::Jump(label));

        builder.push_label(lambda_id);

        match &self.arg {
            syntax::Arg::Args(args) => {
                for id in args.args.iter() {
                    builder.def(&id.v, id.id_ctx);
                    builder.push(Inst::Def(Id::new(id, builder)));
                    builder.push(Inst::Set(Id::new(id, builder)));
                }

                if let Some(id) = &args.varg {
                    builder.def(&id.v, id.id_ctx);
                    builder.push(Inst::Def(Id::new(id, builder)));
                    builder.push(Inst::CollectVArg(Id::new(id, builder)));
                    builder.push(Inst::Set(Id::new(id, builder)));
                    builder.push(Inst::Push(Obj::Null));
                }
            }
            syntax::Arg::VArg(id) => {
                builder.def(&id.v, id.id_ctx);
                builder.push(Inst::Def(Id::new(id, builder)));
                builder.push(Inst::CollectVArg(Id::new(id, builder)));
                builder.push(Inst::Set(Id::new(id, builder)));
                builder.push(Inst::Push(Obj::Null));
            }
        }

        self.body.gen(builder, true);

        builder.push(Inst::Ret);

        builder.push_label(label);

        builder.exit_cur_scope();
    }
}

impl Gen for syntax::Apply {
    fn gen(&self, builder: &mut Builder, is_tail: bool) {
        let is_apply = if let syntax::Exp::Id(id) = &self.func {
            id.v.as_str() == "apply"
        } else {
            false
        };

        let func = if is_apply {
            self.exps.get(0).as_ref().unwrap()
        } else {
            &self.func
        };

        let builtin_inst = if let syntax::Exp::Id(id) = func {
            match id.v.as_str() {
                "display" => Some(Inst::Display),
                "~+" => Some(Inst::Add),
                "~-" => Some(Inst::Sub),
                "~*" => Some(Inst::Mul),
                "~/" => Some(Inst::Div),
                "~=" => Some(Inst::Eq),
                "~<" => Some(Inst::Lt),
                "~<=" => Some(Inst::Le),
                "~>" => Some(Inst::Gt),
                "~>=" => Some(Inst::Ge),
                "not" => Some(Inst::Not),
                "cons" => Some(Inst::Cons),
                "car" => Some(Inst::Car),
                "cdr" => Some(Inst::Cdr),
                "set-car!" => Some(Inst::SetCar),
                "set-cdr!" => Some(Inst::SetCdr),
                "null?" => Some(Inst::IsNull),
                "pair?" => Some(Inst::IsPair),
                "number?" => Some(Inst::IsNumber),
                "boolean?" => Some(Inst::IsBool),
                "string?" => Some(Inst::IsString),
                "proc?" => Some(Inst::IsProc),
                "symbol?" => Some(Inst::IsSymbol),
                "eq?" => Some(Inst::IsEq),
                "equal?" => Some(Inst::IsEqual),
                "symbol->string" => Some(Inst::SymToStr),
                "string->symbol" => Some(Inst::StrToSym),
                "string->number" => Some(Inst::StrToNum),
                "number->string" => Some(Inst::NumToStr),
                "~string-append" => Some(Inst::StringAppend),
                _ => None,
            }
        } else {
            None
        };

        let label = builder.get_label();

        if !is_tail && builtin_inst.is_none() {
            builder.push_temp(TempInst::PushReturnContext(label));
        }

        if is_apply {
            for (i, exp) in self.exps.iter().enumerate().rev() {
                if i == 0 {
                    break;
                }

                exp.gen(builder, false);

                if i == self.exps.len() - 1 {
                    builder.push(Inst::ExpandList);
                }
            }
        } else {
            for exp in self.exps.iter().rev() {
                exp.gen(builder, false);
            }
        }

        if let Some(i) = builtin_inst {
            builder.push(i);
            return;
        }

        func.gen(builder, false);

        builder.push(if is_tail { Inst::OptCall } else { Inst::Call });

        builder.push_label(label);
    }
}

impl Gen for syntax::Quote {
    fn gen(&self, builder: &mut Builder, is_tail: bool) {
        self.s_exp.gen(builder, false);
    }
}

impl Gen for syntax::Set {
    fn gen(&self, builder: &mut Builder, is_tail: bool) {
        self.exp.gen(builder, false);
        builder.push(Inst::Set(Id::new(&self.id, builder)));
        builder.push(Inst::Push(Obj::Null));
    }
}

impl Gen for syntax::Let {
    fn gen(&self, builder: &mut Builder, is_tail: bool) {
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

        apply.gen(builder, false);
    }
}

impl Gen for syntax::LetAster {
    fn gen(&self, builder: &mut Builder, is_tail: bool) {
        let mut t = syntax::Let {
            meta: self.meta.clone(),
            id: None,
            bindings: syntax::Bindings {
                meta: self.meta.clone(),
                bindings: vec![],
            },
            body: self.body.clone(),
        };

        for b in self.bindings.bindings.iter().rev() {
            t = syntax::Let {
                meta: self.meta.clone(),
                id: None,
                bindings: syntax::Bindings {
                    meta: self.meta.clone(),
                    bindings: vec![b.clone()],
                },
                body: syntax::Body {
                    meta: self.meta.clone(),
                    defs: vec![],
                    exps: syntax::NonEmptyVec::new(syntax::Exp::Let(Box::new(t))),
                },
            };
        }

        t.gen(builder, false);
    }
}

impl Gen for syntax::LetRec {
    fn gen(&self, builder: &mut Builder, is_tail: bool) {
        let bindings = self
            .bindings
            .bindings
            .iter()
            .map(|b| syntax::Binding {
                meta: b.meta.clone(),
                id: b.id.clone(),
                exp: syntax::Exp::Const(syntax::Const::Null(syntax::Null {
                    meta: b.meta.clone(),
                })),
            })
            .collect::<Vec<_>>();

        let mut exps = self.body.exps.clone();

        for b in &self.bindings.bindings {
            exps.insert(
                0,
                syntax::Exp::Set(Box::new(syntax::Set {
                    meta: b.meta.clone(),
                    id: b.id.clone(),
                    exp: b.exp.clone(),
                })),
            );
        }

        syntax::Let {
            meta: self.meta.clone(),
            id: None,
            bindings: syntax::Bindings {
                meta: self.bindings.meta.clone(),
                bindings,
            },
            body: syntax::Body {
                meta: self.body.meta.clone(),
                defs: self.body.defs.clone(),
                exps,
            },
        }
        .gen(builder, false);
    }
}

impl Gen for syntax::If {
    fn gen(&self, builder: &mut Builder, is_tail: bool) {
        self.cond.gen(builder, false);
        builder.push(Inst::Not);

        let label_else = builder.get_label();

        builder.push_temp(TempInst::JumpIf(label_else));

        self.then.gen(builder, is_tail);

        let label_exit = builder.get_label();
        builder.push_temp(TempInst::Jump(label_exit));
        builder.push_label(label_else);

        if let Some(el) = &self.el {
            el.gen(builder, is_tail);
        } else {
            builder.push(Inst::Push(Obj::Null));
        }

        builder.push_label(label_exit);
    }
}

impl Gen for syntax::Cond {
    fn gen(&self, builder: &mut Builder, is_tail: bool) {
        let label_exit = builder.get_label();

        for m in &self.matches {
            m.cond.gen(builder, false);
            builder.push(Inst::Not);

            let label = builder.get_label();

            builder.push_temp(TempInst::JumpIf(label));

            for (i, exp) in m.then.get().iter().enumerate() {
                exp.gen(builder, is_tail);

                if i < m.then.len() - 1 {
                    builder.push(Inst::Pop);
                }
            }

            builder.push_temp(TempInst::Jump(label_exit));
            builder.push_label(label);
        }

        if let Some(el) = &self.el {
            for (i, exp) in el.get().iter().enumerate() {
                exp.gen(builder, is_tail);

                if i < el.len() - 1 {
                    builder.push(Inst::Pop);
                }
            }
        } else {
            builder.push(Inst::Push(Obj::Null));
        }

        builder.push_label(label_exit);
    }
}

impl Gen for syntax::And {
    fn gen(&self, builder: &mut Builder, is_tail: bool) {
        let label_exit = builder.get_label();

        for (i, exp) in self.exps.iter().enumerate() {
            if i == self.exps.len() - 1 {
                exp.gen(builder, is_tail);
                continue;
            }

            exp.gen(builder, false);
            builder.push(Inst::Dup);
            builder.push(Inst::Not);
            builder.push_temp(TempInst::JumpIf(label_exit));
            builder.push(Inst::Pop);
        }

        builder.push_label(label_exit);
    }
}

impl Gen for syntax::Or {
    fn gen(&self, builder: &mut Builder, is_tail: bool) {
        let label_exit = builder.get_label();

        for (i, exp) in self.exps.iter().enumerate() {
            if i == self.exps.len() - 1 {
                exp.gen(builder, is_tail);
                continue;
            }

            exp.gen(builder, false);
            builder.push(Inst::Dup);
            builder.push_temp(TempInst::JumpIf(label_exit));
            builder.push(Inst::Pop);
        }

        builder.push_label(label_exit);
    }
}

impl Gen for syntax::Begin {
    fn gen(&self, builder: &mut Builder, is_tail: bool) {
        for (i, exp) in self.exps.iter().enumerate() {
            let is_last = i == self.exps.len() - 1;

            exp.gen(builder, is_tail && is_last);

            if !is_last {
                builder.push(Inst::Pop);
            }
        }
    }
}

impl Gen for syntax::Do {
    fn gen(&self, builder: &mut Builder, is_tail: bool) {
        builder.enter_new_scope();

        let label_start = builder.get_label();
        let label_exit = builder.get_label();
        let lambda_id = builder.get_label();
        let label_lambda_exit = builder.get_label();

        builder.push_temp(TempInst::PushReturnContext(label_exit));
        builder.push_temp(TempInst::CreateClosure(lambda_id));
        builder.push_temp(TempInst::Jump(label_lambda_exit));

        builder.push_label(lambda_id);

        for b in &self.bindings {
            syntax::DefVar {
                meta: b.meta.clone(),
                id: b.id.clone(),
                exp: b.i.clone(),
            }
            .gen(builder, false);
            builder.push(Inst::Pop);
        }

        builder.push_label(label_start);

        self.cond.gen(builder, false);

        let label_ret = builder.get_label();
        builder.push_temp(TempInst::JumpIf(label_ret));

        self.body.gen(builder, false);
        builder.push(Inst::Pop);

        for b in &self.bindings {
            syntax::Set {
                meta: b.meta.clone(),
                id: b.id.clone(),
                exp: b.u.clone(),
            }
            .gen(builder, false);
            builder.push(Inst::Pop);
        }

        builder.push_temp(TempInst::Jump(label_start));

        builder.push_label(label_ret);

        for (i, v) in self.value.iter().enumerate() {
            v.gen(builder, false);

            if i < self.value.len() - 1 {
                builder.push(Inst::Pop);
            }
        }

        builder.push(Inst::Ret);
        builder.push_label(label_lambda_exit);

        builder.push(Inst::Call);
        builder.push_label(label_exit);

        builder.exit_cur_scope();
    }
}

impl Gen for syntax::Body {
    fn gen(&self, builder: &mut Builder, is_tail: bool) {
        for def in &self.defs {
            def.gen(builder, false);
            builder.push(Inst::Pop);
        }

        for (i, exp) in self.exps.get().iter().enumerate() {
            let is_last = i == self.exps.len() - 1;

            exp.gen(builder, is_tail && is_last);

            if !is_last {
                builder.push(Inst::Pop);
            }
        }
    }
}

impl Gen for syntax::Arg {
    fn gen(&self, builder: &mut Builder, is_tail: bool) {
        todo!()
    }
}

impl Gen for syntax::Args {
    fn gen(&self, builder: &mut Builder, is_tail: bool) {
        todo!()
    }
}

impl Gen for syntax::Bindings {
    fn gen(&self, builder: &mut Builder, is_tail: bool) {
        todo!()
    }
}

impl Gen for syntax::Binding {
    fn gen(&self, builder: &mut Builder, is_tail: bool) {
        todo!()
    }
}

impl Gen for syntax::SExp {
    fn gen(&self, builder: &mut Builder, is_tail: bool) {
        match self {
            Self::Const(t) => t.gen(builder, false),
            Self::Id(t) => builder.push(Inst::Push(Obj::Id(Id::new(t, builder)))),
            Self::Pair(t) => t.gen(builder, false),
        }
    }
}

impl Gen for syntax::Pair {
    fn gen(&self, builder: &mut Builder, is_tail: bool) {
        if let Some(last) = &self.last {
            last.gen(builder, false);
        } else {
            builder.push(Inst::Push(Obj::Null));
        }

        for exp in self.exps.iter().rev() {
            exp.gen(builder, false);
            builder.push(Inst::Cons);
        }
    }
}

impl Gen for syntax::Const {
    fn gen(&self, builder: &mut Builder, is_tail: bool) {
        match self {
            Self::Num(t) => t.gen(builder, false),
            Self::Bool(t) => t.gen(builder, false),
            Self::String(t) => t.gen(builder, false),
            Self::Null(t) => t.gen(builder, false),
        }
    }
}

impl Gen for syntax::Bool {
    fn gen(&self, builder: &mut Builder, is_tail: bool) {
        builder.push(Inst::Push(Obj::Bool(self.v)));
    }
}

impl Gen for syntax::Num {
    fn gen(&self, builder: &mut Builder, is_tail: bool) {
        builder.push(Inst::Push(Obj::Number(self.v)));
    }
}

impl Gen for syntax::Str {
    fn gen(&self, builder: &mut Builder, is_tail: bool) {
        builder.push(Inst::Push(Obj::String(self.v.clone())));
    }
}

impl Gen for syntax::Null {
    fn gen(&self, builder: &mut Builder, is_tail: bool) {
        builder.push(Inst::Push(Obj::Null));
    }
}

impl Gen for syntax::Id {
    fn gen(&self, builder: &mut Builder, is_tail: bool) {
        builder.push(Inst::Get(Id::new(&self, builder)));
    }
}

mod builder {
    use std::collections::HashMap;
    use super::*;

    pub struct Builder {
        label: u32,
        insts: Vec<TempInst>,

        id_table: HashMap<String, Vec<u32>>,
        id_def_history: Vec<Vec<String>>,
    }

    #[derive(Debug)]
    pub enum TempInst {
        Raw(Inst),
        Jump(u32),
        JumpIf(u32),
        CreateClosure(u32),
        PushReturnContext(u32),
        Label(u32),
    }

    impl Builder {
        pub fn new() -> Self {
            Self {
                label: 0,
                insts: vec![],

                id_table: Default::default(),
                id_def_history: vec![vec![]],
            }
        }

        pub fn init(&mut self) {
            self.label = 0;
            self.insts = vec![];
        }

        pub fn get_label(&mut self) -> u32 {
            self.label += 1;
            self.label
        }

        pub fn push(&mut self, inst: Inst) {
            self.insts.push(TempInst::Raw(inst));
        }

        pub fn push_temp(&mut self, inst: TempInst) {
            self.insts.push(inst);
        }

        pub fn push_label(&mut self, label: u32) {
            self.insts.push(TempInst::Label(label));
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
                    TempInst::JumpIf(i) => insts.push(Inst::JumpIf(*label_to_pc.get(i).unwrap())),
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

        pub fn enter_new_scope(&mut self) {
            self.id_def_history.push(vec![]);
        }

        pub fn exit_cur_scope(&mut self) {
            let history = self.id_def_history.pop().unwrap();

            for id in history {
                if let Some(table) = self.id_table.get_mut(&id) {
                    let _ = table.pop();
                }
            }
        }

        pub fn def(&mut self, id: &String, id_ctx: u32) {
            self.id_def_history.last_mut().unwrap().push(id.clone());

            if let Some(table) = self.id_table.get_mut(id) {
                table.push(id_ctx);
            } else {
                self.id_table.insert(id.clone(), vec![id_ctx]);
            }
        }

        pub fn get_true_id_ctx(&self, id: &String, id_ctx: u32) -> u32 {
            let Some(table) = self.id_table.get(id) else {
                return 0;
            };

            if table.contains(&id_ctx) {
                id_ctx
            } else {
                *table.last().unwrap_or(&0)
            }
        }
    }
}
