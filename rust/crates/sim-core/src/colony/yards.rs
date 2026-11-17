//! Data-authored dwelling stages and decorative yard props.
use super::content::Catalog;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// Saved daily residence and the highest stage actually reached.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HouseProgress {
    /// Days with at least one live assigned occupant.
    pub occupied_days: u32,
    /// Stable authored stage code; stages never regress.
    pub stage: u8,
}

/// One monotone dwelling stage, shared by the dwelling category.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct YardStage {
    /// Stable content identifier.
    pub id: String,
    /// Append-only saved code.
    pub code: u8,
    /// Localized stage label.
    pub name_key: String,
    /// Required days with live assigned occupants.
    pub min_occupied_days: u32,
    /// Required physical goods in the settlement's stockpiles.
    pub min_stored_goods: u32,
    /// Required highest skill level of an assigned occupant.
    pub min_occupant_skill: u8,
}

/// Decorative model placement relative to a dwelling's unrotated top-left.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct YardProp {
    /// Stable content identifier, also the deterministic placement order.
    pub id: String,
    /// Existing 3D asset manifest key.
    pub model: String,
    /// First dwelling stage that shows this prop.
    pub min_stage: u8,
    /// Signed tile offset from the footprint anchor.
    pub offset: (i32, i32),
    /// Clockwise quarter turns.
    pub rotation: u8,
}

/// Validated shared dwelling-category fallback and its data-only extensions.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct YardsData {
    /// Stages in stable-code order.
    pub stages: Vec<YardStage>,
    /// Props in authored order; visual readers skip blocked placements.
    pub props: Vec<YardProp>,
    /// Era progress needed before the steward selects its dwelling material.
    pub material_wave_permille: u16,
    /// Upper bound on active dwelling generations before admitting an upgrade.
    pub max_simultaneous_upgrades: u8,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct File {
    schema_version: u32,
    #[serde(default)]
    stages: Vec<YardStage>,
    #[serde(default)]
    props: Vec<YardProp>,
    #[serde(default)]
    material_wave_permille: Option<u16>,
    #[serde(default)]
    max_simultaneous_upgrades: Option<u8>,
}

impl YardsData {
    /// Parse and validate selected base and append-only extension data.
    pub fn parse(base: &str, extension: Option<&str>, _catalog: &Catalog) -> Result<Self, String> {
        let mut file = parse_file(base)?;
        if let Some(text) = extension {
            let extra = parse_file(text)?;
            if extra.material_wave_permille.is_some() || extra.max_simultaneous_upgrades.is_some() {
                return Err("yards: extensions cannot replace the material-wave threshold".into());
            }
            file.stages.extend(extra.stages);
            file.props.extend(extra.props);
        }
        file.stages.sort_by_key(|stage| stage.code);
        let mut ids = BTreeSet::new();
        for (index, stage) in file.stages.iter().enumerate() {
            if usize::from(stage.code) != index
                || stage.id.is_empty()
                || !ids.insert(stage.id.as_str())
                || stage.name_key.is_empty()
                || stage.min_occupant_skill > super::skills::MAX_LEVEL
            {
                return Err(format!("yards: invalid stage {}", stage.id));
            }
        }
        if file.stages.first().is_none_or(|stage| {
            stage.min_occupied_days != 0
                || stage.min_stored_goods != 0
                || stage.min_occupant_skill != 0
        }) {
            return Err("yards: stage zero must be an unconditional fallback".into());
        }
        let mut prop_ids = BTreeSet::new();
        for prop in &file.props {
            if prop.id.is_empty()
                || !prop_ids.insert(prop.id.as_str())
                || prop.model.is_empty()
                || prop.model.contains(['/', '\\', '.'])
                || prop.rotation > 3
                || usize::from(prop.min_stage) >= file.stages.len()
                || prop.offset.0.unsigned_abs() > 64
                || prop.offset.1.unsigned_abs() > 64
            {
                return Err(format!("yards: invalid prop {}", prop.id));
            }
        }
        let material_wave_permille = file
            .material_wave_permille
            .ok_or("yards: material-wave threshold is required")?;
        if material_wave_permille > 1000 {
            return Err("yards: material-wave threshold exceeds 1000".into());
        }
        let max_simultaneous_upgrades = file
            .max_simultaneous_upgrades
            .ok_or("yards: upgrade cap is required")?;
        if max_simultaneous_upgrades != 1 {
            return Err("yards: at most one simultaneous dwelling upgrade is supported".into());
        }
        Ok(Self {
            stages: file.stages,
            props: file.props,
            material_wave_permille,
            max_simultaneous_upgrades,
        })
    }

    /// Highest stage allowed by residence time, actual stored goods and occupant skill.
    pub fn stage_for(&self, days: u32, goods: u32, skill: u8) -> u8 {
        self.stages
            .iter()
            .filter(|stage| {
                days >= stage.min_occupied_days
                    && goods >= stage.min_stored_goods
                    && skill >= stage.min_occupant_skill
            })
            .map(|stage| stage.code)
            .max()
            .unwrap_or(0)
    }
}

fn parse_file(text: &str) -> Result<File, String> {
    let file: File = ron::from_str(text).map_err(|error| format!("yards: {error}"))?;
    if file.schema_version != 1 {
        return Err(format!("yards: unsupported schema {}", file.schema_version));
    }
    Ok(file)
}

#[cfg(test)]
mod tests {
    use super::*;
    const DATA: &str = include_str!("../../data/colony/yards.ron");

    #[test]
    fn house_stage_progresses_by_data() {
        let catalog = super::super::content::installed_or_bundled();
        let data = YardsData::parse(DATA, None, catalog).expect("yards");
        assert_eq!(data.stage_for(0, 100, 20), 0);
        assert_eq!(data.stage_for(2, 0, 0), 1);
        assert_eq!(data.stage_for(8, 0, 20), 1);
        assert_eq!(data.stage_for(8, 1, 0), 1);
        assert_eq!(data.stage_for(8, 1, 1), 2);
        let extension = r#"(schema_version:1,props:[(id:"extra",model:"item_049_hay_s",min_stage:2,offset:(-1,0),rotation:0)])"#;
        let extended = YardsData::parse(DATA, Some(extension), catalog).expect("data-only prop");
        assert_eq!(extended.props.len(), data.props.len() + 1);
        assert_eq!(extended.stage_for(8, 1, 1), 2);
        assert!(YardsData::parse(
            DATA,
            Some(extension.replace("extra", "woodpile").as_str()),
            catalog
        )
        .is_err());
    }
}
