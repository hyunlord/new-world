//! M5.5 (ADR 0032): item data — what items are worth to a trader, the node that makes each and the nodes it hints at —
//! from `data/colony/values.ron` (owner directive 2026-10-24: content lives in data files, not in code).
use serde::Deserialize;
use std::collections::BTreeMap;

use super::ItemKind;

#[cfg(test)]
const VALUES_RON: &str = include_str!("../../data/colony/values.ron");

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ValuesFile {
    schema_version: u32,
    items: Vec<ItemEntry>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ItemEntry {
    name: String,
    value: u32,
    #[serde(default)]
    made_by: Option<String>,
    #[serde(default)]
    clue_nodes: Vec<String>,
    #[serde(default)]
    tags: Vec<String>,
}

/// One item's data (`values.ron`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ItemData {
    /// Base trade value.
    pub value: u32,
    /// The knowledge node whose practice makes the item (a caravan's "lacked" good), if any.
    pub made_by: Option<String>,
    /// The knowledge nodes the item hints at while no colonist practises them (eureka).
    pub clue_nodes: Vec<String>,
    /// Categories the code matches instead of item names (`carcass`).
    pub tags: Vec<String>,
}

/// Parse an item table; `None` when the text is malformed, of another schema, names an unknown item or field, or
/// leaves an item out.
pub fn parse_values(text: &str) -> Option<BTreeMap<ItemKind, ItemData>> {
    let file: ValuesFile = ron::from_str(text).ok()?;
    if file.schema_version != 2 {
        return None;
    }
    let mut out = BTreeMap::new();
    for e in file.items {
        if out
            .insert(
                ItemKind::from_name(&e.name)?,
                ItemData {
                    value: e.value,
                    made_by: e.made_by,
                    clue_nodes: e.clue_nodes,
                    tags: e.tags,
                },
            )
            .is_some()
        {
            return None;
        }
    }
    (out.len() == ItemKind::all().len()).then_some(out)
}

/// Whether the selected immutable item catalog loaded.
pub fn loaded() -> bool {
    super::content::installed_or_bundled().items().len() > 0
}

/// Installed item metadata; values and capabilities share one authoritative catalog.
pub fn item_data(kind: ItemKind) -> Option<&'static super::content::ItemDef> {
    super::content::installed_or_bundled().item(kind)
}

/// Whether `kind` carries category `tag` in `values.ron`.
pub fn has_tag(kind: ItemKind, tag: &str) -> bool {
    item_data(kind).is_some_and(|d| d.tags.iter().any(|t| t == tag))
}

/// The base value of `kind` (0 when the table lacks it; the unit test requires every kind).
pub fn base_value(kind: ItemKind) -> u32 {
    item_data(kind).map_or(0, |d| d.value)
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
    fn duplicate_value_rows_are_not_silently_overwritten() {
        let duplicate = VALUES_RON.replacen("items: [", "items: [(name: \"log\", value: 2),", 1);
        assert!(parse_values(&duplicate).is_none());
    }

    #[test]
    fn every_item_has_a_value_and_quality_scales_it() {
        for kind in ItemKind::all() {
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
        assert!(
            parse_values("(schema_version: 2, items: [(name: \"no_such_item\", value: 3)])")
                .is_none()
        );
        assert!(loaded());
        let v1 = VALUES_RON.replace("schema_version: 2", "schema_version: 1");
        assert!(parse_values(&v1).is_none(), "another schema");
        let typo = VALUES_RON.replacen("value: 2)", "valu: 2)", 1);
        assert!(parse_values(&typo).is_none(), "a misspelt field");
        let short = VALUES_RON.replacen("(name: \"log\", value: 2),", "", 1);
        assert!(parse_values(&short).is_none(), "an item left out");
        // The clue data (values.ron): a clay pot is made by pot firing and hints at the pottery chain; a log at nothing.
        let pot = item_data(ItemKind::ClayPot).expect("clay pot");
        assert_eq!(pot.made_by.as_deref(), Some("pot_firing"));
        assert_eq!(pot.clue_nodes, ["clay_prep", "pot_forming", "pot_firing"]);
        let log = item_data(ItemKind::Log).expect("log");
        assert!(log.made_by.is_none() && log.clue_nodes.is_empty());
        // What a kill leaves is tagged "carcass": meat, hide and a wolf's hide, nothing else.
        let carcass: Vec<ItemKind> = ItemKind::all().filter(|k| has_tag(*k, "carcass")).collect();
        assert_eq!(
            carcass,
            [ItemKind::Hide, ItemKind::Meat, ItemKind::WolfHide]
        );
    }
}
