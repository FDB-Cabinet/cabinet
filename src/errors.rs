use foundationdb::FdbBindingError;

pub type Result<T> = std::result::Result<T, CabinetError>;

#[derive(Debug, thiserror::Error)]
pub enum CabinetError {
    #[error("FoundationDB error: {0}")]
    FdbBinddingError(#[from] FdbBindingError),
    #[error("FDB error: {0}")]
    FdbError(#[from] foundationdb::FdbError),
    #[error("Item not found: {0}")]
    ItemNotFound(String),
    #[error("Invalid count stats value: Unable to decode from little endian bytes")]
    InvalidCountStatsValue,
}

impl From<CabinetError> for FdbBindingError {
    fn from(e: CabinetError) -> Self {
        match e {
            CabinetError::FdbBinddingError(e) => e,
            CabinetError::FdbError(e) => FdbBindingError::NonRetryableFdbError(e),
            err => FdbBindingError::CustomError(Box::new(err)),
        }
    }
}
