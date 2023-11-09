use crate::ast::*;

pub enum SortResult<'a> {
    Success(Vec<&'a Id>),
    CicularRef(Vec<&'a Id>),
}
