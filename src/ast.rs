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
pub enum Top<'a> {
    Defs(Vec<Def<'a>>),
    Exp(Exp<'a>),
}
#[derive(Debug, Clone)]
pub enum Def<'a> {
    Node(DefNode<'a>),
    Data(DefData<'a>),
    Func(DefFunc<'a>),
}

#[derive(Debug, Clone)]
pub struct DefNode<'a> {
    pub name: Id<'a>,
    pub init: Exp<'a>,
    pub val: Exp<'a>,
}
#[derive(Debug, Clone)]
pub struct DefData<'a> {
    pub name: Id<'a>,
    pub val: Exp<'a>,
}

#[derive(Debug, Clone)]
pub struct DefFunc<'a> {
    pub name: Id<'a>,
    pub params: Vec<Id<'a>>,
    pub body: Exp<'a>,
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
    Id(Id<'a>),
}

#[derive(Debug, Clone)]
pub struct Id<'a> {
    // pointer to unique string(qstr)
    // tokens of program is stored in qstr pool and pos points to its index
    pub s: &'a str,
}
