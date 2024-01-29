use super::value::Value;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    ColumnExpr {
        column_index: i32,
        table_name: String,
        column_name: String,
    },
    Alias(Box<Expr>, String),
    Literal(Value),
    BinaryExpr {
        left: Box<Expr>,
        op: Operator,
        right: Box<Expr>,
    },
    Agg {
        agg_type: AggExpr,
        args: Vec<Expr>,
    },
}

pub static SUPPORT_AGG_NAME: &[&str] = &["MIN", "MAX", "COUNT", "SUM"];

#[derive(Debug, Clone, PartialEq)]
pub enum AggExpr {
    Min,
    Max,
    Count,
    Sum,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Operator {
    Eq,
    NotEq,
    Lt,
    LtEq,
    Gt,
    GtEq,
    Plus,
    Minus,
    Multiply,
    Divide,
    Modules,
    And,
    Or,
    Xor,
}
