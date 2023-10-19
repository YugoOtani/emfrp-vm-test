pub enum Insn {
    Add,
    Je(isize),
    J(isize),
    Mul,
    Int(i32),
    Bool(bool),
    Exit,
}
