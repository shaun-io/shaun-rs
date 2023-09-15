use crate::parser::expression::Expression;
#[derive(PartialEq, Debug, Clone)]
pub enum Operation {
    And(Box<Expression>, Box<Expression>),
    Not(Box<Expression>),
    Or(Box<Expression>, Box<Expression>),

    NotEqual(Box<Expression>, Box<Expression>),
    Equal(Box<Expression>, Box<Expression>),
    GreaterThan(Box<Expression>, Box<Expression>),
    GreaterThanOrEqual(Box<Expression>, Box<Expression>),
    LessThan(Box<Expression>, Box<Expression>),
    LessThanOrEqual(Box<Expression>, Box<Expression>),

    IsNull(Box<Expression>),

    // + - * !
    Add(Box<Expression>, Box<Expression>),
    Subtract(Box<Expression>, Box<Expression>),
    Multiply(Box<Expression>, Box<Expression>),
    Divide(Box<Expression>, Box<Expression>),

    Assert(Box<Expression>),
    Like(Box<Expression>, Box<Expression>),

    Negate(Box<Expression>),
    BitWiseNot(Box<Expression>),
    // 取余运算符
    Modulo(Box<Expression>, Box<Expression>),
}

impl From<Operation> for Expression {
    fn from(op: Operation) -> Self {
        Self::Operation(op)
    }
}
