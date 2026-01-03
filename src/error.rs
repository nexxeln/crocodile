use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CrocError {
    #[error("Project already initialized at {path}")]
    AlreadyInitialized { path: PathBuf },

    #[error("Not a git repository: {path}")]
    NotGitRepo { path: PathBuf },

    #[error("Path not found: {path}")]
    PathNotFound { path: PathBuf },

    #[error("Invalid configuration: {reason}")]
    InvalidConfig { reason: String },

    #[error("Storage error: {message}")]
    Storage { message: String },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, CrocError>;
