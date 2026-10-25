//! Strict external observer metadata; SHA256 is supplied and verified by the runner/scorer.
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum Arm {
    B0,
    BP,
    #[serde(rename = "BPT")]
    Bpt,
    #[serde(rename = "BPTH")]
    Bpth,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Policy {
    #[serde(rename = "old_3_1")]
    Old3_1,
    #[serde(rename = "scarce_30_10_1")]
    Scarce30_10_1,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub schema_version: u32,
    pub arm: Arm,
    pub arithmetic_manifest_sha256: String,
    pub catalog_revision: String,
    pub policy: Policy,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub manifest: Manifest,
    pub manifest_sha256: String,
}

pub fn valid_hash(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

impl Config {
    pub fn load(path: &Path, supplied_sha256: &str) -> Result<Self, String> {
        let text = std::fs::read_to_string(path).map_err(|e| format!("B4 manifest: {e}"))?;
        let manifest: Manifest =
            serde_json::from_str(&text).map_err(|e| format!("B4 manifest: {e}"))?;
        if manifest.schema_version != 1
            || !valid_hash(supplied_sha256)
            || !valid_hash(&manifest.arithmetic_manifest_sha256)
            || manifest.catalog_revision.is_empty()
        {
            return Err(
                "B4 requires schema1, catalog revision and lowercase SHA256 digests".into(),
            );
        }
        let expected = match manifest.arm {
            Arm::B0 | Arm::BP => Policy::Old3_1,
            Arm::Bpt | Arm::Bpth => Policy::Scarce30_10_1,
        };
        if manifest.policy != expected {
            return Err("B4 arm/policy mismatch".into());
        }
        Ok(Self {
            manifest,
            manifest_sha256: supplied_sha256.into(),
        })
    }
}
