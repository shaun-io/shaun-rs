use crate::parser::column::Column;

use super::aggregate::Aggregate;
use super::filter::Filter;
use super::join::Join;
use super::limit::Limit;
use super::offset::Offset;
use super::projection::Projection;
use super::scan::Scan;
use super::sort::Sort;

#[derive(Debug, Clone)]
pub enum LogicalPlan {
    // select expr_lists
    Projection(Projection),
    // predicate
    // such as SELECT xxx from xxx where (predicate(filter));
    Filter(Filter),
    Aggregate(Aggregate),
    Sort(Sort),
    Join(Join),
    Scan(Scan),
    Limit(Limit),
    Offset(Offset),

    CreateTable(CreateTable),
}

#[derive(Debug, Clone)]
pub struct CreateTable {
    pub table_name: String,
    pub columns: Vec<Column>,
}

impl CreateTable {
    pub fn new_create_table(table_name: String, columns: Vec<Column>) -> Self {
        Self {
            table_name,
            columns,
        }
    }
}
