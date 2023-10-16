use crate::qstr::{QstrIndex, QstrPool};

#[derive(Debug, Clone)]
pub enum Token {
    KwInt,
    Int(i32),
    KwBool,
    Bool(bool),
    KwNode,
    KwFunc,
    KwData,
    KwInit,
    Dot,
    Comma,
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
    Semicoron,
    Eq,
    Gt,
    Lt,
    Ge,
    Le,
    Eof,
    Id(QstrIndex),
}
pub fn tokenize(input: &str) -> Option<(Vec<Token>, QstrPool)> {
    let mut ret = vec![];
    let mut pool = QstrPool::empty();
    let s: Vec<char> = input.chars().collect();
    let mut i = 0;
    while let Some(c) = s.get(i) {
        match *c {
            c if c.is_whitespace() => {}
            '.' => ret.push(Token::Dot),
            ',' => ret.push(Token::Comma),
            '(' => ret.push(Token::RParen),
            ')' => ret.push(Token::LParen),
            '{' => ret.push(Token::RBrace),
            '}' => ret.push(Token::LBrace),
            '[' => ret.push(Token::RBracket),
            ']' => ret.push(Token::LBracket),
            '+' => ret.push(Token::Plus),
            '-' => ret.push(Token::Minus),
            '*' => ret.push(Token::Astarisk),
            ';' => ret.push(Token::Semicoron),
            '=' => {
                if let Some('>') = s.get(i) {
                    ret.push(Token::Ge);
                    i += 1;
                } else {
                    ret.push(Token::Eq);
                }
            }
            '>' => ret.push(Token::Gt),
            '<' => {
                if let Some('=') = s.get(i) {
                    ret.push(Token::Le);
                    i += 1;
                } else {
                    ret.push(Token::Lt);
                }
            }
            '0'..='9' => {
                if let Some((len, tkn)) = self::tokenize_num(i, &s) {
                    i = i + len - 1;
                    ret.push(tkn);
                } else {
                    return None;
                }
            }
            'a'..='z' | 'A'..='Z' => {
                let mut i2 = i + 1;
                while let Some('0'..='9' | 'a'..='z' | 'A'..='Z') = s.get(i2) {
                    i2 += 1
                }
                if let Some(kw_tkn) = some_if_kw(&s[i..i2]) {
                    ret.push(kw_tkn)
                } else {
                    let qind;
                    (pool, qind) = pool.insert(s[i..i2].into_iter().collect());
                    ret.push(Token::Id(qind))
                }
                i = i2 - 1
            }
            _ => return None,
        }
        i += 1;
    }
    ret.push(Token::Eof);
    Some((ret, pool))
}
fn tokenize_num(start: usize, v: &[char]) -> Option<(usize, Token)> {
    let mut i = start + 1;

    if let '0' = v[start] {
        if let Some('0'..='9' | 'a'..='z' | 'A'..='Z') = v.get(start + 1).map(|u| *u as char) {
            return None;
        } else {
            return Some((1, Token::Int(0)));
        }
    } else {
        while let Some('0'..='9') = v.get(i).map(|u| *u as char) {
            i += 1;
        }
        let opt_n = v.iter().collect::<String>().parse::<i32>().ok();
        opt_n.map(|n| (i - start, Token::Int(n)))
    }
}

fn some_if_kw(s: &[char]) -> Option<Token> {
    match s {
        ['I', 'n', 't'] => Some(Token::KwInt),
        ['B', 'o', 'o', 'l'] => Some(Token::KwBool),
        ['i', 'n', 'i', 't'] => Some(Token::KwInit),
        ['d', 'a', 't', 'a'] => Some(Token::KwData),
        ['n', 'o', 'd', 'e'] => Some(Token::KwNode),
        ['f', 'u', 'n', 'c'] => Some(Token::KwFunc),
        _ => None,
    }
}
