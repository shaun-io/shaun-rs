use std::sync::Arc;

use super::column::Column;

#[derive(Debug, Clone, PartialEq)]
pub struct Schema {
    pub columns: Vec<Column>,
}

pub type SchemaRef = Arc<Schema>;

impl Schema {
    pub fn default() -> Self {
        Self { columns: vec![] }
    }
}
