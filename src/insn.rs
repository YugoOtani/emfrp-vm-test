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
