/*
TOP => (DEF)* | EXP
DEF => DEFNODE | DEFDATA | DEFFUNC
DEFNODE => node init[EXP] ID = EXP
DEFDATA => data ID = EXP
DEFFUNC => func ID (PARAMS) = EXP
PARAMS = (ID [, ID]* )?
EXP = if EXP then EXP else EXP  | EXP + TERM  | TERM
TERM = TERM * TERM | FnCall | INTEGER | BOOLEAN | ID
FNCALL = ID(ARGS)
ARGS =  (EXP [, EXP]*)?
ID = [a-zA-Z][a-zA-Z0-9]*
 */
#[derive(Debug, Clone)]
pub enum Program<'a> {
    File(Vec<Def<'a>>),
    Repl(Repl<'a>),
}
#[derive(Debug, Clone)]
pub enum Repl<'a> {
    Exp(Exp<'a>),
    Def(Def<'a>),
}
#[derive(Debug, Clone)]
pub enum Def<'a> {
    Node {
        name: Id<'a>,
        init: Exp<'a>,
        val: Exp<'a>,
    },
    Data {
        name: Id<'a>,
        val: Exp<'a>,
    },
    Func {
        name: Id<'a>,
        params: Vec<Id<'a>>,
        body: Exp<'a>,
    },
}

#[derive(Debug, Clone)]
pub enum Exp<'a> {
    If {
        cond: Box<Exp<'a>>,
        then: Box<Exp<'a>>,
        els: Box<Exp<'a>>,
    },
    Add(Box<Exp<'a>>, Box<Term<'a>>),
    Term(Box<Term<'a>>),
}
#[derive(Debug, Clone)]
pub enum Term<'a> {
    Mul(Box<Term<'a>>, Box<Term<'a>>),
    Int(i32),
    FnCall(Box<Id<'a>>, Vec<Exp<'a>>),
    Bool(bool),
    Last(Id<'a>),
    Id(Id<'a>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Id<'a> {
    pub s: &'a str,
}
