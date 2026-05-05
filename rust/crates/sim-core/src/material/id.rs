//! `MaterialId` — const djb2 hash newtype around `u32`.
//!
//! Identical bytes produce identical ids; the algorithm is byte-based djb2
//! (`hash * 33 + byte`, seeded at 5381). djb2 collisions are possible in
//! principle but the registry treats any collision as `DuplicateId`.

use serde::{Deserialize, Serialize};

/// 32-bit content-derived id for materials. Stable across runs given the
/// same input string (the algorithm is `const fn`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MaterialId(u32);

impl MaterialId {
    /// Const djb2 hash of the input string. Identical bytes → identical id.
    pub const fn from_str_hash(s: &str) -> Self {
        let mut hash: u32 = 5381;
        let bytes = s.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            hash = hash.wrapping_mul(33).wrapping_add(bytes[i] as u32);
            i += 1;
        }
        Self(hash)
    }

    /// Raw underlying `u32`.
    pub const fn raw(self) -> u32 {
        self.0
    }
}

impl std::fmt::Display for MaterialId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MaterialId({:#010x})", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn djb2_seed_for_empty_string() {
        // djb2 seed by definition.
        assert_eq!(MaterialId::from_str_hash("").raw(), 5381);
    }

    #[test]
    fn djb2_deterministic_across_calls() {
        let a = MaterialId::from_str_hash("iron").raw();
        let b = MaterialId::from_str_hash("iron").raw();
        assert_eq!(a, b);
    }

    #[test]
    fn distinct_inputs_distinct_ids_for_known_set() {
        let stone = MaterialId::from_str_hash("stone").raw();
        let iron = MaterialId::from_str_hash("iron").raw();
        let oak = MaterialId::from_str_hash("oak").raw();
        assert_ne!(stone, iron);
        assert_ne!(stone, oak);
        assert_ne!(iron, oak);
    }

    #[test]
    fn display_format_is_hex_padded() {
        let id = MaterialId::from_str_hash("");
        // 5381 = 0x1505
        assert_eq!(format!("{id}"), "MaterialId(0x00001505)");
    }
}
