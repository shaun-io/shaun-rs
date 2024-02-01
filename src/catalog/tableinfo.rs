use super::schema::Schema;

#[derive(Debug)]
pub struct TableInfo {
    pub name: String,
    pub id: i32,
    pub schema: Schema,
}

impl TableInfo {
    pub fn new(table_name: &String, id: i32, schema: &Schema) -> Self {
        Self {
            name: table_name.to_owned(),
            id,
            schema: schema.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct IndexInfo {
    pub key_schema: Schema,
    pub name: String,
    pub id: i32,
    pub table_name: String,
    pub key_size: usize,
    pub is_primary_key: bool,
}
