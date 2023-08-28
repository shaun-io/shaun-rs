use std::fmt::Display;
use super::column::Column;

#[derive(Eq, PartialEq, Debug)]
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
                write!(f, "Statment: Unknown")
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
#[derive(Eq, PartialEq, Debug)]
pub struct CreateTableStmt {
    pub columns: Vec::<Column>,
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
#[derive(Eq, PartialEq, Debug)]
pub struct SelectStmt {}
