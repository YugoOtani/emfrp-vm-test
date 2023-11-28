use std::collections::{HashMap, VecDeque};

use crate::datastructure::List;
use crate::insn::*;
use crate::{ast::*, DEBUG, MAX_NUMBER_OF_NODE};
pub struct RuntimeNodeIndex(usize);
impl RuntimeNodeIndex {
    pub fn i(&self) -> usize {
        self.0
    }
}
#[derive(Debug, Default)]
struct NodeInfo {
    name: Id,
    is_new_name: bool,
    pointed: List<usize>, //index
}

enum SortResult {
    Success(Vec<usize>),
    CicularRef,
}
pub struct Compiler {
    codes: Vec<Insn>,
    node_info: Vec<NodeInfo>,
    symbol_table: Vec<Id>,
}

#[derive(Debug)]
pub enum CompileErr<'a> {
    IdNotFound(&'a Id),
    CircularRef,
    TooManyNodes,
}
pub enum CompiledCode {
    DefNode { init: Vec<Insn>, upd: Vec<Insn> },
    Exp(Vec<Insn>),
}
type CResult<'a, T> = Result<T, CompileErr<'a>>;

impl Compiler {
    pub fn compile<'a, 'b: 'a>(
        &'b mut self,
        prog: &'a Program,
    ) -> Result<CompiledCode, CompileErr<'a>> {
        assert!(self.codes.len() == 0);
        if let Program::Exp(e) = prog {
            e.emit_code(self)?;
            let mut e = self.insn_popall();
            e.push(Insn::Exit);
            return Ok(CompiledCode::Exp(e));
        }

        self.register_new_node(prog)?;
        if self.node_info.len() > MAX_NUMBER_OF_NODE {
            return Err(CompileErr::TooManyNodes);
        }
        self.emit_alloc_node(prog)?;
        self.push_insn(Insn::Halt);
        let init = self.insn_popall();

        let sorted_nodes = match self.topological_sort() {
            SortResult::Success(nd) => nd,
            SortResult::CicularRef => return Err(CompileErr::CircularRef),
        };
        if DEBUG {
            print!("[dependency]");
            for i in &sorted_nodes {
                print!(" -> {}", self.node_info[*i].name.s);
            }
            println!("")
        }

        let mut upd = Vec::with_capacity(2 * sorted_nodes.len() + 2);

        for id in sorted_nodes {
            upd.push(Insn::UpdateNode(id));
            upd.push(Insn::SetNode(id));
        }
        upd.push(Insn::Halt);
        Ok(CompiledCode::DefNode { init, upd })
    }

    fn register_new_node<'a>(&mut self, prog: &'a Program) -> CResult<'a, ()> {
        match prog {
            Program::Defs(defs) => {
                for def in defs {
                    match self.register_new_node_one(def) {
                        Ok(()) => continue,
                        Err(e) => return Err(e),
                    }
                }
                Ok(())
            }
            Program::Def(def) => self.register_new_node_one(def),
            Program::Exp(_) => Ok(()),
        }
    }

    fn unregister_node(&mut self, _dep: List<usize>) {
        // do nothing
    }
    fn register_new_node_one<'a>(&mut self, def: &'a Def) -> CResult<'a, ()> {
        match def {
            Def::Node { name, init: _, val } => match self.node_offset(name) {
                Some(i) => {
                    // node of the same name exist
                    let mut pointed = List::new();
                    val.to_dependency(&mut pointed, self);
                    std::mem::swap(&mut pointed, &mut self.node_info[i].pointed);
                    self.unregister_node(pointed);
                    Ok(())
                }
                None => {
                    let mut pointed = List::new();
                    val.to_dependency(&mut pointed, self);
                    self.node_info.push(NodeInfo {
                        name: name.clone(),
                        is_new_name: true,
                        pointed,
                    });
                    Ok(())
                }
            },
            _ => Ok(()),
        }
    }
    fn push_insn(&mut self, insn: Insn) {
        self.codes.push(insn)
    }
    fn compile_exp<'a>(&mut self, exp: &'a Exp) -> CResult<'a, Vec<Insn>> {
        let mut ret = vec![];
        std::mem::swap(&mut self.codes, &mut ret);
        exp.emit_code(self)?;
        std::mem::swap(&mut self.codes, &mut ret);
        Ok(ret)
    }

    pub fn new() -> Self {
        Compiler {
            codes: vec![],
            node_info: vec![],
            symbol_table: vec![],
        }
    }

    fn insn_popall(&mut self) -> Vec<Insn> {
        std::mem::take(self.codes.as_mut())
    }

    fn topological_sort(&self) -> SortResult {
        let mut q = VecDeque::new();
        let mut cnt = HashMap::new();
        let mut ret = vec![];
        for (
            i,
            NodeInfo {
                name,
                pointed,
                is_new_name: _,
            },
        ) in self.node_info.iter().enumerate()
        {
            if DEBUG {
                assert_eq!(self.node_offset(name), Some(i));
            }

            if pointed.is_empty() {
                q.push_back(i);
                cnt.insert(i, 0);
            } else {
                cnt.insert(i, pointed.len());
            }
        }

        while let Some(nd) = q.pop_front() {
            // ndがさすノードのカウントを減らす
            for (
                i,
                NodeInfo {
                    name: _,
                    pointed,
                    is_new_name: _,
                },
            ) in self.node_info.iter().enumerate()
            {
                if pointed.contains(&nd) {
                    *cnt.get_mut(&i).unwrap() -= 1;
                    if *cnt.get(&i).unwrap() == 0 {
                        q.push_back(i)
                    }
                }
            }
            ret.push(nd);
        }
        if ret.len() == cnt.len() {
            SortResult::Success(ret)
        } else {
            SortResult::CicularRef
        }
    }

    fn node_offset(&self, name: &Id) -> Option<usize> {
        for (i, e) in self.node_info.iter().enumerate() {
            if name == &e.name {
                return Some(i);
            }
        }
        None
    } /*
      fn contain_node(&self, name: &Id) -> bool {
          matches!(self.node_offset(name), Some(_))
      }*/
    fn emit_alloc_node<'a>(&mut self, prog: &'a Program) -> CResult<'a, ()> {
        match prog {
            Program::Defs(defs) => {
                for def in defs {
                    self.emit_alloc_node_one(def)?;
                }
                Ok(())
            }
            Program::Def(def) => self.emit_alloc_node_one(def),
            Program::Exp(_) => Ok(()),
        }
    }
    fn emit_alloc_node_one<'a>(&mut self, def: &'a Def) -> CResult<'a, ()> {
        match def {
            Def::Node { name, init, val } => {
                match init {
                    Some(e) => e.emit_code(self)?,
                    None => self.push_insn(Insn::Nil),
                };
                let mut insn = self.compile_exp(val)?;
                insn.push(Insn::Return);
                let offset = self.node_offset(name).unwrap();
                let insn = if self.node_info[offset].is_new_name {
                    Insn::AllocNodeNew(insn)
                } else {
                    Insn::AllocNode(offset, insn)
                };
                self.push_insn(insn);
                Ok(())
            }
            Def::Data { .. } => Ok(()),
            Def::Func { .. } => Ok(()),
        }
    }
}

