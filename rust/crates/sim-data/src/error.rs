use thiserror::Error;

#[derive(Debug, Error)]
pub enum DataError {
    #[error("IO error reading {path}: {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("JSON parse error in {path}: {source}")]
    Json {
        path: String,
        #[source]
        source: serde_json::Error,
    },
    #[error("Missing required field '{field}' in {path}")]
    MissingField { field: String, path: String },
    #[error("Invalid value for field '{field}' in {path}: {reason}")]
    InvalidField {
        field: String,
        path: String,
        reason: String,
    },
}

pub type DataResult<T> = Result<T, DataError>;
