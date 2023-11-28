use crate::ast::*;
use crate::compile::*;
use grammer::*;
use serial2::*;
use std::io::*;

use lalrpop_util::lalrpop_mod;

pub mod ast;
pub mod compile;
pub mod datastructure;
pub mod dependency;
pub mod emtypes;
pub mod exec;
pub mod insn;
pub mod qstr;
lalrpop_mod!(grammer);
const MACHINE_FILE: &str = "machine_state.txt";
const UART_FILE: &str = "/dev/cu.usbserial-0001";
const BAUD_RATE: u32 = 115200;
const UPD_FREQUENCY_MS: u64 = 1000;
const MAX_NUMBER_OF_NODE: usize = 100;
const DEBUG: bool = true;
const CONSOLE: &str = " > ";
const CONSOLE2: &str = "...";
fn main() {
    /*let mut port = SerialPort::open(UART_FILE, BAUD_RATE).unwrap();
    let mut settings = port.get_configuration().unwrap();
    settings.set_stop_bits(StopBits::One);
    settings.set_flow_control(FlowControl::None);
    settings.set_char_size(CharSize::Bits8);
    port.set_configuration(&settings).unwrap();

    port.write_all(b"hello,world").unwrap();

    port.flush().unwrap();
    println!("written");
    let mut buf: Vec<u8> = iter::repeat(0).take(5).collect();
    port.read(&mut buf).unwrap();
    println!("{:?}", buf);
    for c in port.bytes() {
        println!("{}", c.unwrap())
    }*/
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
        let mut ret = vec![];
        let (init, upd) = match cmp.compile(&prog) {
            Ok(res) => match res {
                CompiledCode::DefNode { init, upd } => (init, upd),
                CompiledCode::Exp(e) => (e, vec![]),
            },
            Err(msg) => {
                println!("{:?}", msg);
                continue;
            }
        };
        if DEBUG {
            println!("init:");
            for insn in &init {
                println!("  {:?}", insn)
            }
            println!("update:");
            for insn in &upd {
                println!("  {:?}", insn)
            }
        }
        for _ in 0..8 {
            ret.push(0);
        }
        for insn in init {
            insn.push_byte_code(&mut ret);
        }
        let ilen = ret.len() - 8;
        for insn in upd {
            insn.push_byte_code(&mut ret);
        }
        let ulen = ret.len() - (8 + ilen);
        for (i, b) in (ilen as i32).to_le_bytes().iter().enumerate() {
            ret[i] = *b;
        }
        for (i, b) in (ulen as i32).to_le_bytes().iter().enumerate() {
            ret[4 + i] = *b;
        }
        println!("{:?}", ret);
    }
}
