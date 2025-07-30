use crate::foundationdb;
use crate::foundationdb::FdbBindingError;
use toolbox::backend::errors::BackendError;

pub type Result<T> = std::result::Result<T, CabinetError>;

#[derive(Debug, thiserror::Error)]
pub enum CabinetError {
    #[error("FoundationDB error: {0}")]
    FdbBinddingError(#[from] FdbBindingError),
    #[error("FDB error: {0}")]
    FdbError(#[from] foundationdb::FdbError),
    #[error(transparent)]
    Backend(#[from] BackendError),
}

impl From<CabinetError> for FdbBindingError {
    fn from(e: CabinetError) -> Self {
        match e {
            CabinetError::FdbBinddingError(e) => e,
            CabinetError::FdbError(e) => FdbBindingError::NonRetryableFdbError(e),
            CabinetError::Backend(err) => err.into(),
        }
    }
}
