use crate::foundationdb;
use crate::foundationdb::FdbBindingError;
use toolbox::backend::errors::BackendError;

pub type Result<T> = std::result::Result<T, CabinetLibError>;

#[derive(Debug, thiserror::Error)]
pub enum CabinetLibError {
    #[error("FoundationDB error: {0}")]
    FdbBinddingError(#[from] FdbBindingError),
    #[error("FDB error: {0}")]
    FdbError(#[from] foundationdb::FdbError),
    #[error(transparent)]
    Backend(#[from] BackendError),
}

impl From<CabinetLibError> for FdbBindingError {
    fn from(e: CabinetLibError) -> Self {
        match e {
            CabinetLibError::FdbBinddingError(e) => e,
            CabinetLibError::FdbError(e) => FdbBindingError::NonRetryableFdbError(e),
            CabinetLibError::Backend(err) => err.into(),
        }
    }
}
