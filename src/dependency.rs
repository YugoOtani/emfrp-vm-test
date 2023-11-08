use std::collections::{HashMap, HashSet, VecDeque};

use crate::ast::*;

pub fn topological_sort(st: &HashMap<Id, HashSet<Id>>) -> Vec<&Id> {
    let mut q = VecDeque::new();
    let mut cnt = HashMap::new();
    let mut ret = vec![];
    for (after, befores) in st {
        if befores.len() == 0 {
            q.push_back(after)
        } else {
            cnt.insert(after, befores.len());
        }
    }
    while let Some(nd) = q.pop_front() {
        for (after, befores) in st {
            if befores.contains(&nd) {
                *cnt.get_mut(after).unwrap() -= 1;
                if *cnt.get(after).unwrap() == 0 {
                    q.push_back(after)
                }
            }
        }
        ret.push(nd);
    }
    ret
}

impl Exp {
    pub fn to_dependency(self, dep: &mut HashSet<Id>, nodes: &HashSet<Id>) {
        match self {
            Exp::If { cond, then, els } => {
                cond.to_dependency(dep, nodes);
                then.to_dependency(dep, nodes);
                els.to_dependency(dep, nodes);
            }
            Exp::Add(e, t) => {
                e.to_dependency(dep, nodes);
                t.to_dependency(dep, nodes);
            }
            Exp::Term(t) => t.to_dependency(dep, nodes),
        }
    }
}
impl Term {
    fn to_dependency(self, dep: &mut HashSet<Id>, nodes: &HashSet<Id>) {
        match self {
            Term::Mul(t1, t2) => {
                t1.to_dependency(dep, nodes);
                t2.to_dependency(dep, nodes);
            }
            Term::Int(_) => return,
            Term::FnCall(_, args) => {
                for arg in args {
                    arg.to_dependency(dep, nodes);
                }
            }
            Term::Bool(_) => return,

            // node left_variable = (idの式)
            Term::Id(id) => {
                if nodes.contains(&id) {
                    dep.insert(id);
                }
            }
            Term::Last(_) => return,
        }
    }
}
