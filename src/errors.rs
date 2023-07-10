use thiserror::Error;

#[derive(Error, Debug)]
pub enum MyceliumError {
    #[error("SystemTimeError: {0}")]
    SystemTimeError(#[from] std::time::SystemTimeError),
}