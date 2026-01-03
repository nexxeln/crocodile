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

    #[error("Cache error: {message}")]
    Cache { message: String },

    #[error("Tmux error: {message}")]
    Tmux { message: String },

    #[error("Entity not found: {entity_type} with id '{id}'")]
    NotFound { entity_type: String, id: String },

    #[error("Invalid role: {role}")]
    InvalidRole { role: String },

    #[error("Missing environment variable: {name}")]
    MissingEnvVar { name: String },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("SQLite error: {0}")]
    Sqlite(#[from] sqlx::Error),
}

pub type Result<T> = std::result::Result<T, CrocError>;
