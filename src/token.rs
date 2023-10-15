use crate::emtypes::Num;

pub enum Token {
    KwInt,
    Int(Num),
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
    Id(&str), //
}
