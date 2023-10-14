/*
TOP => (DEF)* | EXP
DEF => DEFNODE | DEFDATA | DEFFUNC
DEFNODE => node init[EXP] ID = EXP
DEFDATA => data ID = EXP
DEFFUNC => func ID (PARAMS) = EXP
PARAMS = option(ID [, ID]* )
EXP = if EXP then EXP else EXP | EXP + EXP | EXP * EXP | -EXP | FNCALL | TERM
TERM = INTEGER | BOOLEAN | ID
FNCALL = ID(ARGS)
ARGS =  option(EXP [, EXP]*)
ID = [a-zA-Z][a-zA-Z0-9]*
 */
#[derive(Debug, Clone)]
pub enum Top {
    Defs(Vec<Def>),
    Exp(Exp),
}
#[derive(Debug, Clone)]
pub enum Def {
    Node(DefNode),
    Data(DefData),
    Func(DefFunc),
}

#[derive(Debug, Clone)]
pub struct DefNode {
    pub name: Id,
    pub init: Exp,
    pub val: Exp,
}
#[derive(Debug, Clone)]
pub struct DefData {
    pub name: Id,
    pub val: Exp,
}

#[derive(Debug, Clone)]
pub struct DefFunc {
    pub name: Id,
    pub params: Vec<Id>,
    pub body: Exp,
}
#[derive(Debug, Clone)]
pub enum Exp {
    If {
        cond: Box<Exp>,
        then: Box<Exp>,
        els: Box<Exp>,
    },
    Add(Box<Exp>, Box<Exp>),
    Mul(Box<Exp>, Box<Exp>),
    Minus(Box<Exp>),
    FnCall(Id, Vec<Exp>),
    Term(Term),
}
#[derive(Debug, Clone)]
pub enum Term {
    Num(Num),
    Bool(bool),
    Id(Id),
}

#[derive(Debug, Clone)]
pub struct Id {
    // pointer to unique string(qstr)
    // tokens of program is stored in qstr pool and pos points to its index
    pub pos: usize,
}
type Num = i32;
