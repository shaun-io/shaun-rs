use crate::catalog::CataLog;
use crate::error::Error;
use crate::fmt_err;
use crate::parser::stmt::*;

use self::logical_plan::{CreateTable, LogicalPlan};

use super::parser::stmt::Statement;
mod aggregate;
mod filter;
mod join;
mod limit;
mod logical_plan;
mod offset;
mod projection;
mod scan;
mod sort;

use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;
use std::sync::RwLock;
type Result<T> = std::result::Result<T, super::error::Error>;

pub struct Binder {
    pub cata_log: Arc<RwLock<CataLog>>,
}

impl Binder {
    pub fn bind_stmt(&mut self, stmt: Statement) -> Result<LogicalPlan> {
        match stmt {
            Statement::CreateTable(create_stmt) => self.bind_create_stmt(create_stmt),
            Statement::Select(select_stmt) => self.bind_select_stmt(select_stmt),
            _ => Err(Error::BinderErr(fmt_err!("not impl statement"))),
        }
    }

    fn is_table_exist(&self, table_name: &String) -> bool {
        self.cata_log.read().unwrap().is_table_exist(table_name)
    }

    fn bind_select_stmt(&mut self, select_stmt: SelectStmt) -> Result<LogicalPlan> {
        unimplemented!()
    }

    fn bind_create_stmt(&mut self, create_stmt: CreateTableStmt) -> Result<LogicalPlan> {
        // 表不能已经存在
        if self.is_table_exist(&create_stmt.table_name) {
            return Err(Error::BinderErr(fmt_err!(
                "table {} already exist",
                create_stmt.table_name
            )));
        }

        // 一张表的列的名称不能相同
        let mut column_name_set = HashSet::new();
        for column in &create_stmt.columns {
            if column_name_set.contains(&column.name) {
                return Err(Error::BinderErr(fmt_err!(
                    "column {} already exist",
                    column.name
                )));
            }
            column_name_set.insert(&column.name);
        }

        // 一张表只能有一个主键
        if create_stmt.columns.iter().filter(|x| x.primary_key).count() > 1 {
            return Err(Error::BinderErr(fmt_err!(
                "table {} has more than one primary key",
                create_stmt.table_name
            )));
        }

        Ok(LogicalPlan::CreateTable(CreateTable::new_create_table(
            create_stmt.table_name,
            create_stmt.columns,
        )))
    }
}
