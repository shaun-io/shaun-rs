use super::value::Value;

#[derive(Debug, Clone)]
pub enum Expr {
    Constant(Value),              // 常量
    Field(usize, Option<String>), // table_name.field_name

    // expression
    Not(Box<Expr>),
    IsNull(Box<Expr>),
    Negate(Box<Expr>),

    // binary expression
    Eq(Box<Expr>, Box<Expr>),    // =
    NotEq(Box<Expr>, Box<Expr>), // !=
    Lt(Box<Expr>, Box<Expr>),    // <= LessThan
    LtEq(Box<Expr>, Box<Expr>),  // < LessThanOrEqual
    Gt(Box<Expr>, Box<Expr>),    // > GreaterThan
    GtEq(Box<Expr>, Box<Expr>),  // >= GreaterThanOrEqual

    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),

    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
}
