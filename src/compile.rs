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
    sorted_nodes: Vec<Id>,
    upd_formula: HashMap<Id, Exp>,
}
#[derive(Debug)]
pub enum CompileErr {
    IdNotFound(String),
}
pub type CResult = Result<Vec<Insn1>, CompileErr>;
type EmitResult = Result<(), CompileErr>;

impl Compiler {
    pub fn compile(&mut self, prog: &Program) -> CResult {
        assert!(self.codes.len() == 0);
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
            Program::Exp(e) => {
                e.emit_code(self)?;
                return Ok(self.insn_popall());
            }
        };
        let sorted_nodes = topological_sort(&self.pointed);
        self.sorted_nodes = sorted_nodes;
        // TODO:これだとExp用に追加の記憶容量が必要
        // stack上のノードの順番は何でもいいのでそれでなんとかできないか => Expの再コンパイルが不要なように
        // .
        let mut exps = vec![];
        if DEBUG {
            println!("[sorted nodes]")
        }
        for id in &self.sorted_nodes {
            if DEBUG {
                print!(" -> {}", id.s);
            }

            let exp = self.upd_formula.get(id).unwrap();
            exps.push(exp.clone());
        }
        if DEBUG {
            println!("");
        }

        for exp in exps {
            exp.emit_code(self)?;
        }
        Ok(self.insn_popall())
    }
    /*fn add_sym(&mut self, id: Id) {
        self.symbol_table.push(id)
    }*/
    fn add_node(&mut self, def: &Def) {
        match def {
            Def::Node { name, init: _, val } => {
                self.nodes.insert(name.clone());
                self.upd_formula.insert(name.clone(), val.clone());
                self.pointed.insert(name.clone(), HashSet::new());
            }
            _ => return,
        }
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
    // find_sym must be called after node is sorted
    fn find_sym(&self, id: &Id) -> Insn1 {
        let tbl_n = self.symbol_table.len();
        for i in (0..tbl_n).rev() {
            if id == &self.symbol_table[i] {
                return Insn1::GetLocal(i);
            }
        }
        let nd_n = self.sorted_nodes.len();
        for i in 0..nd_n {
            if id == &self.sorted_nodes[i] {
                return Insn1::GetNode(id.clone());
            }
        }
        todo!()
    }
    pub fn new() -> Self {
        Compiler {
            codes: vec![],
            nodes: HashSet::new(),
            symbol_table: vec![],
            pointed: HashMap::new(),
            sorted_nodes: vec![],
            upd_formula: HashMap::new(),
        }
    }
    pub fn insn_popall(&mut self) -> Vec<Insn1> {
        let mut new = vec![];
        std::mem::swap(self.codes.as_mut(), &mut new);
        new
    }
    fn push_insn(&mut self, insn: Insn1) {
        self.codes.push(insn)
    }
    pub fn codes_move(self) -> Vec<Insn1> {
        self.codes
    }
    pub fn codes(&self) -> &Vec<Insn1> {
        &self.codes
    }
    pub fn sorted_nodes(&self) -> &Vec<Id> {
        &self.sorted_nodes
    }
    pub fn to_insn2(&self, insns: Vec<Insn1>) -> Vec<Insn2> {
        insns
            .into_iter()
            .map(|insn| match insn {
                Insn1::Add => Insn2::Add,
                Insn1::Je(i) => Insn2::Je(i),
                Insn1::J(i) => Insn2::J(i),
                Insn1::Mul => Insn2::Mul,
                Insn1::Int(i) => Insn2::Int(i),
                Insn1::Bool(b) => Insn2::Bool(b),
                Insn1::GetLocal(i) => Insn2::GetLocal(i),
                Insn1::SetLocal(i) => Insn2::SetLocal(i),
                Insn1::GetNode(id) => Insn2::GetLocal(self.find_nd(&id).unwrap()),
                Insn1::SetNode(id) => Insn2::GetLocal(self.find_nd(&id).unwrap()),
                Insn1::Exit => Insn2::Exit,
            })
            .collect()
    }
    fn find_nd(&self, id: &Id) -> Option<usize> {
        for (i, id2) in self.sorted_nodes.iter().enumerate() {
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
