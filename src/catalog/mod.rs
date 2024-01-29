use std::{
    collections::HashMap,
    io::IntoInnerError,
    sync::{
        atomic::{self, AtomicI32},
        Arc,
    },
};

use log::error;

use self::{
    schema::Schema,
    tableinfo::{IndexInfo, TableInfo},
};
use crate::{
    error::{Error, Error::Internal, Result},
    fmt_err,
};
pub mod column;
pub mod schema;
pub mod tableinfo;

pub type TableID = i32;
pub type TableInfoRef = Arc<TableInfo>;
pub type IndexInfoRef = Arc<IndexInfo>;

#[derive(Debug)]
pub struct CataLog {
    id_table_map: HashMap<i32, Arc<TableInfo>>,
    name_id_map: HashMap<String, i32>,
    next_table_id: AtomicI32,
    id_index_map: HashMap<i32, IndexInfo>,
    index_names_map: HashMap<String, HashMap<String, i32>>,
}

impl CataLog {
    pub fn default() -> Self {
        Self {
            id_index_map: HashMap::new(),
            name_id_map: HashMap::new(),
            next_table_id: 0.into(),
            id_table_map: HashMap::new(),
            index_names_map: HashMap::new(),
        }
    }

    pub fn create_table(&mut self, table_name: &String, schema: &Schema) -> Result<TableInfoRef> {
        // 表不能已经存在
        if self.name_id_map.contains_key(table_name) {
            return Err(Internal(fmt_err!("{table_name} has been exist")));
        }

        let table_id = self.next_table_id.fetch_add(1, atomic::Ordering::Release);
        let table_info = Arc::new(TableInfo::new(table_name, table_id, schema));

        // 更新元数据信息
        self.id_table_map.insert(table_id, Arc::clone(&table_info));
        self.name_id_map.insert(table_name.clone(), table_id);
        self.index_names_map
            .insert(table_name.clone(), HashMap::new());

        Ok(table_info)
    }

    pub fn is_table_exist(&self, table_name: &str) -> bool {
        match self.name_id_map.get(table_name) {
            Some(_) => true,
            None => false,
        }
    }

    pub fn table_by_name(&self, table_name: &str) -> Option<TableInfoRef> {
        match self.name_id_map.get(table_name) {
            Some(id) => match self.id_table_map.get(id) {
                Some(table) => Some(Arc::clone(table)),
                None => {
                    error!("table_name: {table_name} {id} is not exist");
                    None
                }
            },
            None => None,
        }
    }

    pub fn table_by_id(&self, table_id: TableID) -> Option<TableInfoRef> {
        match self.id_table_map.get(&table_id) {
            Some(table_info) => Some(Arc::clone(table_info)),
            None => None,
        }
    }

    pub fn create_index(
        &mut self,
        index_name: &String,
        table_name: &String,
        shema: &Schema,
        key_shema: &Schema,
        key_attrs: Vec<u32>,
        key_size: usize,
        is_primary_key: bool,
    ) -> Result<IndexInfoRef> {
        match self.name_id_map.get(table_name) {
            Some(_) => {}
            None => {
                return Err(Internal(fmt_err!("{table_name} is not exist in catalog")));
            }
        };

        match self.index_names_map.get(table_name) {
            Some(table_index) => match table_index.get(index_name) {
                Some(_) => {}
                None => {
                    return Err(Internal(fmt_err!("{table_name} is not exist")));
                }
            },
            None => {
                return Err(Internal(fmt_err!("{table_name} should be exist but not")));
            }
        };

        let table_meta = self.table_by_name(table_name).unwrap();

        unimplemented!()
    }
}
