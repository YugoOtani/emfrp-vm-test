use std::{
    sync::{
        mpsc::{self, Receiver},
        Arc, Mutex,
    },
    thread::{self},
    time::Duration,
};

use crate::{
    insn::*, DEBUG, INITIAL_NODE_SIZE, MACHINE_FILE, MACHINE_MEMCHECK_MILLIS, UPD_FREQUENCY_MS,
};
use std::fmt::Debug;
use std::fs::OpenOptions;
use std::{io::Write, sync::mpsc::Sender, time};
// TODO: stack size
// TODO: Value of Stack
#[derive(Debug, Clone)]
pub enum Value {
    Int(i32),
    Bool(bool),
    Nil,
    Insn(*const Insn),
    Usize(usize),
}
#[derive(Debug)]
pub enum RuntimeErr {
    ValueLeft,
}

#[derive(Debug)]
pub struct Msg {
    res_receiver: Receiver<String>,
    code: Arc<Mutex<Option<Code>>>,
    code_is_updated: Arc<Mutex<bool>>,
}
unsafe impl Send for Msg {}
unsafe impl Sync for Msg {}
pub enum Code {
    DefNode { init: Vec<Insn>, upd: Vec<Insn> },
    Exp(Vec<Insn>),
}

struct Machine {
    stack: Vec<Value>,
    nodes: Vec<Value>,
    lasts: Vec<Value>,
    upd: Vec<Vec<Insn>>,
    out: Sender<String>,
    current_code: Vec<Insn>,
}
impl Debug for Machine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut nd_upd = String::new();
        assert_eq!(self.upd.len(), self.nodes.len());
        for i in 0..self.upd.len() {
            nd_upd.push_str(&format!(
                "    {:?} {:?} {:?}\n",
                self.lasts[i], self.nodes[i], self.upd[i]
            ))
        }
        write!(
            f,
            "[Machine State]\n  stack : {:?}\n  node  : \n{}  insn  : {:?}",
            self.stack, nd_upd, self.current_code
        )
    }
}
impl Debug for Code {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut ret = String::from("[Bytecode]\n");
        match self {
            Code::DefNode { init, upd } => {
                ret.push_str(" init:\n");
                for insn in init {
                    ret.push_str(&format!("  {:?}\n", insn));
                }
                ret.push_str(" upd:\n");
                for insn in upd {
                    ret.push_str(&format!("  {:?}\n", insn));
                }
            }
            Code::Exp(e) => {
                for insn in e {
                    ret.push_str(&format!(" {:?}\n", insn));
                }
            }
        }

        write!(f, "{}", ret)
    }
}

/*
[machine, timer] <-(program, res_sender)-> [pc]
 */

impl Msg {
    pub fn get_msg(&self) -> String {
        self.res_receiver.recv().unwrap()
    }
    // If send_code returns None, upd = true
    // If upd = true, try_receive_code returns Some(code) and upd turns to false
    // Since upd doesn't turn to false otherwise, when send_code returns None,
    // try_receive_code returns Some(code) sometime.
    // When try_receive_code returns Some(code),
    // new_code is called and send message into channel(Machine::out)
    // Therefore, caller of send_code can use self.get_msg (blocks until msg comes)
    pub fn send_code(&self, code: Code) -> Option<Code> {
        let mut upd = self.code_is_updated.lock().unwrap();
        if *upd {
            // previous update is not taken by machine
            Some(code)
        } else {
            *upd = true;
            *self.code.lock().unwrap() = Some(code);
            // successfully taken
            None
        }
    }
    fn try_receive_code(
        code: &Arc<Mutex<Option<Code>>>,
        code_is_updated: &Arc<Mutex<bool>>,
    ) -> Option<Code> {
        let mut upd = code_is_updated.lock().unwrap();
        if *upd {
            let mut ret = None;
            mtx_swap(code, &mut ret);
            *upd = false;
            // when code_is_updated = true, code must points to Some(code)
            let ret = ret.unwrap();
            Some(ret)
        } else {
            None
        }
    }
}
pub fn machine_run() -> Msg {
    let (res_sender, res_receiver) = mpsc::channel();
    let code = Arc::new(Mutex::new(None));
    let code_is_updated = Arc::new(Mutex::new(false));
    let timer = timer();
    let code_clone = code.clone();
    let upd_clone = code_is_updated.clone();
    thread::spawn(move || {
        let code_mtx = code;
        let mut machine = Machine::new(res_sender);
        let code = loop {
            if let Some(code) = Msg::try_receive_code(&code_mtx, &code_is_updated) {
                break code;
            }
            thread::sleep(Duration::from_millis(MACHINE_MEMCHECK_MILLIS));
        };
        machine.new_code(code);
        loop {
            if check_if_true(&timer) {
                machine.exec_upd();
            }

            if let Some(newcode) = Msg::try_receive_code(&code_mtx, &code_is_updated) {
                machine.new_code(newcode);
            }
        }
    });
    Msg {
        res_receiver,
        code: code_clone,
        code_is_updated: upd_clone,
    }
}

fn timer() -> Arc<Mutex<bool>> {
    let timer = Arc::new(Mutex::new(false));
    let clone = timer.clone();
    thread::spawn(move || loop {
        thread::sleep(Duration::from_millis(UPD_FREQUENCY_MS));
        *timer.lock().unwrap() = true;
    });
    clone
}
fn check_if_true(mtx: &Arc<Mutex<bool>>) -> bool {
    let mut guard = mtx.lock().unwrap();
    if *guard {
        *guard = false;
        true
    } else {
        false
    }
}

