use crate::insn::*;

// TODO:stack size
#[derive(Debug, Clone)]
pub enum Value {
    Int(i32),
    Bool(bool),
}
#[derive(Debug)]
pub enum RuntimeErr {
    ValueLeft,
}

pub fn exec(mut insns: Vec<Insn>) -> Result<Value, RuntimeErr> {
    let mut rip = (&insns[0]) as *const Insn;
    let mut stack = vec![];
    insns.push(Insn::Exit);
    unsafe {
        loop {
            match rip.as_ref().unwrap() {
                Insn::Add => {
                    let v1 = stack.pop().unwrap();
                    let v2 = stack.pop().unwrap();
                    match (v1, v2) {
                        (Value::Int(i1), Value::Int(i2)) => stack.push(Value::Int(i1 + i2)),
                        _ => panic!(),
                    }
                }
                Insn::Je(offset) => match stack.pop().unwrap() {
                    Value::Bool(b) => {
                        if b {
                            rip = rip.offset(*offset);
                        }
                    }
                    _ => panic!(),
                },
                Insn::J(offset) => rip = rip.offset(*offset),
                Insn::Mul => {
                    let v1 = stack.pop().unwrap();
                    let v2 = stack.pop().unwrap();
                    match (v1, v2) {
                        (Value::Int(i1), Value::Int(i2)) => stack.push(Value::Int(i1 + i2)),
                        _ => panic!(),
                    }
                }
                Insn::Int(i) => stack.push(Value::Int(*i)),
                Insn::Bool(b) => stack.push(Value::Bool(*b)),
                Insn::Exit => {
                    let v = stack.pop().unwrap();
                    if stack.is_empty() {
                        return Ok(v);
                    } else {
                        return Err(RuntimeErr::ValueLeft);
                    }
                }
            }
            rip = rip.offset(1)
        }
    }
}