impl Exp {
    pub fn emit_code<'a>(&'a self, c: &mut Compiler) -> CResult<'a, ()> {
        match self {
            Exp::If { cond, then, els } => {
                cond.emit_code(c)?;
                let offset = c.codes.len();
                c.push_insn(Insn::Placeholder);
                then.emit_code(c)?;
                let offset2 = c.codes.len();
                c.push_insn(Insn::Placeholder);
                els.emit_code(c)?;
                let offset3 = c.codes.len();
                // cond JE then J els
                c.codes[offset as usize] = Insn::je(bytecode_len(offset, offset2, &c.codes) as i32);
                c.codes[offset2 as usize] =
                    Insn::j(bytecode_len(offset2, offset3, &c.codes) as i32);
                Ok(())
            }
            Exp::Add(e, t) => {
                e.emit_code(c)?;
                t.emit_code(c)?;
                c.push_insn(Insn::Add);
                Ok(())
            }
            Exp::Term(t) => t.emit_code(c),
        }
    }
    fn to_dependency(&self, lst: &mut List<usize>, cmp: &Compiler) {
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
    fn emit_code<'a>(&'a self, c: &mut Compiler) -> CResult<'a, ()> {
        match self {
            Term::Mul(t1, t2) => {
                t1.emit_code(c)?;
                t2.emit_code(c)?;
                c.push_insn(Insn::Mul);
            }
            Term::Int(i) => c.push_insn(Insn::Int(*i)),
            Term::FnCall(_, _) => todo!(),
            Term::Bool(b) => c.push_insn(Insn::Bool(*b)),
            Term::Last(id) => c.push_insn(Insn::GetLast(c.node_offset(id).unwrap())),
            Term::Id(id) => {
                for (i, id2) in c.symbol_table.iter().enumerate() {
                    if id == id2 {
                        c.push_insn(Insn::GetLocal(i));
                        return Ok(());
                    }
                }
                if let Some(i) = c.node_offset(id) {
                    c.push_insn(Insn::GetNode(i))
                } else {
                    todo!()
                }
            }
        }
        Ok(())
    }
    fn to_dependency(&self, lst: &mut List<usize>, c: &Compiler) {
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
                if let Some(u) = c.node_offset(id) {
                    lst.push(u)
                }
            }
            Term::Last(_) => return,
        }
    }
}
