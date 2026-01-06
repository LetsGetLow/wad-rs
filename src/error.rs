use thiserror::Error;

#[derive(Debug, Error)]
pub enum WADError {
    #[error("General error: {0}")]
    GeneralError(#[from] Box<dyn std::error::Error>),
    #[error("Invalid WAD header identification")]
    InvalidHeaderIdentification,
    #[error("Data too small to contain valid WAD header")]
    HeaderDataTooSmall,
}