#[derive(Debug, Clone)]

pub enum Insn {
    None,
    Nil,
    Add,
    Je(i32),
    J(i32),
    Mul,
    Int(i32),
    Bool(bool),
    GetLocal(StackOffset),
    SetLocal(StackOffset),
    Halt, // almost the same as Exit but not returning value
    AllocNode(NodeOffset, Vec<Insn>),
    AllocNodeNew(Vec<Insn>),
    AllocFunc(FuncOffset, Vec<Insn>),
    AllocFuncNew(Vec<Insn>),
    AllocData(DataOffset, Vec<Insn>),
    AllocDataNew(Vec<Insn>),
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
pub type FuncOffset = usize;
pub type DataOffset = usize;
