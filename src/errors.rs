use thiserror::Error;

#[derive(Error, Debug)]
pub enum MyceliumError {
    #[error("SystemTimeError: {0}")]
    SystemTimeError(#[from] std::time::SystemTimeError),
    #[error("SignatureError")]
    SignatureError(String),
    #[error("SporeError: {0}")]
    SporeError(#[from] SporeError),
}

#[derive(Error, Debug)]
pub enum SporeError {
    #[error("Failed to sign data: {0}")]
    SignError(String),
    #[error("Failed to verify signature: {0}")]
    VerifyError(String),
    #[error("Failed to resolve Spore: {0}")]
    ResolveError(String),
}