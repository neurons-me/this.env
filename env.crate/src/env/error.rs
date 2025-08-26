// src/env/error.rs (por ejemplo)
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EnvError {
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}