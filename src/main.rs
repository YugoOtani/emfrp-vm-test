pub mod ast;
pub mod compile;
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
use std::sync::mpsc;
//TODO
// node用の領域をまとめて確保
// nodeの書き換え時のノードの再確保を少なくできるようにする
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
// expの場合は値を返すが、nodeの場合はループを続ける。この違いをどう表現するか
// exec時のunwrap(addなのにstackが空の場合にどうするか)
// node用のメモリ領域をスタックと別に確保するかどうか。そうするならなぜそうしたか
// 参照されなくなったnode, dataの削除
//今 ->

lalrpop_mod!(pub emfrp);
const CONSOLE: &str = " > ";
const CONSOLE2: &str = "...";
const DEBUG: bool = false;
const MACHINE_MEMCHECK_MILLIS: u64 = 100;
const MACHINE_FILE: &str = "machine_state.txt";
fn main() {
    // channel : machine --> this thread
    // receive result from machine
    let (res_sender, res_receiver) = mpsc::channel();

    // channel : this thread --> machine
    // send bytecode to machine
    let (_, msg_sender) = machine_run(res_sender);

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
                match res {
                    CompileResult::DefNode(code) => {
                        if DEBUG {
                            println!("[Compile Result]");
                            for insn in &code {
                                println!("  {:?}", insn);
                            }
                        }
                        msg_sender.send(Msg::DefNode(code)).unwrap();
                    }
                    CompileResult::Exp(e) => {
                        msg_sender.send(Msg::Exp(e)).unwrap();
                        match res_receiver.recv() {
                            // wait until result comes
                            Ok(res) => println!("{res}"),
                            Err(_) => unreachable!(), //? channel is closed
                        }
                    }
                }
            }
            Err(msg) => println!("{:?}", msg),
        }
    }
}

#[test]
fn lalrpop_test() {}
