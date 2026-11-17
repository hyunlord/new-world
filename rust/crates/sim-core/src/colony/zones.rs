//! Validated data rules for settlement layout; physical tiles remain in BuildingState.
use crate::colony::knowledge::ContentStatus;
use serde::Deserialize;
use std::collections::BTreeSet;

/// Existing placement behaviour served by a layout rule; zone identities remain data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum ZoneUse {
    /// Dwelling footprints, including player-authored housing zones.
    Housing,
    /// Stock and storehouse space.
    Storage,
    /// Workshop footprints.
    Workshop,
    /// Cultivated land.
    Field,
    /// Livestock enclosure footprints.
    Pen,
}

/// Placement relationship to the village.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum Adjacency {
    /// Adjacent to an existing zone, building or hearth, allowing the authored gap.
    NextToZoneOrHearth,
}

/// Things a new zone must leave outside its footprint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum ZoneBlocker {
    /// Standing trees.
    Trees,
    /// Quarriable resource nodes.
    Nodes,
    /// Cultivated fields.
    Fields,
    /// Built or planned structures.
    Structures,
    /// Ore deposits.
    Outcrops,
}

/// One zone layout rule. Content names are resolved against buildings.ron.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ZoneRule {
    /// Existing building zone kind id.
    pub kind: String,
    /// Placement behaviour, independent of the content id.
    pub category: ZoneUse,
    /// Rectangle dimensions.
    pub size: (u32, u32),
    /// Rectangle that must still fit before another zone is needed.
    pub slot_size: (u32, u32),
    /// Empty tiles between layout rectangles.
    pub min_spacing: u32,
    /// Maximum empty gap to an existing village tile.
    pub max_gap: u32,
    /// Adjacency constraint.
    pub adjacency: Adjacency,
    /// Required path stage beside later rectangles; each kind bootstraps without one.
    pub min_path_stage: u8,
    /// Positive settlement fact that requests this kind of space.
    pub trigger: String,
    /// Existing steward action family whose permission admits layout growth.
    #[serde(default)]
    pub scope: crate::colony::steward::StewardScope,
    /// Maximum zones is ceil(population / people_per_zone), at least one.
    pub people_per_zone: u32,
    /// Physical contents excluded from new layout rectangles.
    pub avoid: Vec<ZoneBlocker>,
    /// Runtime connection status.
    #[serde(default)]
    pub status: ContentStatus,
    /// Explanation for an inert rule.
    #[serde(default)]
    pub debt: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ZonesFile {
    schema_version: u32,
    zone_rules: Vec<ZoneRule>,
}

/// Immutable selected layout rules in authored order.
#[derive(Debug, Default)]
pub struct ZonesData {
    /// Validated base rules followed by extension rules.
    pub rules: Vec<ZoneRule>,
}

impl ZonesData {
    /// Parse and append extension rules without replacing existing identities.
    pub fn parse(base: &str, extension: Option<&str>) -> Result<Self, String> {
        let mut rules = Vec::new();
        let mut kinds = BTreeSet::new();
        for text in std::iter::once(base).chain(extension) {
            let file: ZonesFile = ron::from_str(text).map_err(|e| format!("zones.ron: {e}"))?;
            if file.schema_version != 1 {
                return Err("zones.ron: expected schema_version 1".into());
            }
            for rule in file.zone_rules {
                if rule.kind.trim().is_empty() || !kinds.insert(rule.kind.clone()) {
                    return Err(format!("zones.ron: empty or duplicate kind {}", rule.kind));
                }
                if rule.size.0 == 0
                    || rule.size.1 == 0
                    || rule.slot_size.0 == 0
                    || rule.slot_size.1 == 0
                    || rule.slot_size.0 > rule.size.0
                    || rule.slot_size.1 > rule.size.1
                    || rule.people_per_zone == 0
                    || rule.min_spacing > rule.max_gap
                    || rule.max_gap > 32
                    || rule.size.0 > 128
                    || rule.size.1 > 128
                    || rule.min_path_stage > 3
                    || !matches!(
                        rule.trigger.as_str(),
                        "beds_short"
                            | "storage_full"
                            | "workshop_space_short"
                            | "field_need"
                            | "pen_need"
                            | "adult_count"
                    )
                {
                    return Err(format!(
                        "zones.ron: invalid geometry/cap/trigger for {}",
                        rule.kind
                    ));
                }
                if [
                    ZoneBlocker::Nodes,
                    ZoneBlocker::Fields,
                    ZoneBlocker::Structures,
                    ZoneBlocker::Outcrops,
                ]
                .iter()
                .any(|blocker| !rule.avoid.contains(blocker))
                {
                    return Err(format!(
                        "zones.ron: missing physical guard for {}",
                        rule.kind
                    ));
                }
                if rule.status == ContentStatus::Inert
                    && rule.debt.as_ref().is_none_or(|debt| debt.trim().is_empty())
                {
                    return Err(format!("zones.ron: inert kind {} needs debt", rule.kind));
                }
                rules.push(rule);
            }
        }
        Ok(Self { rules })
    }

    /// Resolve a content id.
    pub fn rule_for(&self, kind: &str) -> Option<&ZoneRule> {
        self.rules.iter().find(|rule| rule.kind == kind)
    }

    /// Executable rules in stable authored order.
    pub fn connected(&self) -> impl Iterator<Item = &ZoneRule> {
        self.rules
            .iter()
            .filter(|rule| rule.status == ContentStatus::Connected)
    }
}

/// Parse one complete layout source.
pub fn parse_zones(text: &str) -> Result<ZonesData, String> {
    ZonesData::parse(text, None)
}

/// Rules from the selected catalogue.
pub fn installed() -> &'static ZonesData {
    crate::colony::content::installed_or_bundled().zones()
}

#[cfg(test)]
mod tests {
    use super::*;
    const DATA: &str = include_str!("../../data/colony/zones.ron");

    #[test]
    fn zones_data_validates() {
        let data = parse_zones(DATA).expect("bundled rules");
        assert_eq!(data.connected().count(), 5);
        assert!(
            ZonesData::parse(DATA, Some(DATA)).is_err(),
            "duplicate ids rejected"
        );
        for (from, to) in [
            ("people_per_zone: 8", "people_per_zone: 0"),
            ("size: (9, 9)", "size: (0, 9)"),
            ("min_path_stage: 1", "min_path_stage: 4"),
            ("max_gap: 1", "max_gap: 0"),
            ("slot_size: (4, 4)", "slot_size: (10, 4)"),
            ("trigger: \"beds_short\"", "trigger: \"missing_fact\""),
            ("Fields, Structures, Outcrops", "Fields, Structures"),
        ] {
            assert!(parse_zones(&DATA.replacen(from, to, 1)).is_err(), "{to}");
        }
    }
}
