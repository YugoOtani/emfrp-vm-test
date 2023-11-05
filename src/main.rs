pub mod ast;
pub mod compile;
pub mod dependency;
pub mod emtypes;
pub mod exec;
pub mod insn;
pub mod machine;
pub mod qstr;

use crate::compile::*;
use insn::*;
use lalrpop_util::lalrpop_mod;
use machine::*;

use std::io::{prelude::*, stdin, stdout};

use crate::ast::*;
use crate::emfrp::*;
use std::sync::mpsc;
//TODO
// nodeのinitの検査
// @lastの依存関係
// Global変数
// node を　dataでおきかえたとき
// 循環参照検知
// node a = EXPのEXPにおける、依存関係解析時のnodeとdataの区別)
// nodeのスコープ、存在検査
// move upd_formula out of dependencies
// dependencyのメモリ量 (clone多用)
// compile.rs l52
// init[val] におけるvalにnodeがないか検査
// @lastのemit_code
// machineで、runの各ループごとにexecを呼んでおり、そこで新しいスタックを用意しているのでそれを改善する
// 新しいinsn2を送ったあと、それまでのnodeの値との整合性をどうするか

lalrpop_mod!(pub emfrp);
const CONSOLE: &str = " > ";
const CONSOLE2: &str = "...";
const DEBUG: bool = false;
fn main() {
    let (sender, from_machine) = mpsc::channel();
    let (_, sender) = run(sender);

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
        if DEBUG {
            println!("{:?}", prog);
        }
        match cmp.compile(&prog) {
            Ok(res) => {
                let insn2 = to_insn2(res, cmp.sorted_nodes());

                sender.send(ChangeCode::new(insn2)).unwrap();

                match from_machine.recv() {
                    Ok(msg) => println!("{msg}"),
                    Err(_) => unreachable!(), //? channel is closed
                }
            }
            Err(msg) => println!("{:?}", msg),
        }
    }
}

#[test]
fn lalrpop_test() {}
