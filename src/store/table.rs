use super::rocks::{Rocks, RocksRef};
use crate::{
    catalog::schema::SchemaRef,
    error::{Error, Result},
    types::value::{ScalarValue, Value},
};

#[derive(Debug)]
pub struct Table {
    pub name: String,
    pub db: RocksRef,
    pub schema: SchemaRef,
}

// impl Table {
//     pub fn next_tuple(&self) -> Option<Vec<ScalarValue>> {
        
//     }

//     pub fn insert_tuple() -> Result<()> {
//         unimplemented!()
//     }
// }
