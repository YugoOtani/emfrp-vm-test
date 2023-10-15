use crate::{emtypes::Num, qstr::QstrIndex};

pub enum Token {
    KwInt,
    Num(Num),
    KwBool,
    Bool,
    KwNode,
    KwFunc,
    KwData,
    KwInit,
    Dot,
    At,
    RParen, // ()
    LParen,
    RBrace, // {}
    LBrace,
    RBracket, // []
    LBracket,
    Plus,
    Minus,
    Astarisk,
    Id(QstrIndex),
}
