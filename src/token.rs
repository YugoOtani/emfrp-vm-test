#[derive(Debug, Clone)]
pub enum Token<'a> {
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
    Id(&'a [u8]),
}
pub fn tokenize(input: &str) -> Option<Vec<Token>> {
    let mut ret = vec![];
    let input_b = input.as_bytes();
    let mut i = 0;
    while let Some(c) = input_b.get(i) {
        match *c as char {
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
                if let Some(b'>') = input_b.get(i) {
                    ret.push(Token::Ge);
                    i += 1;
                } else {
                    ret.push(Token::Eq);
                }
            }
            '>' => ret.push(Token::Gt),
            '<' => {
                if let Some(b'=') = input_b.get(i) {
                    ret.push(Token::Le);
                    i += 1;
                } else {
                    ret.push(Token::Lt);
                }
            }
            '0'..='9' => {
                if let Some((len, tkn)) = self::tokenize_num(i, input_b) {
                    i = i + len - 1;
                    ret.push(tkn);
                } else {
                    return None;
                }
            }
            'a'..='z' | 'A'..='Z' => {
                let mut i2 = i + 1;
                while let Some('0'..='9' | 'a'..='z' | 'A'..='Z') =
                    input_b.get(i2).map(|c| *c as char)
                {
                    i2 += 1
                }
                if let Some(kw_tkn) = some_if_kw(&input_b[i..i2]) {
                    ret.push(kw_tkn)
                } else {
                    ret.push(Token::Id(&input_b[i..i2]))
                }
                i = i2 - 1
            }
            _ => return None,
        }
        i += 1;
    }
    ret.push(Token::Eof);
    Some(ret)
}
fn tokenize_num(start: usize, v: &[u8]) -> Option<(usize, Token)> {
    let mut i = start + 1;

    if let '0' = v[start] as char {
        if let Some('0'..='9' | 'a'..='z' | 'A'..='Z') = v.get(start + 1).map(|u| *u as char) {
            return None;
        } else {
            return Some((1, Token::Int(0)));
        }
    } else {
        while let Some('0'..='9') = v.get(i).map(|u| *u as char) {
            i += 1;
        }
        let opt_n = String::from_utf8(v[start..i].to_vec())
            .unwrap()
            .parse::<i32>()
            .ok();
        opt_n.map(|n| (i - start, Token::Int(n)))
    }
}

fn some_if_kw(s: &[u8]) -> Option<Token> {
    match s {
        b"Int" => Some(Token::KwInt),
        b"Bool" => Some(Token::KwBool),
        b"init" => Some(Token::KwInit),
        b"data" => Some(Token::KwData),
        b"node" => Some(Token::KwNode),
        b"func" => Some(Token::KwFunc),
        _ => None,
    }
}
#[test]
fn tokenize_num_test() {
    assert!(matches!(
        tokenize_num(0, b"123;"),
        Some((3, Token::Int(123)))
    ));
    assert!(matches!(
        tokenize_num(2, b"  1230"),
        Some((4, Token::Int(1230)))
    ));
    assert!(matches!(tokenize_num(1, b" 0 ;"), Some((1, Token::Int(0)))));
    assert!(tokenize_num(1, b" 00 ;").is_none());
    assert!(tokenize_num(1, b" 1000000000000000000000 ;").is_none());
}
