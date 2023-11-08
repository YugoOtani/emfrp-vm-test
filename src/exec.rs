use crate::{insn::*, DEBUG, MACHINE_FILE, MACHINE_MEMCHECK_MILLIS};
use std::fs::OpenOptions;
use std::{
    io::Write,
    sync::{mpsc::Sender, Arc, Mutex},
    thread, time,
};

// TODO: stack size
// TODO: Value of Stack
#[derive(Debug, Clone)]
pub enum Value {
    Int(i32),
    Bool(bool),
    Nil,
}
#[derive(Debug)]
pub enum RuntimeErr {
    ValueLeft,
}

pub fn exec(
    mem_is_updated: Arc<Mutex<bool>>,
    shared_mem: Arc<Mutex<Vec<Insn2>>>,
    out: Sender<String>,
) {
    let mut mem = vec![];
    let mut stack = vec![];
    let mut nodes = vec![];

    OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(MACHINE_FILE)
        .unwrap();

    // lock order : updated -> mem
    // unlock order : mem -> updated
    loop {
        {
            let mut updated = mem_is_updated.lock().unwrap();
            if *updated {
                swap(shared_mem.clone(), &mut mem);
                *updated = false;
                break;
            }
        }
        thread::sleep(time::Duration::from_millis(MACHINE_MEMCHECK_MILLIS));
    }

    let mut rip = &mem[0] as *const Insn2;
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
                        out.send(format!("[OK] {:?}", v)).unwrap();
                        return;
                    } else {
                        out.send(format!("[RuntimeError] value left on the stack"))
                            .unwrap();
                        return;
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
                Insn2::Print => {
                    let v = stack.pop().unwrap();
                    out.send(format!("{:?}", v)).unwrap();
                }
                Insn2::AllocNode => {
                    let v = stack.pop().unwrap();
                    nodes.push(v);
                }
                Insn2::DeleteNodes => nodes.clear(),

                Insn2::GetNode(i) => stack.push(nodes[*i].clone()),
                Insn2::SetNode(i) => {
                    let v = stack.pop().unwrap();
                    nodes[*i] = v;
                }
                Insn2::AllocNodes(i) => {
                    for _ in 0..*i {
                        nodes.push(Value::Nil);
                    }
                }
            }
            rip = rip.offset(1);
            let mut mtx = mem_is_updated.lock().unwrap();
            if *mtx {
                swap(shared_mem.clone(), &mut mem);
                *mtx = false;
                rip = &mem[0] as *const Insn2;
                stack = vec![]; //
            }
            let s = if DEBUG {
                format!("stack : {:?}   node : {:?}\n", stack, nodes)
            } else {
                format!("node : {:?}\n", nodes)
            };
            OpenOptions::new()
                .create(true)
                .append(true)
                .open(MACHINE_FILE)
                .unwrap()
                .write_all(s.as_bytes())
                .unwrap();
            thread::sleep(time::Duration::from_millis(100))
        }
    }
}

fn swap<T>(mtx: Arc<Mutex<T>>, t: &mut T) {
    std::mem::swap(mtx.lock().as_deref_mut().unwrap(), t)
}
