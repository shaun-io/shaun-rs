use std::collections::BTreeMap;

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
    Alter(AlterStmt),
    CreateIndex(CreateIndexStmt),
    ShowDatabase,
    ShowTables,
    Set(SetStmt),
    DescribeTable(String),
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

#[derive(PartialEq, Debug)]
pub struct ExplainStmt {
    pub statement: Box<Statement>,
}
#[derive(PartialEq, Debug)]
pub struct CreateTableStmt {
    pub columns: Vec<Column>,
    pub table_name: String,
}
#[derive(Eq, PartialEq, Debug)]
pub struct DropTableStmt {
    pub table_name: String,
}
#[derive(PartialEq, Debug)]
pub struct DeleteTableStmt {
    pub table_name: String,
    pub r#where: Option<Expression>,
}
#[derive(PartialEq, Debug)]
pub struct InsertStmt {
    pub table_name: String,
    pub columns: Option<Vec<String>>,
    pub values: Vec<Vec<Option<Expression>>>,
}
#[derive(PartialEq, Debug)]
pub struct UpdateStmt {
    pub table_name: String,
    pub set: BTreeMap<String, Expression>,
    pub wheres: Option<Expression>,
}

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
    pub froms: Option<Vec<FromItem>>,
    pub wheres: Option<Expression>,
    pub group_by: Option<Vec<Expression>>,
    pub having: Option<Expression>,
    pub order: Option<Vec<(Expression, OrderByType)>>,
    pub offset: Option<Expression>,
    pub limit: Option<Expression>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum AlterType {
    AddColumn(Column),                     // 增加列
    DropColumn(String),                    // 删除列
    ModifyColumn(Column),                  // 修改列数据类型
    RenameColumn(String, String),          // 重命名列名称
    RenameTable(String),                   // 重命名表名称
    AddIndex(Option<String>, Vec<String>), // 增加索引
    RemoveIndex(String),                   // 删除索引
}

#[derive(Debug, PartialEq, Clone)]
pub struct AlterStmt {
    pub alter_type: AlterType,
    pub table_name: String,
}

#[derive(Debug, PartialEq, Clone)]
pub enum TransactionIsolationLevel {
    ReadUncommitted, // 读未提交
    ReadCommitted,   // 读已提交
    RepeatableRead,  // 可重复读
    Serializable,    // 串行化
}

#[derive(Debug, PartialEq, Clone)]
pub enum SetVariableType {
    Transaction(TransactionIsolationLevel),
    Value(SetValue),
}

#[derive(Debug, PartialEq, Clone)]
pub struct SetValue {
    pub variable_name: String,
    pub value: Expression,
}

#[derive(Debug, PartialEq, Clone)]
pub struct SetStmt {
    pub set_value: SetVariableType,
    pub is_session: bool,
}

#[derive(Debug, PartialEq, Clone)]
pub struct CreateIndexStmt {
    pub index_name: String,
    pub is_unique: bool, // 是否是唯一索引
    pub table_name: String,
    pub columns: Vec<String>,
}
