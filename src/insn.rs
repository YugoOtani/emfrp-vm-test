pub enum Insn {
    Add,
    Je(usize),
    J(usize),
    Mul,
    Int(i32),
    Bool(bool),
}
