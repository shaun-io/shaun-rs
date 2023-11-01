use core::fmt::Display;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    Parse(String),
    Storage(String),
    Internal(String),
    Conf(String),
    Other(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Parse(err)
            | Self::Storage(err)
            | Self::Internal(err)
            | Self::Conf(err)
            | Self::Other(err) => {
                write!(f, "{}", err)
            }
        }
    }
}

#[macro_export]
macro_rules! fmt_err {
    ($($arg:tt)*) => {
        format!("{}:{} {}", file!(), line!(), format!($($arg)*))
    };
}
