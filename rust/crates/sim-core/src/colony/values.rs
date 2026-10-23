//! M5.5 (ADR 0032): what items are worth to a trader — the value table `data/colony/values.ron`.
use serde::Deserialize;
use std::collections::BTreeMap;
use std::sync::OnceLock;

use super::ItemKind;

const VALUES_RON: &str = include_str!("../../data/colony/values.ron");

#[derive(Deserialize)]
struct ValuesFile {
    #[allow(dead_code)]
    schema_version: u32,
    values: Vec<(String, u32)>,
}

/// Parse a value table; `None` when the text is malformed or names an unknown item.
pub fn parse_values(text: &str) -> Option<BTreeMap<ItemKind, u32>> {
    let file: ValuesFile = ron::from_str(text).ok()?;
    let mut out = BTreeMap::new();
    for (name, value) in file.values {
        out.insert(ItemKind::from_name(&name)?, value);
    }
    Some(out)
}

fn table() -> &'static BTreeMap<ItemKind, u32> {
    static TABLE: OnceLock<BTreeMap<ItemKind, u32>> = OnceLock::new();
    TABLE.get_or_init(|| parse_values(VALUES_RON).unwrap_or_default())
}

/// The base value of `kind` (0 when the table lacks it; the unit test requires every kind).
pub fn base_value(kind: ItemKind) -> u32 {
    table().get(&kind).copied().unwrap_or(0)
}

/// The value of one item of `kind` at `quality` (goal §3.3): base × (500 + 5 × quality) / 1,000, rounded, at least 1.
pub fn value(kind: ItemKind, quality: u8) -> u32 {
    let raw = u64::from(base_value(kind)) * (500 + 5 * u64::from(quality));
    ((raw + 500) / 1000).max(1) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_item_has_a_value_and_quality_scales_it() {
        for kind in ItemKind::ALL {
            assert!(base_value(kind) > 0, "{kind:?} has no value");
        }
        // A flint blade (6) at quality 50: 6 × 750 / 1,000 = 4.5 → 5; at 100: 6; at 0: 3.
        assert_eq!(value(ItemKind::FlintBlade, 50), 5);
        assert_eq!(value(ItemKind::FlintBlade, 100), 6);
        assert_eq!(value(ItemKind::FlintBlade, 0), 3);
        // A fur cloak (30) at quality 70: 30 × 850 / 1,000 = 25.5 → 26.
        assert_eq!(value(ItemKind::FurCloak, 70), 26);
        // Never below 1 (firewood 1 at quality 0: 0.5 → 1).
        assert_eq!(value(ItemKind::Firewood, 0), 1);
        assert!(parse_values("(schema_version: 1, values: [(\"no_such_item\", 3)])").is_none());
    }
}
