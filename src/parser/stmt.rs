use crate::parser::column::Column;

use super::expression::Expression;

#[derive(PartialEq, Debug)]
pub enum Statement {
    Begin(BeginStmt),
    Commit,
    Rollback,
    Explain(ExplainStmt),
    CreateTable(CreateTableStmt),
    DropTable(DropTableStmt),
    Delete(DeleteTableStmt),
    Insert(InsertStmt),
    Update(UpdateStmt),
    Select(SelectStmt),
}

impl std::fmt::Display for Statement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Begin(begin_stmt) => {
                write!(f, "Statement: {}", begin_stmt)
            }
            Self::Commit => {
                write!(f, "Statement: Commit")
            }
            _ => {
                write!(f, "Statement: Unknown")
            }
        }
    }
}

#[derive(Eq, PartialEq, Debug)]
pub struct BeginStmt {
    pub is_readonly: bool,
    pub version: Option<u64>,
}

impl std::fmt::Display for BeginStmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "BeginStmt [is_readonly: {} version: {}]",
            self.is_readonly,
            match self.version {
                Some(n) => n.to_string(),
                None => "None".to_string(),
            },
        )
    }
}

#[derive(Eq, PartialEq, Debug)]
pub struct ExplainStmt {}
#[derive(PartialEq, Debug)]
pub struct CreateTableStmt {
    pub columns: Vec<Column>,
    pub table_name: String,
}
#[derive(Eq, PartialEq, Debug)]
pub struct DropTableStmt {}
#[derive(Eq, PartialEq, Debug)]
pub struct DeleteTableStmt {}
#[derive(Eq, PartialEq, Debug)]
pub struct InsertStmt {}
#[derive(Eq, PartialEq, Debug)]
pub struct UpdateStmt {}

#[derive(Debug, PartialEq, Clone)]
pub enum FromItem {
    Table {
        name: String,
        alias: Option<String>,
    },
    Join {
        left: Box<FromItem>,
        right: Box<FromItem>,
        join_type: JoinType,
        predicate: Option<Expression>,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub enum JoinType {
    Left,
    Right,
    Outer,
    Inner,
}

#[derive(Debug, PartialEq, Clone)]
pub enum OrderByType {
    Asc,
    Desc,
}

// 1 + 2 + 3 + !4 * (3 + test.id)

#[derive(Debug, PartialEq, Clone)]
pub struct SelectStmt {
    pub selects: Vec<(Expression, Option<String>)>,
    pub froms: Vec<FromItem>,
    pub wheres: Option<Expression>,
    pub group_by: Vec<Expression>,
    pub having: Option<Expression>,
    pub order: Vec<(Expression, OrderByType)>,
    pub offset: Option<Expression>,
    pub limit: Option<Expression>,
}
