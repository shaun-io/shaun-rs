#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    All,
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
}




#[derive(Debug, Clone)]
pub enum ScalarValue {
    Null,
    Bool(Option<bool>),
    Int8(Option<i8>),
    Int16(Option<i16>),
    Int32(Option<i32>),
    Int64(Option<i64>),
    UInt8(Option<u8>),
    UInt16(Option<u16>),
    UInt32(Option<u32>),
    UInt64(Option<u64>),
    Float32(Option<f32>),
    Float64(Option<f64>),
    String(String),
}
