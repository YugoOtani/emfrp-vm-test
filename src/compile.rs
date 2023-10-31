use std::collections::{HashMap, VecDeque};

use crate::ast::*;
use crate::insn::Insn;

//TODOS
// how to handle error
// calc codesize, tablesize first
// initail value of node
//  when to add variable name (in def) to symbol_table
// name duplication?
struct Compiler<'a> {
    codes: Vec<Insn>,
    symbol_table: HashMap<Id<'a>, usize>,
}
#[derive(Debug)]
pub enum CompileErr {
    IdNotFound(String),
}
type CResult = Result<(), CompileErr>;

pub fn compile(prog: &Program) -> Result<Vec<Insn>, CompileErr> {
    let mut c = Compiler::new();
    match c.compile_prog(prog) {
        Ok(()) => Ok(c.codes),
        Err(errkind) => Err(errkind),
    }
}

impl<'a> Compiler<'a> {
    fn add_sym(&mut self, id: Id<'a>) {
        let i = self.symbol_table.len();
        self.symbol_table.insert(id, i);
    }
    fn new() -> Self {
        Compiler {
            codes: vec![],
            symbol_table: HashMap::new(),
        }
    }
    fn push_insn(&mut self, insn: Insn) {
        self.codes.push(insn)
    }

    fn compile_prog(&mut self, prog: &'a Program) -> CResult {
        match prog {
            Program::Repl(Repl::Def(def)) => {
                todo!()
            }
            Program::Repl(Repl::Exp(exp)) => {
                self.compile_exp(exp)?;
                self.push_insn(Insn::Exit);
                Ok(())
            }
            Program::File(defs) => todo!(),
        }
    }
    fn compile_exp(&mut self, exp: &Exp) -> CResult {
        match exp {
            Exp::Add(e, t) => {
                self.compile_exp(&*e)?;
                self.compile_term(&*t)?;
                self.push_insn(Insn::Add);
                Ok(())
            }
            Exp::If { .. } => {
                todo!()
            }

            Exp::Term(t) => self.compile_term(t),
        }
    }

    fn compile_term(&mut self, term: &Term) -> CResult {
        match term {
            Term::Mul(t1, t2) => {
                self.compile_term(&*t1)?;
                self.compile_term(&*t2)?;
                self.push_insn(Insn::Mul);
                Ok(())
            }
            Term::Int(i) => {
                self.push_insn(Insn::Int(*i));
                Ok(())
            }
            Term::Bool(b) => {
                self.push_insn(Insn::Bool(*b));
                Ok(())
            }
            Term::FnCall(..) => {
                todo!()
            }
            Term::Id(id) | Term::Last(id) => match self.symbol_table.get(id) {
                Some(offset) => {
                    self.push_insn(Insn::GetLocal(*offset));
                    Ok(())
                }
                None => Err(CompileErr::IdNotFound(String::from(id.s))),
            },
        }
    }
}
impl<'a> Def<'a> {
    fn ids(&'a self, v: &mut Vec<Id<'a>>) {
        match self {
            Def::Node { name, init: _, val } => {
                v.push(name.clone());
                val.ids(v);
            }
            Def::Data { name, val } => {
                v.push(name.clone());
                val.ids(v);
            }
            Def::Func { .. } => todo!(),
        }
    }
    fn check_dependency(&'a self, mp: &mut HashMap<Id<'a>, Vec<Id<'a>>>) {
        match self {
            Def::Node { name, init: _, val } => {
                val.check_dependency(mp, name);
            }
            Def::Data { name, val } => {
                val.check_dependency(mp, name);
            }
            Def::Func { .. } => todo!(),
        }
    }
}
impl<'a> Exp<'a> {
    fn ids(&'a self, v: &mut Vec<Id<'a>>) {
        match self {
            Exp::If { .. } => return,
            Exp::Add(e, t) => {
                e.ids(v);
                t.ids(v);
            }
            Exp::Term(t) => t.ids(v),
        }
    }
    fn check_dependency(&'a self, mp: &mut HashMap<Id<'a>, Vec<Id<'a>>>, id: &Id<'a>) {
        match self {
            Exp::If { .. } => return,
            Exp::Add(e, t) => {
                e.check_dependency(mp, id);
                t.check_dependency(mp, id);
            }
            Exp::Term(t) => t.check_dependency(mp, id),
        }
    }
}
impl<'a> Term<'a> {
    fn ids(&'a self, v: &mut Vec<Id<'a>>) {
        match self {
            Term::Mul(t1, t2) => {
                t1.ids(v);
                t2.ids(v);
            }
            Term::Int(_) => return,
            Term::FnCall(_, _) => todo!(),
            Term::Bool(_) => return,
            Term::Last(id) => v.push(id.clone()),
            Term::Id(id) => v.push(id.clone()),
        }
    }
    fn check_dependency(&'a self, mp: &mut HashMap<Id<'a>, Vec<Id<'a>>>, id: &Id<'a>) {
        match self {
            Term::Mul(t1, t2) => {
                t1.check_dependency(mp, id);
                t2.check_dependency(mp, id);
            }
            Term::Int(_) => return,
            Term::FnCall(_, _) => return,
            Term::Bool(_) => return,
            Term::Last(id2) => {
                if id != id2 {
                    mp.get_mut(id).unwrap().push(id2.clone());
                }
            }
            Term::Id(id2) => {
                mp.get_mut(id2).unwrap().push(id.clone());
            }
        }
    }
}
