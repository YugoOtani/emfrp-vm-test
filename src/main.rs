use serial2::*;
use std::io::Read;
use std::iter;
use std::time::Duration;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

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
const DEBUG: bool = false;
fn main() {
    let mut port = SerialPort::open(UART_FILE, BAUD_RATE).unwrap();
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
    }
}
