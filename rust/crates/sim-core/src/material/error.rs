//! `MaterialError` — exactly 6 variants, in §3.D order.

use crate::material::id::MaterialId;

/// Errors emitted by the material loader, validator, and registry.
///
/// Variant order is locked by §3.D of the material schema spec.
#[derive(Debug, thiserror::Error)]
pub enum MaterialError {
    /// RON parser rejected the file (syntax / type mismatch).
    #[error("RON parse error: {0}")]
    ParseError(String),

    /// Underlying I/O failure (file not found, permission denied, …).
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    /// Loader saw a `schema_version` it does not support.
    #[error("schema mismatch: file declares version {file_version}, supported = {supported}")]
    SchemaMismatch {
        /// Version declared inside the offending file.
        file_version: u32,
        /// Version the loader was built against.
        supported: u32,
    },

    /// `MaterialProperties::validate` rejected an out-of-range numeric field.
    #[error("property `{property}` value {value} out of range; expected {expected}")]
    PropertyOutOfRange {
        /// Snake-case field name.
        property: &'static str,
        /// Supplied value.
        value: f64,
        /// Human-readable expected-range description.
        expected: &'static str,
    },

    /// Registry refused to register a second def with the same id.
    #[error("duplicate material id: {0}")]
    DuplicateId(MaterialId),

    /// Loader encountered a terrain string the schema does not know.
    #[error("unknown terrain type: {0}")]
    UnknownTerrainType(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn implements_std_error_send_sync_static() {
        fn assert_error<T: std::error::Error + std::fmt::Display + std::fmt::Debug + Send + Sync + 'static>() {}
        assert_error::<MaterialError>();
    }

    #[test]
    fn io_variant_constructs_from_std_io_error() {
        let e: MaterialError = std::io::Error::other("x").into();
        assert!(matches!(e, MaterialError::Io(_)));
    }

    #[test]
    fn six_variants_cover_match() {
        let cases = [
            MaterialError::ParseError("p".into()),
            MaterialError::Io(std::io::Error::other("x")),
            MaterialError::SchemaMismatch { file_version: 2, supported: 1 },
            MaterialError::PropertyOutOfRange {
                property: "density",
                value: 0.0,
                expected: "100..=25000",
            },
            MaterialError::DuplicateId(MaterialId::from_str_hash("x")),
            MaterialError::UnknownTerrainType("wetland".into()),
        ];
        for e in cases {
            match e {
                // Tuple variant: explicit String type assertion.
                MaterialError::ParseError(s) => {
                    let _: String = s;
                }
                // Tuple variant: explicit std::io::Error type assertion.
                MaterialError::Io(e) => {
                    let _: std::io::Error = e;
                }
                // Named-field variant: lock both field names and types.
                MaterialError::SchemaMismatch {
                    file_version,
                    supported,
                } => {
                    let _: u32 = file_version;
                    let _: u32 = supported;
                }
                // Named-field variant: lock all 3 fields' names and types.
                MaterialError::PropertyOutOfRange {
                    property,
                    value,
                    expected,
                } => {
                    let _: &'static str = property;
                    let _: f64 = value;
                    let _: &'static str = expected;
                }
                // Tuple variant: explicit MaterialId type assertion.
                MaterialError::DuplicateId(id) => {
                    let _: MaterialId = id;
                }
                // Tuple variant: explicit String type assertion.
                MaterialError::UnknownTerrainType(s) => {
                    let _: String = s;
                }
            }
        }
    }
}
