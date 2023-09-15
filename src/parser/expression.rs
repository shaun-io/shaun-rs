use crate::parser::operation::Operation;

#[derive(PartialEq, Debug, Clone)]
pub enum Literal {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
}

#[derive(PartialEq, Debug, Clone)]
pub enum Expression {
    Field(Option<String>, String),
    Column(usize),
    Literal(Literal),
    Function(String, Vec<Expression>),
    Operation(Operation),
}
