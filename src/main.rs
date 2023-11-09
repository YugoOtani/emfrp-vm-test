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
use std::sync::mpsc;
//TODO

// vectorのサイズをあらかじめ計算する
// node再定義時、前回のノードの値との対応をどうするか
// node同士、nodeとdataの名前の衝突
// node用の領域をまとめて確保する
// nodeの書き換え時、ノードの再確保を少なくする
// dataの実装
// @lastの実装
// Global変数
// 循環参照検知
// replにおいてexpをprintする

//compile時検査
// nodeのinitの値の計算ができるか
// nodeのinitの部分が省略されているか、ノードを参照する場合
// nodeのスコープ
// machine側のエラーの処理方法 exec時のunwrap(addなのにstackが空の場合にどうするか)など
// node用のメモリ領域をスタックと別に確保するかどうか。そうするならなぜそうしたか
// 参照されなくなったnode, dataの削除
// compile.rs のunsafeブロック

lalrpop_mod!(pub emfrp);
const CONSOLE: &str = " > ";
const CONSOLE2: &str = "...";
const DEBUG: bool = true;
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

        match cmp.compile(&prog) {
            Ok(res) => {
                match res {
                    CompileResult::DefNode(code) => {
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
