#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Insn {
    None,
    Nil,
    Int(i32),
    Bool(bool),
    Add,
    Mul,
    Je8(i8),
    Je32(i32),
    J8(i8),
    J32(i32),

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

impl Insn {
    pub fn push_byte_code(self, ret: &mut Vec<u8>) {
        let op_code = match self {
            Insn::None => 0,
            Insn::Nil => 1,
            Insn::Int(_) => 2,
            Insn::Bool(_) => 3,
            Insn::Add => 4,
            Insn::Mul => 5,
            Insn::Je8(_) => 6,
            Insn::Je32(_) => 7,
            Insn::J8(_) => 8,
            Insn::J32(_) => 9,
            Insn::GetLocal(_) => 10,
            Insn::SetLocal(_) => 11,
            Insn::AllocNode(_, _) => 12,
            Insn::AllocNodeNew(_) => 13,
            Insn::UpdateNode(_) => 14,
            Insn::GetNode(_) => 15,
            Insn::SetNode(_) => 16,
            Insn::GetLast(_) => 17,
            Insn::SaveLast => 18,
            Insn::AllocFunc(_, _) => 19,
            Insn::AllocFuncNew(_) => 20,
            Insn::AllocData(_, _) => 21,
            Insn::AllocDataNew(_) => 22,
            Insn::Return => 23,
            Insn::Call(_) => 24,
            Insn::Exit => 25,
            Insn::Halt => 26,
            Insn::Placeholder => panic!(),
        };
        ret.push(op_code);
        match self {
            // no immediate value
            Insn::None
            | Insn::Nil
            | Insn::Add
            | Insn::Mul
            | Insn::Halt
            | Insn::Return
            | Insn::SaveLast
            | Insn::Exit
            | Insn::Placeholder => return,
            // i8
            Insn::Je8(i) | Insn::J8(i) => ret.push(i.to_le_bytes()[0]),
            Insn::Call(i)
            | Insn::UpdateNode(i)
            | Insn::GetNode(i)
            | Insn::SetNode(i)
            | Insn::GetLast(i)
            | Insn::GetLocal(i)
            | Insn::SetLocal(i) => ret.push(i.to_le_bytes()[0]),

            //i32
            Insn::Int(i) | Insn::Je32(i) | Insn::J32(i) => push_int_le(i, ret),

            Insn::Bool(b) => ret.push(if b { 1 } else { 0 }),
            Insn::AllocDataNew(insns) | Insn::AllocFuncNew(insns) | Insn::AllocNodeNew(insns) => {
                let offset = ret.len();
                for _ in 0..4 {
                    ret.push(0);
                }
                for insn in insns {
                    insn.push_byte_code(ret);
                }
                let code_len = ret.len() - (offset + 4);
                for (i, v) in code_len.to_le_bytes().iter().enumerate() {
                    ret[offset + i] = *v;
                }
            }
            Insn::AllocNode(i, insns) | Insn::AllocFunc(i, insns) | Insn::AllocData(i, insns) => {
                ret.push(i.to_le_bytes()[0]);
                let offset = ret.len();
                for _ in 0..4 {
                    ret.push(0);
                }
                for insn in insns {
                    insn.push_byte_code(ret);
                }
                let code_len = ret.len() - (offset + 4);
                for (i, v) in code_len.to_le_bytes().iter().enumerate() {
                    ret[offset + i] = *v;
                }
            }
        }
    }
}
impl Insn {
    pub fn j(i: i32) -> Self {
        if i8::MIN as i32 <= i && i <= i8::MAX as i32 {
            Insn::J8(i as i8)
        } else {
            Insn::J32(i as i32)
        }
    }
    pub fn je(i: i32) -> Self {
        if i8::MIN as i32 <= i && i <= i8::MAX as i32 {
            Insn::Je8(i as i8)
        } else {
            Insn::Je32(i as i32)
        }
    }
}
fn push_int_le(i: i32, ret: &mut Vec<u8>) {
    for b in i.to_le_bytes() {
        ret.push(b)
    }
}
pub fn bytecode_len(st: usize, ed: usize, v: &Vec<Insn>) -> usize {
    let mut ret = 0;
    for i in st..ed {
        ret += match &v[i] {
            // no immediate value
            Insn::None
            | Insn::Nil
            | Insn::Add
            | Insn::Mul
            | Insn::Halt
            | Insn::Return
            | Insn::SaveLast
            | Insn::Exit
            | Insn::Placeholder => 1,
            // i8
            Insn::Je8(_) | Insn::J8(_) => 2,
            Insn::Call(_)
            | Insn::UpdateNode(_)
            | Insn::GetNode(_)
            | Insn::SetNode(_)
            | Insn::GetLast(_)
            | Insn::GetLocal(_)
            | Insn::SetLocal(_) => 2,

            //i32
            Insn::Int(_) | Insn::Je32(_) | Insn::J32(_) => 5,

            Insn::Bool(_) => 2,
            Insn::AllocDataNew(insns) | Insn::AllocFuncNew(insns) | Insn::AllocNodeNew(insns) => {
                1 + bytecode_len(0, insns.len(), &insns)
            }
            Insn::AllocNode(_, insns) | Insn::AllocFunc(_, insns) | Insn::AllocData(_, insns) => {
                2 + bytecode_len(0, insns.len(), &insns)
            }
        }
    }
    ret
}
#[test]
fn insn_8_32() {
    assert_eq!(Insn::j(1000), Insn::J32(1000));
    assert_eq!(Insn::j(100), Insn::J8(100));
    assert_eq!(12, 12usize.to_le_bytes()[0]);
}
