use std::collections::{HashMap, HashSet};

use crate::datastructure::List;
use crate::dependency::SortResult;
use crate::insn::*;
use crate::{ast::*, DEBUG};

struct NodeInfo {
    pointed: List<Id>,
    init: Vec<Insn1>,
    update: Vec<Insn1>,
}

pub struct Compiler {
    codes: Vec<Insn1>,
    node_info: HashMap<Id, NodeInfo>,
    symbol_table: Vec<Id>,
}

#[derive(Debug)]
pub enum CompileErr<'a> {
    IdNotFound(&'a Id),
    CircularRef(Vec<&'a Id>),
}
pub enum CompileResult {
    DefNode(Vec<Insn2>),
    Exp(Vec<Insn2>),
}
pub type CResult<'a> = Result<CompileResult, CompileErr<'a>>;
type EmitResult<'a> = Result<(), CompileErr<'a>>;

impl Compiler {
    pub fn compile<'a, 'b: 'a>(&'b mut self, prog: &'a Program) -> CResult<'a> {
        assert!(self.codes.len() == 0);
        self.register_new_node(prog);
        self.compile_nodes(prog)?;
        let sorted_nodes = match self.topological_sort() {
            SortResult::Success(nd) => nd,
            SortResult::CicularRef(nd) => return Err(CompileErr::CircularRef(nd)),
        };
        if DEBUG {
            print!("[dependency]");
            for id in &sorted_nodes {
                print!(" -> {}", id.s);
            }
            println!("")
        }
        let mut ret = vec![];
        ret.push(Insn2::DeleteNodes);
        for id in &sorted_nodes {
            let NodeInfo {
                pointed: _,
                init,
                update: _,
            } = self.node_info.get(id).unwrap();
            for insn in init {
                Self::to_insn2(insn, &sorted_nodes, &mut ret);
            }
            ret.push(Insn2::AllocNode);
        }
        let n0 = ret.len() as isize;
        for id in &sorted_nodes {
            let NodeInfo {
                pointed: _,
                init: _,
                update,
            } = self.node_info.get(id).unwrap();
            for insn in update {
                Self::to_insn2(insn, &sorted_nodes, &mut ret);
            }
        }
        let n1 = ret.len() as isize;
        ret.push(Insn2::J(n0 - n1));
        if DEBUG {
            println!("----[bytecode]----");
            for insn in &ret {
                println!(" : {:?}", insn);
            }
            println!("------------------")
        }
        Ok(CompileResult::DefNode(ret))
    }

    /*fn add_sym(&mut self, id: Id) {
        self.symbol_table.push(id)
    }*/
    // add nodes to self
    fn register_new_node(&mut self, prog: &Program) {
        match prog {
            Program::Def(def) => {
                self.add_node(def);
                self.add_dependency(def);
            }
            Program::Defs(defs) => {
                for def in defs {
                    self.add_node(def);
                }
                for def in defs {
                    self.add_dependency(def);
                }
            }
            Program::Exp(_) => return,
        };
    }
    fn add_node(&mut self, def: &Def) {
        match def {
            Def::Node {
                name,
                init: _,
                val: _,
            } => {
                self.node_info.insert(
                    name.clone(),
                    NodeInfo {
                        pointed: List::new(),
                        init: vec![],
                        update: vec![],
                    },
                );
            }
            _ => return,
        }
    }
    fn push_insn(&mut self, insn: Insn1) {
        self.codes.push(insn)
    }
    fn add_dependency(&mut self, def: &Def) {
        match def {
            Def::Node { name, init: _, val } => {
                let ptr = &mut self.node_info.get_mut(name).unwrap().pointed as *mut List<Id>;
                let lst = unsafe { ptr.as_mut().unwrap() };
                val.to_dependency(lst, self);
            }
            _ => return,
        }
    }
    fn compile_nodes<'a>(&mut self, prog: &'a Program) -> EmitResult<'a> {
        fn compile_node<'a>(c: &mut Compiler, def: &'a Def) -> EmitResult<'a> {
            match def {
                //todo unregister
                Def::Node { name, init, val } => {
                    assert!(c.codes.len() == 0);
                    init.emit_code(c)?;
                    let init = c.insn_popall();
                    val.emit_code(c)?;
                    let update = c.insn_popall();

                    let info = c.node_info.get_mut(name).unwrap();
                    info.init = init;
                    info.update = update;
                    Ok(())
                }
                Def::Data { .. } => Ok(()),
                Def::Func { .. } => Ok(()),
            }
        }
        match prog {
            Program::Defs(defs) => {
                for def in defs {
                    compile_node(self, def)?;
                }
                Ok(())
            }
            Program::Def(def) => compile_node(self, def),
            Program::Exp(_) => Ok(()),
        }
    }

    fn find_sym(&self, id: &Id) -> Insn1 {
        let tbl_n = self.symbol_table.len();
        for i in (0..tbl_n).rev() {
            if id == &self.symbol_table[i] {
                return Insn1::GetLocal(i);
            }
        }
        if self.node_info.contains_key(id) {
            return Insn1::GetNode(id.clone());
        }
        todo!() //Global Variable
    }
    pub fn new() -> Self {
        Compiler {
            codes: vec![],
            node_info: HashMap::new(),
            symbol_table: vec![],
        }
    }
    fn insn_popall(&mut self) -> Vec<Insn1> {
        let mut new = vec![];
        std::mem::swap(self.codes.as_mut(), &mut new);
        new
    }
    fn topological_sort(&self) -> SortResult {
        fn dfs(cur: &Id, par: &Id, finished: &mut HashSet<Id>, seen: &mut HashSet<Id>) {}
        todo!()
    }

    fn to_insn2(insn: &Insn1, sorted_nodes: &Vec<&Id>, res: &mut Vec<Insn2>) {
        let insn2 = match insn {
            Insn1::Add => Insn2::Add,
            Insn1::Je(i) => Insn2::Je(*i),
            Insn1::J(i) => Insn2::J(*i),
            Insn1::Mul => Insn2::Mul,
            Insn1::Int(i) => Insn2::Int(*i),
            Insn1::Bool(b) => Insn2::Bool(*b),
            Insn1::Print => Insn2::Print,
            Insn1::GetLocal(i) => Insn2::GetLocal(*i),
            Insn1::SetLocal(i) => Insn2::SetLocal(*i),
            Insn1::AllocNode => Insn2::AllocNode,
            Insn1::AllocNodes(i) => Insn2::AllocNodes(*i),
            Insn1::GetNode(id) => Insn2::GetNode(Self::find_nd(id, sorted_nodes).unwrap()),
            Insn1::SetNode(id) => Insn2::SetNode(Self::find_nd(id, sorted_nodes).unwrap()),
            Insn1::Exit => Insn2::Exit,
            Insn1::Placeholder => unreachable!(), //placeholder must be replaced at this point
        };
        res.push(insn2);
    }
    fn find_nd(id: &Id, sorted_nodes: &Vec<&Id>) -> Option<usize> {
        for (i, &id2) in sorted_nodes.iter().enumerate() {
            if id == id2 {
                return Some(i);
            }
        }
        None
    }
}

