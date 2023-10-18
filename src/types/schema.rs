use crate::parser::data_type::DataType;
use std::collections::HashMap;

use super::value::Value;

#[derive(Debug, Clone)]
pub struct ColumnDef {
    pub name: String,
    pub data_type: DataType,
    pub is_primary_key: bool,
    pub is_nullable: bool,
    pub default: Option<Value>,
    pub is_unique: bool,
    pub references: Option<String>,
    pub has_index: bool,
}

#[derive(Debug, Clone)]
pub struct Table {
    // 表的 id
    pub table_id: u64,
    // 表的名称
    pub name: String,
    // 列的信息
    pub columns: Vec<ColumnDef>,
    // column_name => column_idx(columns)
    pub column_idx: HashMap<String, usize>,
    // column_id => column_idx(columns)
    pub idx_column_name: HashMap<u64, usize>,
}

#[derive(Debug, Clone)]
pub struct TableSet {
    pub table_vecs: Vec<Table>,
    // table_name => table
    pub table_set: HashMap<String, usize>,
}
