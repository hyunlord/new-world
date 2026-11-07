//! Authored era ladder, lane registry and deterministic progress classification.
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// One era in authored order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EraDef {
    /// Stable content id.
    pub id: String,
    /// Localisation key.
    pub name_key: String,
    /// Contiguous progress-vector index.
    pub order: u8,
    /// Progress needed for a transition into this era.
    pub transition_permille: u16,
    /// Progress needed for this era proper.
    pub era_permille: u16,
    /// Narrative chapter metadata.
    pub chapter: String,
}

/// A technology-map lane. Extensions append lanes by data alone.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LaneDef {
    /// Stable content id.
    pub id: String,
    /// Localisation key.
    pub name_key: String,
    /// Contiguous display order.
    pub order: u8,
}

/// Resolved presentation and era thresholds from `eras.ron`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EraCatalog {
    /// Authored format version.
    pub schema_version: u32,
    /// Era records, sorted by order.
    pub eras: Vec<EraDef>,
    /// Lane records, sorted by order.
    pub lanes: Vec<LaneDef>,
    /// Accepted legacy lane names mapped to canonical ids.
    #[serde(default)]
    pub lane_aliases: BTreeMap<String, String>,
}

impl EraCatalog {
    /// Parse and validate a complete ladder.
    pub fn parse(text: &str) -> Result<Self, String> {
        let mut out: Self = ron::from_str(text).map_err(|e| format!("eras.ron: {e}"))?;
        out.eras.sort_by_key(|e| e.order);
        out.lanes.sort_by_key(|l| l.order);
        out.validate()?;
        Ok(out)
    }

    fn validate(&self) -> Result<(), String> {
        if self.schema_version != 1 || self.eras.is_empty() || self.lanes.is_empty() {
            return Err("eras.ron: schema 1, eras and lanes are required".into());
        }
        let mut era_ids = BTreeSet::new();
        for (i, era) in self.eras.iter().enumerate() {
            if usize::from(era.order) != i
                || era.id.is_empty()
                || !era_ids.insert(&era.id)
                || era.name_key.is_empty()
                || era.transition_permille > era.era_permille
                || era.era_permille > 1000
            {
                return Err(format!("eras.ron: invalid era {}", era.id));
            }
        }
        let mut lane_ids = BTreeSet::new();
        for (i, lane) in self.lanes.iter().enumerate() {
            if usize::from(lane.order) != i
                || lane.id.is_empty()
                || !lane_ids.insert(&lane.id)
                || lane.name_key.is_empty()
            {
                return Err(format!("eras.ron: invalid lane {}", lane.id));
            }
        }
        if self.lane_aliases.values().any(|id| !lane_ids.contains(id)) {
            return Err("eras.ron: alias target is not a lane".into());
        }
        Ok(())
    }

    /// Append an extension's eras/lanes, retaining and validating the base prefix.
    pub fn append(&mut self, text: &str) -> Result<(), String> {
        let extension: Self = ron::from_str(text).map_err(|e| format!("eras.ron: {e}"))?;
        if extension.schema_version != 1 {
            return Err("eras.ron: extension requires schema 1".into());
        }
        self.eras.extend(extension.eras);
        self.lanes.extend(extension.lanes);
        for (alias, target) in extension.lane_aliases {
            if self.lane_aliases.insert(alias, target).is_some() {
                return Err("eras.ron: extension replaces a lane alias".into());
            }
        }
        self.eras.sort_by_key(|e| e.order);
        self.lanes.sort_by_key(|l| l.order);
        self.validate()
    }

    /// Resolve a canonical or legacy lane id.
    pub fn lane_order(&self, id: &str) -> Option<u8> {
        let canonical = self.lane_aliases.get(id).map_or(id, String::as_str);
        self.lanes
            .iter()
            .find(|l| l.id == canonical)
            .map(|l| l.order)
    }

    /// Classify a progress vector, allowing regression when practitioners disappear.
    pub fn current(&self, progress: &[u16]) -> (u8, u8) {
        let proper = self.eras.iter().rev().find(|era| {
            progress.get(usize::from(era.order)).copied().unwrap_or(0) >= era.era_permille
        });
        let current = proper.map_or((0, 0), |era| (era.order, 2));
        let next = usize::from(current.0) + 1;
        if let Some(era) = self.eras.get(next) {
            if progress.get(next).copied().unwrap_or(0) >= era.transition_permille {
                return (era.order, 1);
            }
        }
        current
    }
}

/// An authored era's registry index, preserving historical unit-enum save codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Era(pub u8);

#[allow(non_upper_case_globals)]
impl Era {
    /// Historical founder-era identity.
    pub const Paleolithic: Self = Self(0);
    /// Historical second-era identity.
    pub const Neolithic: Self = Self(1);
    /// Registry index; content may append eras without a code change.
    pub const fn code(self) -> u8 {
        self.0
    }
    /// Selected authored id; use a local catalogue while building definitions.
    pub fn name(self) -> &'static str {
        crate::colony::knowledge::defs()
            .eras
            .eras
            .get(usize::from(self.0))
            .map_or("unknown", |e| e.id.as_str())
    }
    /// Resolve the selected data id.
    pub fn from_name(name: &str) -> Option<Self> {
        crate::colony::knowledge::defs()
            .eras
            .eras
            .iter()
            .find(|era| era.id == name)
            .map(|era| Self(era.order))
    }
}
impl Serialize for Era {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if !serializer.is_human_readable() {
            return serializer.serialize_u32(u32::from(self.0));
        }
        let name = match self.0 {
            0 => "Paleolithic",
            1 => "Neolithic",
            _ => self.name(),
        };
        serializer.serialize_unit_variant("Era", u32::from(self.0), name)
    }
}
impl<'de> Deserialize<'de> for Era {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::de::{self, EnumAccess, VariantAccess, Visitor};
        if !deserializer.is_human_readable() {
            return u8::try_from(u32::deserialize(deserializer)?)
                .map(Self)
                .map_err(de::Error::custom);
        }
        struct EraVisitor;
        impl<'de> Visitor<'de> for EraVisitor {
            type Value = Era;
            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("an authored era id")
            }
            fn visit_enum<A: EnumAccess<'de>>(self, access: A) -> Result<Era, A::Error> {
                let (name, variant) = access.variant_seed(NameSeed)?;
                variant.unit_variant()?;
                match name.as_str() {
                    "Paleolithic" => Ok(Era::Paleolithic),
                    "Neolithic" => Ok(Era::Neolithic),
                    name => {
                        let catalog =
                            crate::colony::content::installed().map_err(de::Error::custom)?;
                        catalog
                            .knowledge_defs()
                            .eras
                            .eras
                            .iter()
                            .find(|era| era.id == name)
                            .map(|era| Era(era.order))
                            .ok_or_else(|| de::Error::custom(format!("unknown era {name}")))
                    }
                }
            }
        }
        struct NameSeed;
        impl<'de> de::DeserializeSeed<'de> for NameSeed {
            type Value = String;
            fn deserialize<D: serde::Deserializer<'de>>(self, d: D) -> Result<String, D::Error> {
                d.deserialize_identifier(NameVisitor)
            }
        }
        struct NameVisitor;
        impl<'de> Visitor<'de> for NameVisitor {
            type Value = String;
            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("an era identifier")
            }
            fn visit_str<E: de::Error>(self, value: &str) -> Result<String, E> {
                Ok(value.to_owned())
            }
        }
        deserializer.deserialize_enum("Era", &[], EraVisitor)
    }
}
