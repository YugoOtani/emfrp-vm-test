#[derive(Debug, Clone)]

pub enum Insn {
    Nil,
    Add,
    Je(isize),
    J(isize),
    Mul,
    Int(i32),
    Bool(bool),
    GetLocal(StackOffset),
    SetLocal(StackOffset),
    Halt, // almost the same as Exit but not returning value
    AllocNode(NodeOffset, Vec<Insn>),
    ReallocNode,
    Return,
    Call(NArgs),
    UpdateNode(NodeOffset),
    GetNode(NodeOffset),
    SetNode(NodeOffset),
    GetLast(NodeOffset),
    SaveLast,
    Exit,
    Placeholder,
}
pub type NArgs = usize;
pub type NodeOffset = usize;
pub type StackOffset = usize;
