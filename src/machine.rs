use std::{
    sync::{
        mpsc::{self, *},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
};

use crate::{exec::*, insn::*};
#[derive(Debug)]
pub enum Msg {
    DefNode(Vec<Insn2>),
    Exp(Vec<Insn2>),
}
unsafe impl Send for Msg {}
unsafe impl Sync for Msg {}

pub fn machine_run(out: Sender<String>) -> (JoinHandle<()>, Sender<Msg>) {
    let (code_sender, code_receiver) = mpsc::channel();
    let shared_mem = Arc::new(Mutex::new(vec![]));
    let mem_updated = Arc::new(Mutex::new(false));
    let mem_clone = shared_mem.clone();
    let upd_clone = mem_updated.clone();

    (
        thread::spawn(move || {
            thread::spawn(move || exec(upd_clone, mem_clone, out));
            //move rx, out
            loop {
                match code_receiver.recv() {
                    Ok(msg) => {
                        match msg {
                            Msg::DefNode(mut code) => {
                                let mut mtx = mem_updated.lock().unwrap();

                                // if updated = false, updated mem is not written by the thread above
                                if !*mtx {
                                    swap(shared_mem.clone(), &mut code);
                                    *mtx = true;
                                }
                            }
                            Msg::Exp(_) => todo!(),
                        }
                    }

                    Err(_) => return, // sender is dropped?
                };
            }
        }),
        code_sender,
    )
}
fn swap<T>(mtx: Arc<Mutex<T>>, t: &mut T) {
    std::mem::swap(mtx.lock().as_deref_mut().unwrap(), t)
}
