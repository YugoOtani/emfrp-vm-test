pub mod ast;
pub mod compile;
pub mod dependency;
pub mod emtypes;
pub mod exec;
pub mod insn;
pub mod machine;
pub mod qstr;
use compile::compile;
use exec::exec;

use lalrpop_util::lalrpop_mod;

use std::fs::File;
use std::io::prelude::*;

lalrpop_mod!(pub emfrp);
fn main() {
    let mut f = File::open("sample.txt").expect("file not found");

    let mut content = String::new();
    f.read_to_string(&mut content).unwrap();
    interpret(content);
}
fn interpret(s: String) {
    let ast = emfrp::TopParser::new().parse(&s);
    match ast {
        Ok(ast) => match compile(&ast) {
            Ok(insns) => {
                for insn in &insns {
                    println!("{:?}", insn);
                }
                match exec(insns) {
                    Ok(v) => println!("{:?}", v),
                    Err(kind) => {
                        println!("{:?}", ast);
                        println!("{:?}", kind);
                    }
                }
            }
            Err(kind) => {
                println!("{:?}", ast);
                println!("{:?}", kind)
            }
        },
        Err(msg) => println!("{:?}", msg),
    }
}

#[test]
fn lalrpop_test() {}
