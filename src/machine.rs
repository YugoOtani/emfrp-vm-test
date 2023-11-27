use std::{
    sync::{
        mpsc::{self, Receiver},
        Arc, Mutex,
    },
    thread::{self},
    time::{Duration, Instant},
};

use crate::{
    compile::RuntimeNodeIndex, insn::*, DEBUG, MACHINE_FILE, MAX_NUMBER_OF_NODE, UPD_FREQUENCY_MS,
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
unsafe impl Send for Value {}
#[derive(Debug)]
pub enum RuntimeErr {
    StackOverflow,
}

#[derive(Debug)]
pub struct Msg {
    code: Arc<Mutex<Option<Code>>>,
    code_is_updated: Arc<Mutex<bool>>,
}
unsafe impl Send for Msg {}
unsafe impl Sync for Msg {}
pub enum Code {
    DefNode { init: Vec<Insn>, upd: Vec<Insn> },
    Exp(Vec<Insn>),
}
#[derive(Debug)]
enum InputAction {
    Device(fn() -> Value),
    Insn(Vec<Insn>),
    None,
}
type OutputAction = Option<fn(&Value)>;

pub struct Machine {
    stack: Vec<Value>,
    node_v: Vec<Value>,
    node_v_last: Vec<Value>,
    node_input_action: Vec<InputAction>,
    node_output_action: Vec<OutputAction>,
    out: Sender<String>,
    update: Vec<Insn>,
}
impl Debug for Machine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut nd_upd = String::new();
        for i in 0..self.node_v.len() {
            nd_upd.push_str(&format!(
                "    {:?} {:?} {:?}\n",
                self.node_v[i], self.node_v_last[i], self.node_input_action[i]
            ))
        }
        write!(
            f,
            "[Machine State]\n  stack : {:?}\n  node  : \n{}  insn  : {:?}",
            self.stack, nd_upd, self.update
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

impl Msg {
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
pub fn write_file_append(s: String) {
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(MACHINE_FILE)
        .unwrap()
        .write_all(s.as_bytes())
        .unwrap();
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
    pub fn add_input_node(&mut self, ind: RuntimeNodeIndex, f: fn() -> Value) {
        let i = ind.i();
        self.node_input_action[i] = InputAction::Device(f);
        self.update.push(Insn::UpdateNode(i));
        self.update.push(Insn::SetNode(i));
    }
    pub fn add_output_node(&mut self, ind: RuntimeNodeIndex, f: fn(&Value)) {
        let i = ind.i();
        self.node_output_action[i] = Some(f);
        self.update.push(Insn::UpdateNode(i));
        self.update.push(Insn::SetNode(i));
    }
    fn make_empty_node(&mut self) {
        self.node_v.push(Value::Nil);
        self.node_v_last.push(Value::Nil);
        self.node_input_action.push(InputAction::None);
        self.node_output_action.push(None);
    }
    pub fn new() -> (Self, Receiver<String>) {
        OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(MACHINE_FILE)
            .unwrap();
        let (sender, receiver) = mpsc::channel();
        let node_v = Vec::with_capacity(MAX_NUMBER_OF_NODE);
        let node_v_last = Vec::with_capacity(MAX_NUMBER_OF_NODE);
        let node_input_action = Vec::with_capacity(MAX_NUMBER_OF_NODE);
        let node_output_action = Vec::with_capacity(MAX_NUMBER_OF_NODE);

        let mut machine = Self {
            stack: vec![],
            node_v,
            node_v_last,
            out: sender,
            update: vec![],
            node_input_action,
            node_output_action,
        };
        for _ in 0..MAX_NUMBER_OF_NODE {
            machine.make_empty_node();
        }

        (machine, receiver)
    }
    pub fn run(mut self) -> Msg {
        let code = Arc::new(Mutex::new(None));
        let code_is_updated = Arc::new(Mutex::new(false));
        let timer = timer();
        let code_clone = code.clone();
        let upd_clone = code_is_updated.clone();
        thread::spawn(move || {
            let code_mtx = code;
            loop {
                if check_if_true(&timer) {
                    if let Err(e) = self.exec_upd() {
                        write_file_append(format!("Update Error : {:?}", e))
                    }
                }

                if let Some(newcode) = Msg::try_receive_code(&code_mtx, &code_is_updated) {
                    self.new_code(newcode);
                }
            }
        });

        Msg {
            code: code_clone,
            code_is_updated: upd_clone,
        }
    }

    fn exec_upd(&mut self) -> Result<(), RuntimeErr> {
        self.update.push(Insn::Halt);
        let code = &self.update[0] as *const Insn;
        let res = self.exec_insn(code);

        res?;
        let top = self.update.pop();
        assert!(matches!(top, Some(Insn::Halt)));
        write_file_append(format!("{:?}\n", self.node_v));

        Ok(())
    }

    fn exec_insn(&mut self, insn: *const Insn) -> Result<Value, RuntimeErr> {
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
                                rip = rip.offset(*offset as isize - 1);
                            }
                        }
                        _ => panic!(),
                    },
                    Insn::J(offset) => rip = rip.offset(*offset as isize - 1),
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
                            return Ok(v);
                        } else {
                            panic!()
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
                        self.node_v[*u] = v;
                        self.node_input_action[*u] = InputAction::Insn(insn.clone());
                    }
                    Insn::GetNode(i) => self.stack.push(self.node_v[*i].clone()),
                    Insn::SetNode(i) => {
                        let v = self.stack.pop().unwrap();
                        if let Some(f) = self.node_output_action[*i] {
                            f(&v)
                        }
                        self.node_v[*i] = v;
                    }
                    Insn::Placeholder => panic!(),
                    Insn::Halt => {
                        assert!(self.stack.is_empty());
                        return Ok(Value::Nil);
                    }
                    Insn::UpdateNode(i) => match &self.node_input_action[*i] {
                        InputAction::Device(f) => self.stack.push(f()),
                        InputAction::Insn(insn) => {
                            self.stack.push(Value::Usize(rbp));
                            self.stack.push(Value::Insn(rip));
                            rip = &insn[0] as *const Insn;
                            rip = rip.offset(-1);
                        }
                        InputAction::None => {
                            self.stack.push(Value::Nil);
                        }
                    },
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
                    Insn::SaveLast => std::mem::swap(&mut self.node_v_last, &mut self.node_v),
                    Insn::GetLast(i) => self.stack.push(self.node_v_last[*i].clone()),
                    Insn::None => todo!(),
                }
                rip = rip.offset(1);
                if DEBUG {
                    write_file_append(format!("{:?}\n", self));
                }

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
                let st = Instant::now();
                let res = self.exec_insn(&init[0] as *const Insn); // codes for defining node is contained in init
                let ed = Instant::now();
                self.update = upd;
                let msg = if let Ok(Value::Nil) = res {
                    format!(
                        "Node was defined successfully [{}us]",
                        ed.duration_since(st).as_micros()
                    )
                } else {
                    format!("Could not define node")
                };
                self.send_msg(msg)
            }
            Code::Exp(exp) => {
                let st = Instant::now();
                let res = self.exec_insn(&exp[0] as *const Insn);
                let ed = Instant::now();
                let msg = if let Ok(v) = res {
                    format!("[OK] {:?} ({}us)", v, ed.duration_since(st).as_micros())
                } else {
                    format!("[ERROR]")
                };
                self.send_msg(msg);
                // Exit returns value into channel, so doesn't need to send message
            }
        }
    }
}

fn mtx_swap<T>(mtx: &Arc<Mutex<T>>, t: &mut T) {
    std::mem::swap(mtx.lock().as_deref_mut().unwrap(), t)
}
