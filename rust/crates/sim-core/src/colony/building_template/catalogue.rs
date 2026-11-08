//! Local building-template parser; connected content resolves against the selected catalogue.
use super::{geometry, BuildingTemplate, BuildingZoneKind, ContentStatus};
use crate::colony::content::Catalog;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// Validated template family, retaining file order for menu display.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TemplateCatalog {
    /// Version of the authored template schema.
    pub schema_version: u32,
    /// All templates, including grey/inert entries.
    pub buildings: Vec<BuildingTemplate>,
    /// Zone rows retain source order; omitted extension rows contribute nothing.
    #[serde(default)]
    pub(super) zone_kinds: Vec<BuildingZoneKind>,
}

impl TemplateCatalog {
    /// Parse a base family plus optional data-only extension, resolving connected entries.
    pub fn parse(base: &str, extension: Option<&str>, catalog: &Catalog) -> Result<Self, String> {
        let mut out: Self = ron::from_str(base).map_err(|e| format!("buildings.ron: {e}"))?;
        if let Some(text) = extension {
            let extra: Self = ron::from_str(text).map_err(|e| format!("buildings.ron: {e}"))?;
            if extra.schema_version != 1 {
                return Err("buildings.ron: extension requires schema 1".into());
            }
            out.buildings.extend(extra.buildings);
            out.zone_kinds.extend(extra.zone_kinds);
        }
        out.validate(catalog)?;
        Ok(out)
    }

    /// Resolve a content id without using the installed global catalogue.
    pub fn get(&self, id: &str) -> Option<&BuildingTemplate> {
        self.buildings.iter().find(|template| template.id == id)
    }

    /// Authored automatic-growth zones in stable base-then-extension order.
    pub fn zone_kinds(&self) -> &[BuildingZoneKind] {
        &self.zone_kinds
    }

    fn validate(&self, catalog: &Catalog) -> Result<(), String> {
        if self.schema_version != 1 {
            return Err("buildings.ron: schema 1 required".into());
        }
        let mut zone_ids = BTreeSet::new();
        for zone in &self.zone_kinds {
            if zone.id.trim().is_empty()
                || zone.name_key.trim().is_empty()
                || !zone_ids.insert(&zone.id)
            {
                return Err(format!("zone {}: invalid or duplicate identity", zone.id));
            }
            match zone.status {
                ContentStatus::Connected if zone.id != "housing" => {
                    return Err(format!("zone {}: no connected zone driver", zone.id));
                }
                ContentStatus::Inert
                    if zone.debt.as_deref().is_none_or(|d| d.trim().is_empty()) =>
                {
                    return Err(format!("zone {}: inert zone needs debt", zone.id));
                }
                _ => {}
            }
        }
        let mut ids = BTreeSet::new();
        for template in &self.buildings {
            if template.id.is_empty()
                || !ids.insert(&template.id)
                || template.name_key.is_empty()
                || !(1..=4).contains(&template.tier)
                || template.footprint.0 == 0
                || template.footprint.1 == 0
                || template.footprint.0 > 64
                || template.footprint.1 > 64
                || !catalog
                    .knowledge_defs()
                    .eras
                    .eras
                    .iter()
                    .any(|era| era.id == template.era)
                || template.art.tint_material.is_empty()
                || template.art.icon.is_empty()
            {
                return Err(format!(
                    "building {}: invalid identity/era/tier/footprint/art",
                    template.id
                ));
            }
            if template
                .function
                .storage
                .as_deref()
                .is_some_and(|selector| selector != "all" && selector != "grain")
            {
                return Err(format!(
                    "building {}: unsupported storage selector (expected all, grain or None)",
                    template.id
                ));
            }
            if let Some(id) = &template.zone_kind {
                let zone = self
                    .zone_kinds
                    .iter()
                    .find(|zone| &zone.id == id)
                    .ok_or_else(|| format!("building {}: unknown zone {id}", template.id))?;
                if template.status == ContentStatus::Connected
                    && zone.status != ContentStatus::Connected
                {
                    return Err(format!(
                        "building {}: connected template requires connected zone {id}",
                        template.id
                    ));
                }
            }
            if template.status == ContentStatus::Inert {
                if template
                    .debt
                    .as_deref()
                    .is_none_or(|debt| debt.trim().is_empty())
                {
                    return Err(format!(
                        "building {}: inert template needs debt",
                        template.id
                    ));
                }
                continue;
            }
            for node in &template.requires_knowledge {
                if catalog.knowledge_defs().id_of(node).is_none() {
                    return Err(format!(
                        "building {}: unknown knowledge {node}",
                        template.id
                    ));
                }
            }
            for target in [&template.upgrade_from, &template.upgrade_to]
                .into_iter()
                .flatten()
            {
                if self.get(target).is_none() {
                    return Err(format!(
                        "building {}: unknown upgrade {target}",
                        template.id
                    ));
                }
            }
            for station in &template.stations {
                template.expand(catalog, Some(station), 0)?;
            }
            let planned = template.expand(catalog, None, 0)?;
            let cost = geometry::material_cost(&planned, catalog)?;
            if cost != template.materials {
                return Err(format!(
                    "building {}: declared materials {:?} != expanded {:?}",
                    template.id, template.materials, cost
                ));
            }
        }
        Ok(())
    }
}
