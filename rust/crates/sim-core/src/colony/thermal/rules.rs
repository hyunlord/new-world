//! Authored colony thermal profile resolved through actual material and item registries.
use crate::colony::{content::Catalog, ItemKind};
use crate::material::{MaterialDef, MaterialId};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Resolved integer thermal values pinned to their authored profile.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ThermalRules {
    /// Exact authored profile fingerprint, independent of the baseline catalog.
    #[serde(default)]
    pub profile_revision: String,
    /// Outdoor seasonal bases in spring, summer, autumn, winter order.
    pub season_outdoor_d10: [i32; 4],
    /// Twenty-four authored hourly outdoor offsets in deci-degrees.
    pub hour_offset_d10: [i32; 24],
    /// Additional outdoor offset for an active cold pressure.
    pub cold_snap_offset_d10: i32,
    /// Felt temperature below which the colony derives Cold.
    pub cold_threshold_d10: i32,
    /// Heat capacity per authoritative interior tile.
    pub capacity_per_tile: u32,
    /// Combined roof and floor conductance per interior tile.
    pub roof_floor_loss_per_tile: u32,
    /// Conductance of a closed door edge.
    pub closed_door_loss: u32,
    /// Conductance of an open door edge.
    pub open_door_loss: u32,
    /// Automatic open duration after an actual passage.
    pub door_passage_ticks: u32,
    /// Heat contributed by one fully paid positive fuel interval.
    pub paid_hearth_heat: u32,
    /// Heat per living adult physically present.
    pub adult_body_heat: u32,
    /// Heat per living child physically present.
    pub child_body_heat: u32,
    /// Insulation by real saved construction material identity, range0..999.
    #[serde(default)]
    pub material_insulation_permille: BTreeMap<MaterialId, u16>,
    /// Additive felt warmth by real worn item identity.
    #[serde(default)]
    pub apparel_insulation_d10: BTreeMap<ItemKind, i32>,
    /// Explicit conductance fallback for enclosing natural terrain.
    pub natural_boundary_loss: u32,
    /// Validated authored dwelling progression definitions.
    pub stages: Vec<DwellingStageDef>,
}
/// An authored physical dwelling predicate and bounded mood contribution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DwellingStageDef {
    /// Stable data identity persisted in dwelling history.
    pub id: u16,
    /// Localization key used by the existing room interface.
    pub label_key: String,
    /// Ordering independent of the stable stage identity.
    pub rank: u16,
    /// Minimum physically constructed beds.
    pub min_beds: u16,
    /// Required eligible floor coverage in permille.
    pub floor_coverage_permille: u16,
    /// Requires current fully paid positive hearth service.
    pub requires_paid_heat: bool,
    /// Requires actual current warm occupants, not an empty warm room.
    pub requires_warm_occupants: bool,
    /// Required reachable stored food measured in adult-day demand.
    pub nearby_food_adult_days: u16,
    /// Maximum navigation distance for nearby physical stores.
    pub food_nav_distance: u32,
    /// Continuous deficient duration before dropping the displayed stage.
    pub downgrade_delay_ticks: u64,
    /// Single additive mood contribution for a resident.
    pub mood_permille: i32,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Profile {
    schema_version: u32,
    rules: ThermalRules,
    materials: Vec<(String, u16)>,
    apparel: Vec<(String, i32)>,
}
impl ThermalRules {
    /// Bundled colony-only authored starting values.
    pub const SOURCE: &'static str = include_str!("../../../data/colony/thermal.ron");
    /// Resolve the bundled profile against the installed physical catalog.
    pub fn bundled() -> Result<Self, String> {
        Ok(crate::colony::content::installed_or_bundled()
            .thermal_rules()
            .clone())
    }
    pub(crate) fn unresolved(source: &str) -> Result<Self, String> {
        ron::from_str::<Profile>(source)
            .map(|profile| profile.rules)
            .map_err(|e| e.to_string())
    }
    /// A selected thermal profile never changes the historical content revision.
    pub fn from_source(
        source: &str,
        catalog: &Catalog,
        materials: &[MaterialDef],
    ) -> Result<Self, String> {
        let profile: Profile = ron::from_str(source).map_err(|e| e.to_string())?;
        if profile.schema_version != 1
            || !profile.rules.material_insulation_permille.is_empty()
            || !profile.rules.apparel_insulation_d10.is_empty()
        {
            return Err("thermal profile must resolve names through schema1 registries".into());
        }
        let mut rules = profile.rules;
        if !rules.profile_revision.is_empty() {
            return Err("profile revision is computed from source".into());
        }
        for (name, value) in profile.materials {
            let material = materials
                .iter()
                .find(|d| d.name.to_lowercase() == name)
                .ok_or_else(|| format!("unknown thermal material {name}"))?;
            if material.id != MaterialId::from_str_hash(&name)
                || rules
                    .material_insulation_permille
                    .insert(material.id, value)
                    .is_some()
            {
                return Err("duplicate or mismatched thermal material".into());
            }
        }
        if materials
            .iter()
            .any(|d| !rules.material_insulation_permille.contains_key(&d.id))
        {
            return Err("thermal profile omits a registered construction material".into());
        }
        for (name, value) in profile.apparel {
            let id = catalog
                .item_by_name(&name)
                .filter(|id| catalog.item(*id).is_some_and(|d| d.apparel.is_some()))
                .ok_or_else(|| format!("unknown thermal apparel {name}"))?;
            if rules.apparel_insulation_d10.insert(id, value).is_some() {
                return Err("duplicate thermal apparel".into());
            }
        }
        rules.profile_revision = rules.resolved_revision()?;
        rules.validate()?;
        Ok(rules)
    }
    /// Hash resolved numeric identities and values without depending on the global catalog.
    pub fn resolved_revision(&self) -> Result<String, String> {
        let mut copy = self.clone();
        copy.profile_revision.clear();
        let apparel: Vec<_> = std::mem::take(&mut copy.apparel_insulation_d10)
            .into_iter()
            .map(|(kind, warmth)| (kind.code(), warmth))
            .collect();
        let bytes = ron::ser::to_string(&(copy, apparel)).map_err(|e| e.to_string())?;
        let hash = bytes.bytes().fold(0xcbf29ce484222325u64, |h, b| {
            (h ^ u64::from(b)).wrapping_mul(0x100000001b3)
        });
        Ok(format!("thermal-v1-fnv1a64:{hash:016x}:{}", bytes.len()))
    }

