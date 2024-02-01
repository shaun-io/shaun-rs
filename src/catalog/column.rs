use crate::types::LogicalType;

#[derive(Debug, Clone, PartialEq)]
pub struct Column {
    pub column_type: LogicalType,
    pub name: String,
}

impl Column {
    pub fn new(logical_type: LogicalType, name: &str) -> Self {
        Self {
            column_type: logical_type,
            name: name.to_owned(),
        }
    }
}
