use foundationdb::FdbBindingError;

#[derive(Debug, thiserror::Error)]
pub enum StatsError {
    #[error("FoundationDB error: {0}")]
    FdbBinddingError(#[from] foundationdb::FdbBindingError),
    #[error("FDB error: {0}")]
    FdbError(#[from] foundationdb::FdbError),
    #[error("Item not found")]
    ItemNotFound,
    #[error("Item value incorrect :  expected {:?}, actual {:?}", String::from_utf8_lossy(&expected) ,String::from_utf8_lossy(&actual))]
    ItemValueIncorrect { expected: Vec<u8>, actual: Vec<u8> },
}

impl From<StatsError> for FdbBindingError {
    fn from(value: StatsError) -> Self {
        FdbBindingError::CustomError(Box::new(value))
    }
}
