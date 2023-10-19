use crate::ast::*;
use crate::insn::Insn;
use crate::qstr::*;

// how to handle error
// calc codesize first
struct Compiler {
    codes: Vec<Insn>,
    qstr_pool: QstrPool,
}
enum CompileErr {
    IdNotFound,
}
type CResult = Result<(), CompileErr>;

pub fn compile(top: &Top) -> Result<Vec<Insn>, String> {
    let mut c = Compiler::new();
    match c.top(top) {
        Ok(()) => Ok(c.codes),
        Err(errkind) => match errkind {
            CompileErr::IdNotFound => Err("id not found".to_string()),
        },
    }
}

impl Compiler {
    fn new() -> Self {
        Compiler {
            codes: vec![],
            qstr_pool: QstrPool::empty(),
        }
    }
    fn push_insn(&mut self, insn: Insn) {
        self.codes.push(insn)
    }
    // compile top
    fn top(&mut self, top: &Top) -> CResult {
        match top {
            Top::Defs(defs) => {
                for def in defs {
                    self.def(def)?;
                }
                Ok(())
            }
            Top::Exp(exp) => self.exp(exp),
        }
    }
    fn exp(&mut self, exp: &Exp) -> CResult {
        match exp {
            Exp::Add(e, t) => {
                self.exp(&*e)?;
                self.term(&*t)?;
                self.push_insn(Insn::Add);
                Ok(())
            }
            Exp::If { cond, then, els } => {
                todo!()
            }

            Exp::Term(t) => self.term(t),
        }
    }
    fn def(&mut self, def: &Def) -> CResult {
        todo!()
    }
    fn term(&mut self, term: &Term) -> CResult {
        match term {
            Term::Mul(t1, t2) => {
                self.term(&*t1)?;
                self.term(&*t2)?;
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
            Term::FnCall(id, args) => {
                todo!()
            }
            Term::Id(id) => todo!(),
        }
    }
}
