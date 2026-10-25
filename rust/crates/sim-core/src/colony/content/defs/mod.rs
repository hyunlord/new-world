//! Common immutable identity metadata.
use serde::Deserialize;

/// Validated IdentityDef metadata.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IdentityDef {
    /// Code.
    pub code: u32,
    /// Id.
    pub id: String,
    /// Name key.
    pub name_key: String,
    /// Serde name.
    pub serde_name: String,
    /// Serde aliases.
    pub serde_aliases: Vec<String>,
    /// Resolver aliases.
    pub resolver_aliases: Vec<String>,
}

mod blueprint;
mod item;
mod recipe;
pub use blueprint::*;
pub use item::*;
pub use recipe::*;
