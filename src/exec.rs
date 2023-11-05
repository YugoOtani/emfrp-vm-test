use crate::insn::*;
use std::{thread, time};

// TODO: stack size
// TODO: Value of Stack
#[derive(Debug, Clone)]
pub enum Value {
    Int(i32),
    Bool(bool),
}
#[derive(Debug)]
pub enum RuntimeErr {
    ValueLeft,
}

pub fn exec(insns: Vec<Insn2>) -> Result<Value, RuntimeErr> {
    let mut rip = (&insns[0]) as *const Insn2;
    let mut stack = vec![];
    unsafe {
        loop {
            match rip.as_ref().unwrap() {
                Insn2::Add => {
                    let v1 = stack.pop().unwrap();
                    let v2 = stack.pop().unwrap();
                    match (v1, v2) {
                        (Value::Int(i1), Value::Int(i2)) => stack.push(Value::Int(i1 + i2)),
                        _ => panic!(),
                    }
                }
                Insn2::Je(offset) => match stack.pop().unwrap() {
                    Value::Bool(b) => {
                        if b {
                            rip = rip.offset(*offset - 1);
                        }
                    }
                    _ => panic!(),
                },
                Insn2::J(offset) => rip = rip.offset(*offset - 1),
                Insn2::Mul => {
                    let v1 = stack.pop().unwrap();
                    let v2 = stack.pop().unwrap();
                    match (v1, v2) {
                        (Value::Int(i1), Value::Int(i2)) => stack.push(Value::Int(i1 + i2)),
                        _ => panic!(),
                    }
                }
                Insn2::Int(i) => stack.push(Value::Int(*i)),
                Insn2::Bool(b) => stack.push(Value::Bool(*b)),
                Insn2::Exit => {
                    let v = stack.pop().unwrap();
                    if stack.is_empty() {
                        return Ok(v);
                    } else {
                        return Err(RuntimeErr::ValueLeft);
                    }
                }
                Insn2::GetLocal(offset) => {
                    let v = stack[*offset].clone();
                    stack.push(v)
                }
                Insn2::SetLocal(offset) => {
                    let v = stack.pop().unwrap();
                    stack[*offset] = v;
                }
                Insn2::GetNode(_) => todo!(),
                Insn2::SetNode(_) => todo!(),
            }
            println!("{:?}", stack);
            thread::sleep(time::Duration::from_millis(100));
            rip = rip.offset(1)
        }
    }
}