impl Exp {
    pub fn emit_code<'a>(&'a self, c: &mut Compiler) -> EmitResult<'a> {
        match self {
            Exp::If { .. } => todo!(),
            Exp::Add(e, t) => {
                e.emit_code(c)?;
                t.emit_code(c)?;
                c.push_insn(Insn1::Add);
                Ok(())
            }
            Exp::Term(t) => t.emit_code(c),
        }
    }
    fn to_dependency(&self, lst: &mut List<Id>, cmp: &Compiler) {
        match self {
            Exp::If { cond, then, els } => {
                cond.to_dependency(lst, cmp);
                then.to_dependency(lst, cmp);
                els.to_dependency(lst, cmp);
            }
            Exp::Add(e, t) => {
                e.to_dependency(lst, cmp);
                t.to_dependency(lst, cmp);
            }
            Exp::Term(t) => t.to_dependency(lst, cmp),
        }
    }
}
impl Term {
    fn emit_code<'a>(&'a self, c: &mut Compiler) -> EmitResult<'a> {
        match self {
            Term::Mul(t1, t2) => {
                t1.emit_code(c)?;
                t2.emit_code(c)?;
                c.push_insn(Insn1::Mul);
            }
            Term::Int(i) => c.push_insn(Insn1::Int(*i)),
            Term::FnCall(_, _) => todo!(),
            Term::Bool(b) => c.push_insn(Insn1::Bool(*b)),
            Term::Last(_) => todo!(),
            Term::Id(id) => c.push_insn(c.find_sym(id)),
        }
        Ok(())
    }
    fn to_dependency(&self, lst: &mut List<Id>, c: &Compiler) {
        match self {
            Term::Mul(t1, t2) => {
                t1.to_dependency(lst, c);
                t2.to_dependency(lst, c);
            }
            Term::Int(_) => return,
            Term::FnCall(_, args) => {
                for arg in args {
                    arg.to_dependency(lst, c);
                }
            }
            Term::Bool(_) => return,

            // node left_variable = (idの式)
            Term::Id(id) => {
                if c.node_info.contains_key(id) {
                    lst.push(id.clone())
                } else {
                    // idがnodeではない(dataなど)
                }
            }
            Term::Last(_) => return,
        }
    }
}

// init -> node領域確保 -> loop {save last -> update}

// a b c a@last b@last
