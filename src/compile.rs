use std::collections::{HashMap, HashSet};

use crate::dependency::topological_sort;
use crate::insn::*;
use crate::{ast::*, DEBUG};

//TODOS
// how to handle error
// calc codesize, tablesize first
// initail value of node
//  when to add variable name (in def) to symbol_table
// name duplication?
pub struct Compiler {
    codes: Vec<Insn1>,
    nodes: HashSet<Id>,
    symbol_table: Vec<Id>,
    pointed: HashMap<Id, HashSet<Id>>,
    compiled_node: HashMap<Id, CompiledNode>,
}
struct CompiledNode {
    init: Vec<Insn1>,
    update: Vec<Insn1>,
}
#[derive(Debug)]
pub enum CompileErr {
    IdNotFound(String),
}
pub enum CompileResult {
    DefNode(Vec<Insn2>),
    Exp(Vec<Insn2>),
}
pub type CResult = Result<CompileResult, CompileErr>;
type EmitResult = Result<(), CompileErr>;

impl Compiler {
    pub fn compile(&mut self, prog: &Program) -> CResult {
        assert!(self.codes.len() == 0);
        self.register_new_node(prog);
        self.compile_node(prog)?;
        let sorted_nodes = topological_sort(&self.pointed);
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
            let CompiledNode { init, update: _ } = self.compiled_node.get(id).unwrap();
            for insn in init {
                Self::to_insn2(insn, &sorted_nodes, &mut ret);
            }
            ret.push(Insn2::AllocNode);
        }
        let n0 = ret.len() as isize;
        for id in &sorted_nodes {
            let CompiledNode { init: _, update } = self.compiled_node.get(id).unwrap();
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
                self.nodes.insert(name.clone());
                self.pointed.insert(name.clone(), HashSet::new());
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
                val.clone()
                    .to_dependency(self.pointed.get_mut(name).unwrap(), &self.nodes);
            }
            _ => return,
        }
    }
    fn compile_node(&mut self, prog: &Program) -> EmitResult {
        fn compile_def(c: &mut Compiler, def: &Def) -> EmitResult {
            match def {
                //todo unregister
                Def::Node { name, init, val } => {
                    assert!(c.codes.len() == 0);
                    init.emit_code(c)?;
                    let init = c.insn_popall();
                    val.emit_code(c)?;
                    let update = c.insn_popall();
                    c.compiled_node
                        .insert(name.clone(), CompiledNode { init, update });
                    Ok(())
                }
                Def::Data { .. } => Ok(()),
                Def::Func { .. } => Ok(()),
            }
        }
        match prog {
            Program::Defs(defs) => {
                for def in defs {
                    compile_def(self, def)?;
                }
                Ok(())
            }
            Program::Def(def) => compile_def(self, def),
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
        if self.nodes.contains(id) {
            return Insn1::GetNode(id.clone());
        }
        todo!() //Global Variable
    }
    pub fn new() -> Self {
        Compiler {
            codes: vec![],
            nodes: HashSet::new(),
            symbol_table: vec![],
            pointed: HashMap::new(),
            compiled_node: HashMap::new(),
        }
    }
    pub fn insn_popall(&mut self) -> Vec<Insn1> {
        let mut new = vec![];
        std::mem::swap(self.codes.as_mut(), &mut new);
        new
    }

    pub fn to_insn2(insn: &Insn1, sorted_nodes: &Vec<&Id>, res: &mut Vec<Insn2>) {
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
    pub fn emit_code(&self, c: &mut Compiler) -> EmitResult {
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
}
impl Term {
    fn emit_code(&self, c: &mut Compiler) -> EmitResult {
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
}

// init -> node領域確保 -> loop {save last -> update}

// a b c a@last b@last
