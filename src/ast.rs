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
pub enum Program {
    Defs(Vec<Def>),
    Def(Def),
    Exp(Exp),
}
#[derive(Debug, Clone)]
pub enum Def {
    Node {
        name: Id,
        init: Exp,
        val: Exp,
    },
    Data {
        name: Id,
        val: Exp,
    },
    Func {
        name: Id,
        params: Vec<Id>,
        body: Exp,
    },
}

#[derive(Debug, Clone)]
pub enum Exp {
    If {
        cond: Box<Exp>,
        then: Box<Exp>,
        els: Box<Exp>,
    },
    Add(Box<Exp>, Box<Term>),
    Term(Box<Term>),
}
#[derive(Debug, Clone)]
pub enum Term {
    Mul(Box<Term>, Box<Term>),
    Int(i32),
    FnCall(Box<Id>, Vec<Exp>),
    Bool(bool),
    Last(Id),
    Id(Id),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Id {
    pub s: String,
}
