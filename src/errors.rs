use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Insufficient balance")]
    InsufficientBalance,

    #[error("Database error")]
    DatabaseError(#[from] diesel::result::Error),
}
