use std::{collections::HashSet, iter::FilterMap};

use super::Planner;
use crate::{
    error::{Error::Plan, Result},
    fmt_err,
    parser::stmt::CreateTableStmt,
};

impl Planner {
    /// check the create table
    /// ```
    /// 1. table can't already exist in catalog
    /// 2. table can't have multi same column name
    /// 3. table only support one primary key
    /// ```
    pub(crate) fn check_create_table(&mut self, create_stmt: &CreateTableStmt) -> Result<()> {
        if self.catalog.is_table_exist(&create_stmt.table_name) {
            return Err(Plan(fmt_err!(
                "{} is already exist in database",
                &create_stmt.table_name
            )));
        }

        let mut column_name_set = HashSet::new();
        for col in &create_stmt.columns {
            if column_name_set.contains(&col.name) {
                return Err(Plan(fmt_err!("{} is fuzzy in your SQL", col.name)));
            }
            column_name_set.insert(&col.name);
        }

        if create_stmt.columns.iter().filter(|x| x.primary_key).count() != 1 {
            return Err(Plan(fmt_err!(
                "table {} only support one primary key",
                create_stmt.table_name
            )));
        }

        Ok(())
    }
}
