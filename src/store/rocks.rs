use std::{path::Iter, sync::Arc};

use rocksdb::{
    DBAccess, DBCommon, DBIteratorWithThreadMode, IteratorMode, MultiThreaded, SingleThreaded,
    ThreadMode, DB,
};

use crate::{
    error::{Error, Error::Store, Result},
    fmt_err,
};

#[derive(Debug)]
pub struct Rocks {
    pub path: String,
    pub handle: DB,
}

type RocksIter<'a> = DBIteratorWithThreadMode<'a, DB>;

pub type RocksRef = Arc<Rocks>;

impl Rocks {
    pub(crate) fn new(path: &str) -> Result<Self> {
        Ok(Self {
            path: path.to_string(),
            handle: match DB::open_default(path) {
                Ok(db) => db,
                Err(e) => {
                    return Err(Store(fmt_err!("{e} init rocksdb fail")));
                }
            },
        })
    }

    pub(crate) fn put(
        &mut self,
        key: &[u8],
        value: &[u8],
    ) -> std::result::Result<(), rocksdb::Error> {
        self.handle.put(key, value)
    }

    pub(crate) fn get<K: AsRef<[u8]>>(
        &mut self,
        key: K,
    ) -> std::result::Result<Option<Vec<u8>>, rocksdb::Error> {
        self.handle.get(key)
    }

    pub(crate) fn iter<'a>(&self, mode: IteratorMode<'a>) -> RocksIter {
        self.handle.iterator(mode)
    }

    // pub(crate) fn prefix_iter(&self, prefix: &str) -> RocksPrefixIter {
    //     let pp = self.handle.prefix_iterator(prefix);
    // }
}

#[cfg(test)]
mod tests {

    use super::*;

    fn type_of<T>(_: T) -> &'static str {
        std::any::type_name::<T>()
    }

    #[test]
    pub fn test() {
        // DBCommon 实现了 Drop
        // let mut db = Rocks::new("./rocksdata").unwrap();
        // db.put(b"123", b"dada").unwrap();
        // db.put(b"123dakld", b"dakdlakldjakl").unwrap();
        // db.put(b"bbb", b"daldald").unwrap();
        // let data = db.get("123").unwrap().unwrap();
        // assert_eq!(data, Vec::from("dada"));

        // let _ = db
        //     .iter(IteratorMode::Start)
        //     .map(|item| match item {
        //         Ok((k, v)) => {
        //             println!(
        //                 "{} {}",
        //                 std::str::from_utf8(&*k).unwrap(),
        //                 std::str::from_utf8(&*v).unwrap()
        //             );
        //         }
        //         Err(err) => {
        //             println!("{err}");
        //         }
        //     })
        //     .for_each(drop);
        // println!("prefix_iter");
        // const a: &[u8] = b"da";
        // let pp = db.handle.prefix_iterator("12");
        // let mut got = pp.collect::<Result<Vec<_>, >>().unwrap();
        // got.reverse();
        // assert_eq!(got.as_slice(), &[Box::new(a), Box::new(a)]);
    }
}
