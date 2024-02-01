use std::sync::Arc;

use super::column::Column;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Schema {
    pub columns: Vec<Column>,
}

pub type SchemaRef = Arc<Schema>;
