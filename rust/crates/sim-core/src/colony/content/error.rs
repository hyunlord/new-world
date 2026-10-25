//! Contextual catalogue validation and installation errors.
use thiserror::Error;

/// A structural or semantic content definition error.
#[derive(Debug, Clone, Error, PartialEq, Eq)]
#[error("content {context}: {message}")]
pub struct CatalogError {
    /// Source, family or entry being validated.
    pub context: String,
    /// Concrete validation failure.
    pub message: String,
}
impl CatalogError {
    pub(super) fn new(context: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            context: context.into(),
            message: message.into(),
        }
    }
}

/// A checked access never initializes or replaces the process catalogue.
#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum CatalogAccessError {
    /// Bootstrap must precede typed parsing.
    #[error("content catalogue is not installed")]
    NotInstalled,
    /// Explicit bootstrap previously failed.
    #[error("content bootstrap failed: {0}")]
    BootFailed(String),
    /// The installed family has no such code.
    #[error("unknown required {family} code {code}")]
    Unknown {
        /// Content family type name.
        family: &'static str,
        /// Unregistered wire code.
        code: u32,
    },
}

/// Sticky process bootstrap failure.
#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum BootError {
    /// Pure catalogue construction failed.
    #[error(transparent)]
    Invalid(#[from] CatalogError),
    /// A different selection already owns this process.
    #[error(
        "content catalogue already initialized with a different selection; start a fresh process"
    )]
    AlreadyInitialized,
}
