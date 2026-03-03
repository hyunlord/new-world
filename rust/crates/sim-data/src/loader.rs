use std::path::Path;
use serde::de::DeserializeOwned;
use crate::error::{DataError, DataResult};

/// Load and deserialize a single JSON file.
pub fn load_json<T: DeserializeOwned>(path: &Path) -> DataResult<T> {
    let content = std::fs::read_to_string(path).map_err(|e| DataError::Io {
        path: path.display().to_string(),
        source: e,
    })?;
    serde_json::from_str(&content).map_err(|e| DataError::Json {
        path: path.display().to_string(),
        source: e,
    })
}

/// Discover all .json files in a directory (non-recursive).
pub fn list_json_files(dir: &Path) -> DataResult<Vec<std::path::PathBuf>> {
    let mut files = Vec::new();
    let entries = std::fs::read_dir(dir).map_err(|e| DataError::Io {
        path: dir.display().to_string(),
        source: e,
    })?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "json") {
            files.push(path);
        }
    }
    files.sort();
    Ok(files)
}

/// Discover all .json files in a directory tree (recursive, 1 level deep sub-dirs).
pub fn list_json_files_recursive(dir: &Path) -> DataResult<Vec<std::path::PathBuf>> {
    let mut files = Vec::new();
    for entry in std::fs::read_dir(dir)
        .map_err(|e| DataError::Io {
            path: dir.display().to_string(),
            source: e,
        })?
        .flatten()
    {
        let path = entry.path();
        if path.is_dir() {
            files.extend(list_json_files(&path)?);
        } else if path.extension().is_some_and(|e| e == "json") {
            files.push(path);
        }
    }
    files.sort();
    Ok(files)
}
