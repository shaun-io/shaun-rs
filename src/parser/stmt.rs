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

pub struct BeginStmt {}

pub struct ExplainStmt {}

pub struct CreateTableStmt {}

pub struct DropTableStmt {}

pub struct DeleteTableStmt {}

pub struct InsertStmt {}

pub struct UpdateStmt {}

pub struct SelectStmt {}
