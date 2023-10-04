#[derive(Eq, PartialEq, Debug, Clone)]
pub enum DataType {
    Char,
    Bool,
    Int8,
    Int16,
    Int32,
    Int64,
    Uint8,
    Uint16,
    Uint32,
    Uint64,
    Float32,
    Float64,
    Varchar(usize),
    String,
}
