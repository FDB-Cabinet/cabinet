use cabinet_lib::foundationdb;
use cabinet_lib::foundationdb::FdbBindingError;
use cabinet_protocol::ParseError;
use std::io;

#[derive(Debug, thiserror::Error)]
pub enum CabinetError {
    #[error("FoundationDB error: {0}")]
    FdbBinddingError(#[from] FdbBindingError),
    #[error("FDB error: {0}")]
    FdbError(#[from] foundationdb::FdbError),
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),
    #[error("Command parse error: {0}")]
    CommandParse(#[from] ParseError),
    /// Unable to decode a string as UTF-8
    #[error("UTF-8 error: {0}")]
    Utf8Error(#[from] std::str::Utf8Error),
}

impl From<CabinetError> for FdbBindingError {
    fn from(e: CabinetError) -> Self {
        match e {
            CabinetError::FdbBinddingError(e) => e,
            CabinetError::FdbError(e) => FdbBindingError::NonRetryableFdbError(e),
            err => err.into(),
        }
    }
}
