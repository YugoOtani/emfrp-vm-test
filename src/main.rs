pub mod ast;
pub mod compile;
pub mod datastructure;
pub mod dependency;
pub mod emtypes;
pub mod exec;
pub mod insn;
pub mod machine;
pub mod qstr;

use crate::compile::*;
use lalrpop_util::lalrpop_mod;
use machine::*;

use std::io::{prelude::*, stdin, stdout};

use crate::ast::*;
use crate::emfrp::*;
use std::time::Instant;

lalrpop_mod!(pub emfrp);
const CONSOLE: &str = " > ";
const CONSOLE2: &str = "...";
const DEBUG: bool = true;
const MACHINE_MEMCHECK_MILLIS: u64 = 100;
const MACHINE_FILE: &str = "machine_state.txt";
const UPD_FREQUENCY_MS: u64 = 1000;
const INITIAL_NODE_SIZE: usize = 1;
fn main() {
    let pipe = machine_run();
    let parser_prog = ProgramParser::new();
    let parser_def = DefParser::new();
    let mut cmp = Compiler::new();

    for _ in 0.. {
        stdout().flush().unwrap();
        print!("{CONSOLE}");
        stdout().flush().unwrap();
        let mut input = String::new();
        stdin().read_line(&mut input).unwrap();

        let prog = if let "{" = input.trim() {
            let mut v = vec![];

            loop {
                let mut input2 = String::new();
                stdout().flush().unwrap();
                print!("{CONSOLE2}");
                stdout().flush().unwrap();
                stdin().read_line(&mut input2).unwrap();
                if let "}" = input2.trim() {
                    break Program::Defs(v);
                } else {
                    match parser_def.parse(&input2) {
                        Ok(res) => v.push(res),
                        Err(msg) => {
                            println!("parse error : expected definition or '}}'");
                            println!("{}", msg)
                        }
                    }
                }
            }
        } else {
            match parser_prog.parse(&input) {
                Ok(res) => res,
                Err(msg) => {
                    println!("parse error : {:?}", msg);
                    continue;
                }
            }
        };
        let compile_st = Instant::now();
        let compiled = cmp.compile(&prog);
        let compile_ed = Instant::now();
        if DEBUG {
            println!(
                "compile time : {}us",
                compile_ed.duration_since(compile_st).as_micros()
            )
        }
        match compiled {
            Ok(res) => {
                let mut code = match res {
                    CompiledCode::DefNode { init, upd } => Code::DefNode { init, upd },
                    CompiledCode::Exp(e) => Code::Exp(e),
                };
                println!("{:?}", code);
                while let Some(returned_code) = pipe.send_code(code) {
                    code = returned_code
                }
                let msg = pipe.get_msg();
                println!("{}", msg);
            }
            Err(msg) => println!("{:?}", msg),
        }
    }
}

#[test]
fn lalrpop_test() {}
