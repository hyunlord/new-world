//! Serde helpers for save files (M1.5 §7): hash collections written in
//! sorted key order.
//!
//! A std `HashMap`/`HashSet` iterates in an order that differs between
//! instances and processes, so serializing one directly would make two saves
//! of the same state differ byte for byte. Fields of those types carry
//! `#[serde(serialize_with = "crate::save_serde::sorted_map")]` (or
//! `sorted_set`): the entries are written in ascending key order, and the
//! default `Deserialize` reads them back into a hash collection. The content
//! is the same either way; only the written order is fixed.

use serde::ser::{SerializeMap, SerializeSeq};
use serde::{Serialize, Serializer};
use std::collections::{HashMap, HashSet};
use std::hash::BuildHasher;

/// Serialize a `HashMap` as a map whose entries are in ascending key order.
pub fn sorted_map<K, V, H, S>(map: &HashMap<K, V, H>, serializer: S) -> Result<S::Ok, S::Error>
where
    K: Serialize + Ord,
    V: Serialize,
    H: BuildHasher,
    S: Serializer,
{
    let mut entries: Vec<(&K, &V)> = map.iter().collect();
    entries.sort_by(|a, b| a.0.cmp(b.0));
    let mut out = serializer.serialize_map(Some(entries.len()))?;
    for (key, value) in entries {
        out.serialize_entry(key, value)?;
    }
    out.end()
}

/// Serialize a `HashSet` as a sequence in ascending order.
pub fn sorted_set<T, H, S>(set: &HashSet<T, H>, serializer: S) -> Result<S::Ok, S::Error>
where
    T: Serialize + Ord,
    H: BuildHasher,
    S: Serializer,
{
    let mut items: Vec<&T> = set.iter().collect();
    items.sort();
    let mut out = serializer.serialize_seq(Some(items.len()))?;
    for item in items {
        out.serialize_element(item)?;
    }
    out.end()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Serialize)]
    struct Holder {
        #[serde(serialize_with = "sorted_map")]
        map: HashMap<u32, u8>,
        #[serde(serialize_with = "sorted_set")]
        set: HashSet<u32>,
    }

    #[test]
    fn hash_collections_are_written_in_key_order() {
        // Two holders with the same content built in opposite insertion
        // orders (and distinct hash seeds) write the same JSON, keys ascending.
        let a = Holder {
            map: (0..64).map(|k| (k, k as u8)).collect(),
            set: (0..64).collect(),
        };
        let b = Holder {
            map: (0..64).rev().map(|k| (k, k as u8)).collect(),
            set: (0..64).rev().collect(),
        };
        let ja = serde_json::to_string(&a).unwrap_or_default();
        let jb = serde_json::to_string(&b).unwrap_or_default();
        assert_eq!(ja, jb);
        assert!(ja.starts_with("{\"map\":{\"0\":0,\"1\":1,\"2\":2"), "{ja}");
        assert!(ja.contains("\"set\":[0,1,2,3,"), "{ja}");
    }
}
