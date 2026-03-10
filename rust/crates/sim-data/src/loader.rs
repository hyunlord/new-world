use std::fmt;
use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;

use crate::error::{DataError, DataResult};

/// Rich load error for RON parsing and validation bootstrap.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLoadError {
    /// File that failed to load or validate.
    pub file: PathBuf,
    /// Best-effort 1-based line number, when available.
    pub line: Option<usize>,
    /// Human-readable error detail.
    pub message: String,
}

impl fmt::Display for DataLoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.line {
            Some(line) => write!(f, "{}:{}: {}", self.file.display(), line, self.message),
            None => write!(f, "{}: {}", self.file.display(), self.message),
        }
    }
}

impl std::error::Error for DataLoadError {}

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

/// Load and deserialize a single RON file with field-path-aware errors.
pub fn load_ron_file<T: DeserializeOwned>(path: &Path) -> Result<T, DataLoadError> {
    let content = std::fs::read_to_string(path).map_err(|source| DataLoadError {
        file: path.to_path_buf(),
        line: None,
        message: format!("file read error: {source}"),
    })?;

    let mut deserializer =
        ron::de::Deserializer::from_str(&content).map_err(|source| DataLoadError {
            file: path.to_path_buf(),
            line: Some(source.position.line),
            message: format!("RON parse error: {source}"),
        })?;

    serde_path_to_error::deserialize(&mut deserializer).map_err(|error| DataLoadError {
        file: path.to_path_buf(),
        line: None,
        message: format!(
            "deserialization error at {}: {}",
            error.path(),
            error.inner()
        ),
    })
}

/// Discover all `.ron` files in a directory (non-recursive).
pub fn list_ron_files(dir: &Path) -> Result<Vec<PathBuf>, DataLoadError> {
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut files = Vec::new();
    let entries = std::fs::read_dir(dir).map_err(|source| DataLoadError {
        file: dir.to_path_buf(),
        line: None,
        message: format!("directory read error: {source}"),
    })?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "ron") {
            files.push(path);
        }
    }
    files.sort();
    Ok(files)
}

/// Load every `.ron` file in a directory, flattening per-file `Vec<T>` payloads.
pub fn load_ron_directory<T: DeserializeOwned>(dir: &Path) -> Result<Vec<T>, Vec<DataLoadError>> {
    let files = match list_ron_files(dir) {
        Ok(files) => files,
        Err(error) => return Err(vec![error]),
    };

    let mut values = Vec::new();
    let mut errors = Vec::new();
    for file in files {
        match load_ron_file::<Vec<T>>(&file) {
            Ok(mut defs) => values.append(&mut defs),
            Err(error) => errors.push(error),
        }
    }

    if errors.is_empty() {
        Ok(values)
    } else {
        Err(errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[derive(Debug, Deserialize, PartialEq, Eq)]
    #[serde(deny_unknown_fields)]
    struct LoaderExample {
        id: String,
    }

    struct TempDirGuard {
        path: PathBuf,
    }

    impl TempDirGuard {
        fn new(name: &str) -> Self {
            let nonce = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock error")
                .as_nanos();
            let path = std::env::temp_dir().join(format!(
                "worldsim_loader_test_{}_{}_{}",
                name,
                std::process::id(),
                nonce
            ));
            fs::create_dir_all(&path).expect("failed to create temp dir");
            Self { path }
        }
    }

    impl Drop for TempDirGuard {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    #[test]
    fn load_ron_directory_reads_sorted_files() {
        let temp = TempDirGuard::new("load_ron_directory");
        fs::write(temp.path.join("b.ron"), "[LoaderExample(id: \"second\")]")
            .expect("failed to write b.ron");
        fs::write(temp.path.join("a.ron"), "[LoaderExample(id: \"first\")]")
            .expect("failed to write a.ron");

        let defs: Vec<LoaderExample> =
            load_ron_directory(&temp.path).expect("expected RON directory to load");

        assert_eq!(
            defs,
            vec![
                LoaderExample {
                    id: "first".to_string()
                },
                LoaderExample {
                    id: "second".to_string()
                }
            ]
        );
    }

    #[test]
    fn load_ron_file_reports_parse_errors() {
        let temp = TempDirGuard::new("load_ron_file_error");
        let file = temp.path.join("broken.ron");
        fs::write(&file, "[LoaderExample(id: \"broken\")").expect("failed to write broken.ron");

        let error = load_ron_file::<Vec<LoaderExample>>(&file).expect_err("expected parse error");
        assert!(!error.message.is_empty());
        assert_eq!(error.file, file);
    }
}
