use crate::types::schema::*;
use std::collections::HashMap;
use std::sync::Arc;

pub struct CataLog {
    db_info: TableSet,
}

impl CataLog {
    pub fn create_table(&mut self, table_name: String, column_infos: Vec<ColumnDef>) {
        let current_id = self.db_info.table_vecs.len() as u64;
        self.db_info.table_vecs.push(Table {
            table_id: current_id,
            name: table_name.clone(),
            columns: column_infos,
            column_idx: HashMap::new(),
            idx_column_name: HashMap::new(),
        });
        self.db_info
            .table_set
            .insert(table_name, current_id as usize);
    }

    pub fn get_table_info(&self, table_name: String) -> Option<&Table> {
        match self.db_info.table_set.get(&table_name) {
            Some(idx) => Some(&self.db_info.table_vecs[*idx]),
            None => None,
        }
    }

    pub fn drop_table_info(&mut self, table_name: String) -> bool {
        match self.db_info.table_set.get(&table_name) {
            Some(idx) => {
                self.db_info.table_vecs.remove(*idx);
                true
            }
            None => false,
        }
    }

    pub fn is_table_exist(&self, table_name: &String) -> bool {
        self.db_info.table_set.contains_key(table_name)
    }
}

unsafe impl Send for CataLog {}

unsafe impl Sync for CataLog {}
