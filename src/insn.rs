use crate::ast::Id;

#[derive(Debug)]
pub enum Insn2 {
    Add,
    Je(isize),
    J(isize),
    Mul,
    Int(i32),
    Bool(bool),
    GetLocal(usize),
    SetLocal(usize),
    GetNode(usize),
    SetNode(usize),
    Exit,
}

#[derive(Debug)]
pub enum Insn1 {
    Add,
    Je(isize),
    J(isize),
    Mul,
    Int(i32),
    Bool(bool),
    GetLocal(usize),
    SetLocal(usize),
    GetNode(Id),
    SetNode(Id),
    Exit,
}
pub fn to_insn2(insns: Vec<Insn1>, sorted_nds: &Vec<Id>) -> Vec<Insn2> {
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
            Insn1::GetNode(id) => Insn2::GetLocal(find_nd(&id, sorted_nds).unwrap()),
            Insn1::SetNode(id) => Insn2::GetLocal(find_nd(&id, sorted_nds).unwrap()),
            Insn1::Exit => Insn2::Exit,
        })
        .collect()
}
fn find_nd(id: &Id, ids: &Vec<Id>) -> Option<usize> {
    for (i, id2) in ids.iter().enumerate() {
        if id == id2 {
            return Some(i);
        }
    }
    None
}
