//! Per-world value selectors; parsed once, resolved against each world rule.
/// Validated RuleU32 metadata.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuleU32 {
    /// Value.
    pub value: u32,
    /// Rule.
    pub rule: Option<String>,
}
/// Validated RuleU16 metadata.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuleU16 {
    /// Value.
    pub value: u16,
    /// Rule.
    pub rule: Option<String>,
}
/// Validated RuleBool metadata.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuleBool {
    /// Value.
    pub value: bool,
    /// Rule.
    pub rule: Option<String>,
}
/// Validated RuleSpan metadata.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuleSpan {
    /// Value.
    pub value: (u32, u32),
    /// Rule.
    pub rule: Option<String>,
}
