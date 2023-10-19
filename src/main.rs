pub mod ast;
pub mod compile;
pub mod emtypes;
pub mod exec;
pub mod insn;
pub mod machine;
pub mod qstr;
use lalrpop_util::lalrpop_mod;

lalrpop_mod!(pub emfrp);
fn main() {
    println!("Hello, world!");
}
#[test]
fn lalrpop_test() {
    println!(
        "{:?}",
        emfrp::TopParser::new().parse("node init[123] val = val + foo * true")
    );
    assert!(false);
}
