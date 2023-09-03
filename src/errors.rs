use serde_derive::{Deserialize, Serialize};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Serialize, Deserialize)]
pub enum Error {
    Abort,
    Config(String),
    Internal(String),
    ReadOnly,
    Value(String),
}
