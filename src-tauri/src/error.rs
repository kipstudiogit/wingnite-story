use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Database failure: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("Concurrency lock failure: {0}")]
    Concurrency(String),
    #[error("Filesystem IO failure: {0}")]
    Io(#[from] std::io::Error),
    #[error("Storage path resolution failure: {0}")]
    Path(String),
    #[error("Entity not found: {0}")]
    NotFound(String),
    #[error("Security or integrity violation: {0}")]
    Integrity(String),
    #[error("Streaming transmission failure: {0}")]
    Streaming(String),
    #[error("Rope session error: {0}")]
    Rope(String),
    #[error("Cryptographic vault error: {0}")]
    Vault(String),
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}