impl Machine {
    fn new(res_sender: Sender<String>) -> Self {
        OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(MACHINE_FILE)
            .unwrap();
        let mut upd = Vec::with_capacity(INITIAL_NODE_SIZE);
        let mut nodes = Vec::with_capacity(INITIAL_NODE_SIZE);
        let mut lasts = Vec::with_capacity(INITIAL_NODE_SIZE);
        for _ in 0..INITIAL_NODE_SIZE {
            upd.push(vec![]);
            nodes.push(Value::Nil);
            lasts.push(Value::Nil);
        }
        Self {
            stack: vec![],
            nodes,
            upd,
            lasts,
            out: res_sender,
            current_code: vec![Insn::Halt],
        }
    }
    fn exec_upd(&mut self) {
        let code = &self.current_code[0] as *const Insn;
        self.exec_insn(code)
    }
    fn exec_insn(&mut self, insn: *const Insn) {
        let mut rip = insn;
        let mut rbp = 0;
        unsafe {
            loop {
                match rip.as_ref().unwrap() {
                    Insn::Nil => self.stack.push(Value::Nil),
                    Insn::Add => {
                        let v1 = self.stack.pop().unwrap();
                        let v2 = self.stack.pop().unwrap();
                        match (v1, v2) {
                            (Value::Int(i1), Value::Int(i2)) => {
                                self.stack.push(Value::Int(i1 + i2))
                            }
                            _ => panic!(),
                        }
                    }
                    Insn::Je(offset) => match self.stack.pop().unwrap() {
                        Value::Bool(b) => {
                            if b {
                                rip = rip.offset(*offset - 1);
                            }
                        }
                        _ => panic!(),
                    },
                    Insn::J(offset) => rip = rip.offset(*offset - 1),
                    Insn::Mul => {
                        let v1 = self.stack.pop().unwrap();
                        let v2 = self.stack.pop().unwrap();
                        match (v1, v2) {
                            (Value::Int(i1), Value::Int(i2)) => {
                                self.stack.push(Value::Int(i1 + i2))
                            }
                            _ => panic!(),
                        }
                    }
                    Insn::Int(i) => self.stack.push(Value::Int(*i)),
                    Insn::Bool(b) => self.stack.push(Value::Bool(*b)),
                    Insn::Exit => {
                        let v = self.stack.pop().unwrap();
                        if self.stack.is_empty() {
                            self.out.send(format!("[OK] {:?}", v)).unwrap();
                            return;
                        } else {
                            self.out
                                .send(format!("[RuntimeError] value left on the stack"))
                                .unwrap();
                            return;
                        }
                    }
                    Insn::GetLocal(offset) => {
                        let v = self.stack[*offset + rbp].clone();
                        self.stack.push(v)
                    }
                    Insn::SetLocal(offset) => {
                        let v = self.stack.pop().unwrap();
                        self.stack[*offset + rbp] = v;
                    }
                    Insn::AllocNode(u, insn) => {
                        let v = self.stack.pop().unwrap();
                        self.nodes[*u] = v;
                        self.upd[*u] = insn.clone();
                    }
                    Insn::GetNode(i) => self.stack.push(self.nodes[*i].clone()),
                    Insn::SetNode(i) => {
                        let v = self.stack.pop().unwrap();
                        self.nodes[*i] = v;
                    }
                    Insn::Placeholder => panic!(),
                    Insn::Halt => return,
                    Insn::UpdateNode(i) => {
                        self.stack.push(Value::Usize(rbp));
                        self.stack.push(Value::Insn(rip));
                        rip = &self.upd[*i][0] as *const Insn;
                        rip = rip.offset(-1);
                    }
                    Insn::ReallocNode => {
                        let n = self.nodes.len();
                        for _ in 0..n {
                            self.nodes.push(Value::Nil);
                            self.upd.push(vec![]);
                        }
                    }
                    Insn::Return => {
                        let v = self.stack.pop().unwrap();
                        let old_rip = self.stack.pop().unwrap();
                        let old_rbp = self.stack.pop().unwrap();
                        self.stack.push(v);
                        rbp = if let Value::Usize(u) = old_rbp {
                            u
                        } else {
                            panic!()
                        };
                        rip = if let Value::Insn(u) = old_rip {
                            u
                        } else {
                            panic!()
                        };
                    }
                    Insn::Call(_) => todo!(),
                    Insn::SaveLast => std::mem::swap(&mut self.lasts, &mut self.nodes),
                    Insn::GetLast(i) => self.stack.push(self.lasts[*i].clone()),
                }
                rip = rip.offset(1);

                let s = if DEBUG {
                    format!("node : {:?}   stack : {:?}\n", self.lasts, self.stack)
                } else {
                    format!("node : {:?}\n", self.lasts)
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
    fn send_msg(&self, msg: String) {
        self.out.send(msg).unwrap()
    }
    // new_code must return self.out something because
    // when main thread send code to machine,
    // main thread expects that machine returns msg through channel
    fn new_code(&mut self, code: Code) {
        match code {
            Code::DefNode { init, upd } => {
                self.exec_insn(&init[0] as *const Insn); // codes for defining node is contained in init
                self.current_code = upd;
                self.send_msg("Node was defined successfully".to_string())
            }
            Code::Exp(exp) => {
                self.exec_insn(&exp[0] as *const Insn);
                // Exit returns value into channel, so doesn't need to send message
            }
        }
    }
}

fn mtx_swap<T>(mtx: &Arc<Mutex<T>>, t: &mut T) {
    std::mem::swap(mtx.lock().as_deref_mut().unwrap(), t)
}
