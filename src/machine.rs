use std::{
    sync::{
        mpsc::{self, Receiver, Sender},
        Mutex,
    },
    thread::{self, JoinHandle},
};

use crate::{exec::*, insn::*};

#[derive(Debug)]
pub struct ChangeCode {
    code: Vec<Insn2>,
    node: Vec<Value>,
}
unsafe impl Send for ChangeCode {}
impl ChangeCode {
    pub fn new(code: Vec<Insn2>) -> Self {
        ChangeCode { code, node: vec![] }
    }
}

pub fn run(out: Sender<String>) -> (JoinHandle<()>, Sender<ChangeCode>) {
    let (tx, rx) = mpsc::channel();
    (
        thread::spawn(move || loop {
            match rx.recv() {
                Ok(ChangeCode { code, node: _ }) => /*match exec(code) {
                    Ok(v) => println!("Ok : {:?}", v),
                    Err(msg) => println!("{:?}", msg),
                }*/out.send(format!("{:?}",code)),
                Err(_) => return, // sender is dropped?
            };
        }),
        tx,
    )
}
