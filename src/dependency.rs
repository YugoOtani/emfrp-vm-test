use crate::ast::*;

struct Dependency<'a> {
    before: Id<'a>,
    after: Id<'a>,
}
impl<'a> Dependency<'a> {
    fn from_program(prog: &'a Program) -> Vec<Dependency<'a>> {
        //let mut v = vec![];
        match prog {
            Program::File(defs) => todo!(),
            Program::Repl(Repl::Exp(exp)) => todo!(),
            Program::Repl(Repl::Def(def)) => todo!(),
        }
    }
    fn from_def(def: &'a Def, v: &mut Vec<Dependency>) {
        match def {
            Def::Node { name, init, val } => todo!(),
            Def::Data { name, val } => todo!(),
            Def::Func { name, params, body } => todo!(),
        }
    }
}
