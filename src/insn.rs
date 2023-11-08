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
    Print,
    AllocNode,
    AllocNodes(usize),
    DeleteNodes,
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
    Print,
    AllocNode,
    AllocNodes(usize),
    GetNode(Id),
    SetNode(Id),
    Exit,
    Placeholder,
}
