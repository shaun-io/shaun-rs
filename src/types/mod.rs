pub mod expr;
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

pub fn is_can_cast(from: &LogicalType, to: &LogicalType) -> bool {
    if from == to {
        return true;
    }

    match from {
        LogicalType::Null => true,
        LogicalType::Bool => false,
        LogicalType::Int8 => matches!(
            to,
            LogicalType::Int16
                | LogicalType::Int32
                | LogicalType::Int64
                | LogicalType::UInt8
                | LogicalType::UInt16
                | LogicalType::UInt32
                | LogicalType::UInt64
                | LogicalType::Float32
                | LogicalType::Float64
        ),
        LogicalType::Int16 => matches!(
            to,
            LogicalType::Int32
                | LogicalType::Int64
                | LogicalType::UInt8
                | LogicalType::UInt16
                | LogicalType::UInt32
                | LogicalType::UInt64
                | LogicalType::Float32
                | LogicalType::Float64
        ),
        LogicalType::Int32 => matches!(
            to,
            LogicalType::Int8
                | LogicalType::Int16
                | LogicalType::Int32
                | LogicalType::Int64
                | LogicalType::UInt8
                | LogicalType::UInt16
                | LogicalType::UInt32
                | LogicalType::UInt64
                | LogicalType::Float32
                | LogicalType::Float64
        ),
        LogicalType::Int64 => matches!(
            to,
            LogicalType::Int8
                | LogicalType::Int16
                | LogicalType::Int32
                | LogicalType::UInt8
                | LogicalType::UInt16
                | LogicalType::UInt32
                | LogicalType::UInt64
                | LogicalType::Float32
                | LogicalType::Float64
        ),
        LogicalType::UInt8 => matches!(
            to,
            LogicalType::Int8
                | LogicalType::Int16
                | LogicalType::Int32
                | LogicalType::Int64
                | LogicalType::UInt16
                | LogicalType::UInt32
                | LogicalType::UInt64
                | LogicalType::Float32
                | LogicalType::Float64
        ),
        LogicalType::UInt16 => matches!(
            to,
            LogicalType::Int8
                | LogicalType::Int16
                | LogicalType::Int32
                | LogicalType::Int64
                | LogicalType::UInt8
                | LogicalType::UInt32
                | LogicalType::UInt64
                | LogicalType::Float32
                | LogicalType::Float64
        ),
        LogicalType::UInt32 => matches!(
            to,
            LogicalType::Int8
                | LogicalType::Int16
                | LogicalType::Int32
                | LogicalType::Int64
                | LogicalType::UInt8
                | LogicalType::UInt32
                | LogicalType::UInt64
                | LogicalType::Float32
                | LogicalType::Float64
        ),
        LogicalType::UInt64 => matches!(
            to,
            LogicalType::Int8
                | LogicalType::Int16
                | LogicalType::Int32
                | LogicalType::Int64
                | LogicalType::UInt8
                | LogicalType::UInt16
                | LogicalType::UInt32
                | LogicalType::Float32
                | LogicalType::Float64
        ),
        LogicalType::Float32 => matches!(to, LogicalType::Float64),
        LogicalType::Float64 => false,
        LogicalType::VarChar(_) => matches!(to, LogicalType::String),
        LogicalType::String => matches!(to, LogicalType::VarChar(_)),
    }
}
