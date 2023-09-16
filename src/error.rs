use core::fmt::Display;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    ParseErr(String),
    StorageErr(String),
    InternalErr(String),
    ConfErr(String),
    OtherErr(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ParseErr(err)
            | Self::StorageErr(err)
            | Self::InternalErr(err)
            | Self::ConfErr(err)
            | Self::OtherErr(err) => {
                write!(f, "{}", err)
            }
        }
    }
}
