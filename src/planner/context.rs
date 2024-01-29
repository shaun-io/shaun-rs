use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
    sync::Arc,
};

use crate::{
    catalog::{self, column::Column, tableinfo::TableInfo, TableInfoRef},
    error::{
        Error,
        Error::{Internal, Plan},
        Result,
    },
    fmt_err,
    store::table::{self, Table},
    types::expr::Expr,
};

#[derive(Debug)]
pub struct PlanContext {
    /// table_name => tableinfo
    pub(crate) table_map: HashMap<String, TableInfoRef>,
    // column_name => table_name,
    pub(crate) column_map: HashMap<String, Vec<String>>,
}

impl PlanContext {
    pub(crate) fn new() -> Self {
        Self {
            table_map: HashMap::new(),
            column_map: HashMap::new(),
        }
    }

    pub(crate) fn info_by_name(&self, name: &str) -> Option<TableInfoRef> {
        match self.table_map.get(name) {
            Some(info) => Some(Arc::clone(info)),
            None => None,
        }
    }

    pub(crate) fn tablenames_by_columnname(&self, column_name: &str) -> Option<Vec<String>> {
        match self.column_map.get(column_name) {
            Some(table_names) => {
                return Some(Vec::clone(table_names));
            }
            None => None,
        }
    }

    fn insert_column_info(&mut self, table_info: &TableInfoRef) {
        for col in &table_info.schema.columns {
            if let Some(table_names) = self.column_map.get_mut(&col.name) {
                table_names.push(table_info.name.clone());
            } else {
                self.column_map
                    .insert(col.name.to_owned(), vec![table_info.name.to_owned()]);
            }
        }

    }

    pub(crate) fn insert_table_info(
        &mut self,
        table_name: &str,
        alias: &Option<String>,
        info: &TableInfoRef,
    ) -> Result<()> {
        match alias {
            Some(alias_name) => {
                match (
                    self.table_map.get(table_name),
                    self.table_map.get(alias_name),
                ) {
                    (None, None) => {
                        self.table_map
                            .insert(table_name.to_owned(), Arc::clone(info));
                        self.table_map
                            .insert(alias_name.to_owned(), Arc::clone(info));

                        self.insert_column_info(info);
                        Ok(())
                    }
                    (None, Some(_)) | (Some(_), None) | (Some(_), Some(_)) => {
                        Err(Plan(fmt_err!("{table_name} {alias_name} is exist")))
                    }
                }
            }
            None => match self.table_map.get(table_name) {
                Some(_) => Err(Plan(fmt_err!("{table_name} is exist"))),
                None => {
                    self.table_map
                        .insert(table_name.to_owned(), Arc::clone(info));

                    self.insert_column_info(info);
                    Ok(())
                }
            },
        }
    }
}
