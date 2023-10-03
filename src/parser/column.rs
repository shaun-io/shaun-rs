use crate::parser::expression::Expression;
use crate::parser::DataType;

#[derive(PartialEq, Debug, Clone)]
pub struct Column {
    pub name: String,
    pub data_type: DataType,
    pub primary_key: bool,
    pub nullable: Option<bool>,
    pub default: Option<Expression>,
    pub unique: bool,
    pub index: bool,
    pub references: Option<String>,
}