    /// Bounds make checked i64 thermal arithmetic finite for supported room geometry.
    pub fn validate(&self) -> Result<(), String> {
        if self.profile_revision != self.resolved_revision()? {
            return Err("thermal resolved profile revision mismatch".into());
        }
        let values = [
            self.capacity_per_tile,
            self.roof_floor_loss_per_tile,
            self.closed_door_loss,
            self.open_door_loss,
            self.paid_hearth_heat,
            self.adult_body_heat,
            self.child_body_heat,
            self.natural_boundary_loss,
        ];
        if self.capacity_per_tile == 0
            || self.natural_boundary_loss == 0
            || values.iter().any(|v| *v > 1_000_000)
            || self.closed_door_loss == 0
            || self.open_door_loss < self.closed_door_loss
            || self.door_passage_ticks > 1440
        {
            return Err("invalid thermal capacity, conductance or passage bound".into());
        }
        if self
            .season_outdoor_d10
            .iter()
            .chain(self.hour_offset_d10.iter())
            .chain([&self.cold_snap_offset_d10, &self.cold_threshold_d10])
            .any(|v| !(-10000..=10000).contains(v))
            || self.material_insulation_permille.values().any(|v| *v > 999)
            || self
                .apparel_insulation_d10
                .values()
                .any(|v| !(0..=1000).contains(v))
            || self.material_insulation_permille.is_empty()
        {
            return Err("invalid thermal temperature or insulation".into());
        }
        let mut ids = BTreeSet::new();
        let mut ranks = BTreeSet::new();
        for stage in &self.stages {
            if !ids.insert(stage.id)
                || !ranks.insert(stage.rank)
                || stage.label_key.is_empty()
                || stage.floor_coverage_permille > 1000
                || stage.min_beds == 0
                || stage.food_nav_distance > 4096
                || stage.nearby_food_adult_days > 365
                || stage.downgrade_delay_ticks > 1440 * 365
                || !(-1000..=1000).contains(&stage.mood_permille)
            {
                return Err("invalid dwelling stage".into());
            }
        }
        if self.stages.is_empty() {
            return Err("thermal profile needs dwelling stages".into());
        }
        Ok(())
    }
}
