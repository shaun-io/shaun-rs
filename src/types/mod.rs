pub mod value;

#[derive(Debug, Clone, PartialEq)]
pub enum LogicalType {
    Null,
    Bool,
    Int8,
    Int16,
    Int32,
    Int64,
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    Float32,
    Float64,
    String,
    VarChar(usize),
}
