use toolbox;
use toolbox::foundationdb::FdbBindingError;

#[derive(Debug, thiserror::Error)]
pub enum StatsError {
    #[error("FoundationDB error: {0}")]
    FdbBinddingError(#[from] FdbBindingError),
    #[error("FDB error: {0}")]
    FdbError(#[from] toolbox::foundationdb::FdbError),
    #[error("Item not found")]
    ItemNotFound,
    #[error("Item value incorrect :  expected {:?}, actual {:?}", String::from_utf8_lossy(&expected) ,String::from_utf8_lossy(&actual))]
    ItemValueIncorrect { expected: Vec<u8>, actual: Vec<u8> },
    #[error("Invalid database stats size: expected {expected} bytes, actual {actual} bytes")]
    InvalidDatabaseStatsSize { expected: i64, actual: i64 },
    #[error("Invalid database stats count: expected {expected}, actual {actual}")]
    InvalidDatabaseStatsCount { expected: i64, actual: i64 },
    #[error(transparent)]
    Cabinet(#[from] cabinet_lib::errors::CabinetLibError),
}

impl From<StatsError> for FdbBindingError {
    fn from(value: StatsError) -> Self {
        FdbBindingError::CustomError(Box::new(value))
    }
}
