pub mod ast;
pub mod compile;
pub mod emtypes;
pub mod exec;
pub mod insn;
pub mod machine;
pub mod qstr;
use compile::compile;
use exec::exec;
use exec::Value;
use lalrpop_util::lalrpop_mod;
use std::io::{self, BufRead};

lalrpop_mod!(pub emfrp);
fn main() {
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    interpret(input);
}
fn interpret(s: String) {
    let ast = emfrp::TopParser::new().parse(&s);
    match ast {
        Ok(ast) => match compile(&ast) {
            Ok(insns) => match exec(insns) {
                Ok(v) => println!("{:?}", v),
                Err(kind) => println!("{:?}", kind),
            },
            Err(kind) => println!("{:?}", kind),
        },
        Err(msg) => println!("{:?}", msg),
    }
}

#[test]
fn lalrpop_test() {}
