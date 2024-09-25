use thiserror::Error;

pub type Result<T> = core::result::Result<T, LWGeomError>;

#[derive(Debug, Error)]
pub enum LWGeomError {
    #[error("failed to new CString")]
    CStringError(#[from] std::ffi::NulError),
    #[error("the ptr should not be null")]
    NullPtrError,
    #[error("function `{0}` parameter `{1}` is invalid")]
    InvalidParameterError(String, String),
    #[error("failed to calculate function `{0}` result")]
    CalculateError(String),
    #[error("failed to parse WKT: {0}")]
    WKTParseError(String),
    #[error("failed to call function `{0}`, but no error message returned")]
    FailedWithoutMessageError(String),
}
