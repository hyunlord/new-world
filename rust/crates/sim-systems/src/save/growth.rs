//! Format24 tail for growth state; legacy nested resource/building layouts stay frozen.
use serde::{Deserialize, Serialize};
use sim_core::colony::building_site::ZoneCause;
use sim_engine::SimResources;
use std::collections::{BTreeMap, BTreeSet};

type Tile = (u32, u32);

#[derive(Default, Serialize, Deserialize)]
pub(super) struct GrowthState {
    footfall: Vec<u16>,
    path_stage: Vec<u8>,
    layout_zones: BTreeMap<u32, BTreeSet<Tile>>,
    zone_causes: BTreeMap<u32, ZoneCause>,
    site_zones: BTreeMap<u32, u32>,
    house_growth: BTreeMap<u32, sim_core::colony::yards::HouseProgress>,
}

impl GrowthState {
    pub(super) fn capture(res: &SimResources) -> Self {
        Self {
            footfall: res.colony.footfall.clone(),
            path_stage: res.colony.path_stage.clone(),
            layout_zones: res.colony.buildings.layout_zones.clone(),
            zone_causes: res.colony.buildings.zone_causes.clone(),
            site_zones: res.colony.buildings.site_zones.clone(),
            house_growth: res.colony.buildings.house_growth.clone(),
        }
    }

    pub(super) fn restore(self, res: &mut SimResources) {
        res.colony.footfall = self.footfall;
        res.colony.path_stage = self.path_stage;
        let width = res.tile_grid.width as usize;
        res.colony.footfall_touched = res
            .colony
            .footfall
            .iter()
            .enumerate()
            .filter(|(_, count)| **count > 0)
            .filter_map(|(index, _)| {
                if width == 0 {
                    return None;
                }
                Some((
                    u32::try_from(index % width).ok()?,
                    u32::try_from(index / width).ok()?,
                ))
            })
            .collect();
        res.colony.buildings.layout_zones = self.layout_zones;
        res.colony.buildings.zone_causes = self.zone_causes;
        res.colony.buildings.site_zones = self.site_zones;
        res.colony.buildings.house_growth = self.house_growth;
    }
}

pub(super) fn validate(res: &SimResources) -> Result<(), String> {
    let grid_len = res.tile_grid.width as usize * res.tile_grid.height as usize;
    let colony = &res.colony;
    if colony.footfall.len() != colony.path_stage.len()
        || (!colony.footfall.is_empty() && colony.footfall.len() != grid_len)
    {
        return Err("footfall/path_stage lengths must both be empty or match the grid".into());
    }
    let paths = sim_core::colony::paths::installed();
    for (index, &code) in colony.path_stage.iter().enumerate() {
        let Some(stage) = paths.stage_by_code(code) else {
            return Err("unknown path stage".into());
        };
        if !stage.placed_only {
            continue;
        }
        // Nonempty arrays already require a nonzero grid width above.
        let width = res.tile_grid.width as usize;
        let tile = ((index % width) as u32, (index / width) as u32);
        let physical_stage = res
            .content_state
            .completed
            .get(&(tile, sim_core::colony::content_state::CompletedLayer::Floor))
            .and_then(|identity| identity.blueprint.definition().completed_path_stage);
        if physical_stage != Some(code)
            || res
                .tile_grid
                .floor_material
                .get(index)
                .is_none_or(Option::is_none)
            || res.tile_grid.is_dug(index)
        {
            return Err(format!(
                "placed path at {tile:?} lacks its completed physical floor"
            ));
        }
    }
    let catalog = sim_core::colony::content::installed().map_err(|error| error.to_string())?;
    let state = &colony.buildings;
    let zones: BTreeMap<_, _> = state
        .housing_zones
        .iter()
        .chain(&state.layout_zones)
        .collect();
    if zones.len() != state.housing_zones.len() + state.layout_zones.len() {
        return Err("duplicate housing/layout zone id".into());
    }
    for (&id, tiles) in &zones {
        if *id >= colony.next_zone_id
            || tiles.is_empty()
            || tiles
                .iter()
                .any(|&(x, y)| x >= res.tile_grid.width || y >= res.tile_grid.height)
        {
            return Err(format!("invalid growth zone {id}"));
        }
    }
    for (&id, cause) in &state.zone_causes {
        if !zones.contains_key(&id)
            || catalog.zones().rule_for(&cause.kind).is_none_or(|rule| {
                rule.trigger != cause.trigger
                    || (rule.category == sim_core::colony::zones::ZoneUse::Housing)
                        != state.housing_zones.contains_key(&id)
            })
        {
            return Err(format!("orphan/invalid growth zone cause {id}"));
        }
    }
    if state
        .layout_zones
        .keys()
        .any(|id| !state.zone_causes.contains_key(id))
    {
        return Err("layout zone is missing its authored kind/cause".into());
    }
    for (&site, &zone) in &state.site_zones {
        let Some(building) = state.sites.get(&site) else {
            return Err(format!("orphan building-zone link {site}/{zone}"));
        };
        let Some(tiles) = zones.get(&zone) else {
            return Err(format!("orphan building-zone link {site}/{zone}"));
        };
        let Some(right) = building.anchor.0.checked_add(building.footprint.0) else {
            return Err(format!("overflowing building-zone footprint {site}"));
        };
        let Some(bottom) = building.anchor.1.checked_add(building.footprint.1) else {
            return Err(format!("overflowing building-zone footprint {site}"));
        };
        if !(building.anchor.1..bottom)
            .all(|y| (building.anchor.0..right).all(|x| tiles.contains(&(x, y))))
        {
            return Err(format!(
                "building {site} extends outside linked zone {zone}"
            ));
        }
    }
    for (&site, progress) in &state.house_growth {
        let dwelling = state
            .sites
            .get(&site)
            .and_then(|building| catalog.building_templates().get(&building.template))
            .is_some_and(|template| {
                template.category == sim_core::colony::building_template::BuildingCategory::Dwelling
            });
        let stage = catalog
            .yards()
            .stages
            .iter()
            .find(|stage| stage.code == progress.stage);
        if !dwelling || stage.is_none_or(|stage| progress.occupied_days < stage.min_occupied_days) {
            return Err(format!("invalid house growth {site}"));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::save::{decode, encode, load, load_with, save, BodyV24, Envelope};

    fn engine() -> sim_engine::SimEngine {
        sim_core::colony::content::bootstrap(sim_core::colony::content::BootSelection::Bundled)
            .expect("bundle");
        sim_engine::SimEngine::new(4, 4, sim_core::MaterialRegistry::new())
    }

    #[test]
    fn growth_tail_roundtrips_and_rejects_corrupt_path_lengths_before_registration() {
        let mut engine = engine();
        engine.resources.colony.footfall = vec![0; 16];
        engine.resources.colony.path_stage = vec![0; 16];
        engine.resources.colony.footfall[5] = 123;
        engine.resources.colony.path_stage[5] = 2;
        let valid = save(&mut engine).expect("save growth");
        let mut restored = load_with(&valid, |_| {}).expect("restore growth");
        assert_eq!(restored.resources.colony.footfall[5], 123);
        assert_eq!(restored.resources.colony.path_stage[5], 2);
        assert_eq!(
            restored.resources.colony.footfall_touched,
            [(1, 1)].into_iter().collect()
        );
        assert_eq!(save(&mut restored).expect("stable growth"), valid);
        let mut envelope: Envelope = decode(&valid[12..]).expect("envelope");
        let mut body: BodyV24 = decode(&envelope.payload).expect("body24");
        body.growth.path_stage.pop();
        envelope.payload = encode(&body).expect("malformed growth");
        let mut corrupt = valid[..12].to_vec();
        corrupt.extend(encode(&envelope).expect("envelope"));
        let mut registered = false;
        assert!(load_with(&corrupt, |_| registered = true).is_err());
        assert!(!registered);
    }

    #[test]
    fn growth_tail_rejects_orphan_zone_causes_and_links() {
        let mut engine = engine();
        engine.resources.colony.buildings.zone_causes.insert(
            1,
            ZoneCause {
                kind: "housing".into(),
                trigger: "beds_short".into(),
                people: 12,
                slots: 9,
            },
        );
        assert!(save(&mut engine).is_err());
        engine.resources.colony.next_zone_id = 2;
        engine
            .resources
            .colony
            .buildings
            .housing_zones
            .insert(1, [(1, 1)].into_iter().collect());
        engine.resources.colony.next_zone_id = 3;
        engine
            .resources
            .colony
            .buildings
            .layout_zones
            .insert(2, [(2, 2)].into_iter().collect());
        engine.resources.colony.buildings.zone_causes.insert(
            2,
            ZoneCause {
                kind: "workshop".into(),
                trigger: "workshop_space_short".into(),
                people: 12,
                slots: 1,
            },
        );
        let valid = save(&mut engine).expect("zone cause");
        let restored = load_with(&valid, |_| {}).expect("zone cause roundtrip");
        assert_eq!(
            restored.resources.colony.buildings.layout_zones,
            engine.resources.colony.buildings.layout_zones
        );
        assert_eq!(
            restored.resources.colony.buildings.zone_causes,
            engine.resources.colony.buildings.zone_causes
        );
        engine.resources.colony.buildings.site_zones.insert(999, 1);
        assert!(save(&mut engine).is_err());
    }

    #[test]
    fn growth_tail_preserves_layout_links_and_house_progress_and_rejects_invalid_references() {
        use sim_core::colony::building_site::{BuildCause, BuildingSite, BuildingStatus};
        use sim_core::colony::yards::HouseProgress;

        let mut engine = engine();
        let res = &mut engine.resources;
        res.colony.next_zone_id = 3;
        let state = &mut res.colony.buildings;
        state.layout_zones.insert(2, [(2, 2)].into_iter().collect());
        state.zone_causes.insert(
            2,
            ZoneCause {
                kind: "workshop".into(),
                trigger: "workshop_space_short".into(),
                people: 24,
                slots: 1,
            },
        );
        state
            .housing_zones
            .insert(1, [(1, 1)].into_iter().collect());
        state.sites.insert(
            0,
            BuildingSite {
                id: 0,
                template: "hut".into(),
                station: None,
                anchor: (1, 1),
                rotation: 0,
                footprint: (1, 1),
                pieces: vec![],
                cause: BuildCause::Zone(1),
                source_record: 0,
                placed_tick: 0,
                status: BuildingStatus::Building,
                completion_entry: None,
            },
        );
        state.site_zones.insert(0, 1);
        state.house_growth.insert(
            0,
            HouseProgress {
                occupied_days: 9,
                stage: 0,
            },
        );
        validate(res).expect("valid growth references");
        let bytes = encode(&GrowthState::capture(res)).expect("growth tail");
        let tail: GrowthState = decode(&bytes).expect("tail restore");
        let expected = res.colony.buildings.clone();
        res.colony.buildings.layout_zones.clear();
        res.colony.buildings.zone_causes.clear();
        res.colony.buildings.site_zones.clear();
        res.colony.buildings.house_growth.clear();
        tail.restore(res);
        assert_eq!(res.colony.buildings, expected);
        assert_eq!(
            encode(&GrowthState::capture(res)).expect("stable tail"),
            bytes
        );

        res.colony.buildings.site_zones.insert(0, 2);
        assert!(validate(res)
            .expect_err("footprint outside zone")
            .contains("outside"));
        res.colony.buildings = expected.clone();
        res.colony
            .buildings
            .house_growth
            .get_mut(&0)
            .expect("house")
            .stage = u8::MAX;
        assert!(validate(res)
            .expect_err("unknown stage")
            .contains("house growth"));
        res.colony.buildings = expected.clone();
        res.colony
            .buildings
            .sites
            .get_mut(&0)
            .expect("site")
            .template = "workshop_shed".into();
        assert!(validate(res)
            .expect_err("non-dwelling stage")
            .contains("house growth"));
        res.colony.buildings = expected.clone();
        res.colony
            .buildings
            .layout_zones
            .insert(1, [(0, 0)].into_iter().collect());
        assert!(validate(res)
            .expect_err("shared zone id")
            .contains("duplicate"));
        res.colony.buildings = expected.clone();
        res.colony
            .buildings
            .layout_zones
            .insert(2, [(4, 0)].into_iter().collect());
        assert!(validate(res)
            .expect_err("zone outside grid")
            .contains("invalid growth zone"));
        res.colony.buildings = expected.clone();
        res.colony.buildings.zone_causes.remove(&2);
        assert!(validate(res)
            .expect_err("untyped layout zone")
            .contains("missing"));
        res.colony.buildings = expected.clone();
        res.colony.buildings.zone_causes.insert(
            2,
            ZoneCause {
                kind: "housing".into(),
                trigger: "beds_short".into(),
                people: 24,
                slots: 1,
            },
        );
        assert!(validate(res)
            .expect_err("housing cause on layout zone")
            .contains("cause"));
        res.colony.buildings = expected;
        res.colony
            .buildings
            .zone_causes
            .get_mut(&2)
            .expect("cause")
            .trigger = "beds_short".into();
        assert!(validate(res)
            .expect_err("wrong cause trigger")
            .contains("cause"));
    }

    #[test]
    fn growth_tail_rejects_unknown_or_unbuilt_placed_path_before_registration() {
        let mut engine = engine();
        engine.resources.colony.footfall = vec![0; 16];
        engine.resources.colony.path_stage = vec![0; 16];
        let valid = save(&mut engine).expect("valid save");
        for stage in [u8::MAX, sim_core::colony::paths::STAGE_PAVED] {
            let mut envelope: Envelope = decode(&valid[12..]).expect("envelope");
            let mut body: BodyV24 = decode(&envelope.payload).expect("body24");
            body.growth.path_stage[0] = stage;
            envelope.payload = encode(&body).expect("corrupt stage");
            let mut corrupt = valid[..12].to_vec();
            corrupt.extend(encode(&envelope).expect("envelope"));
            let mut registered = false;
            assert!(
                load_with(&corrupt, |_| registered = true).is_err(),
                "stage {stage} must be rejected"
            );
            assert!(!registered);
        }
    }

    #[test]
    fn genuine_c3_save_migrates_then_continues_identically() {
        let Ok(path) = std::env::var("MGROW_C3_SAVE_PATH") else {
            return;
        };
        let _ = engine();
        let bytes = std::fs::read(path).expect("genuine C3 fixture");
        assert_eq!(&bytes[8..12], &23_u32.to_le_bytes());
        let envelope: Envelope = decode(&bytes[12..]).expect("C3 envelope");
        let old: crate::save::Body = decode(&envelope.payload).expect("genuine C3 body");
        let mut restored = load(&bytes).expect("genuine C3 load");
        let systems = restored.system_states().expect("migrated systems");
        for name in ["PathDaySystem", "HouseGrowthSystem"] {
            assert_eq!(systems.iter().filter(|entry| entry.0 == name).count(), 1);
        }
        let retained: Vec<_> = systems
            .into_iter()
            .filter(|entry| !matches!(entry.0.as_str(), "PathDaySystem" | "HouseGrowthSystem"))
            .collect();
        assert_eq!(
            retained, old.systems,
            "every C3 slot retains its original bytes"
        );
        assert!(restored
            .resources
            .colony
            .footfall
            .iter()
            .all(|count| *count == 0));
        validate(&restored.resources).expect("valid migrated growth, including physical roads");
        for (&(tile, layer), identity) in &restored.resources.content_state.completed {
            if layer != sim_core::colony::content_state::CompletedLayer::Floor {
                continue;
            }
            let Some(stage) = identity.blueprint.definition().completed_path_stage else {
                continue;
            };
            let index = restored.resources.tile_grid.idx(tile.0, tile.1);
            assert_eq!(
                restored.resources.colony.path_stage[index], stage,
                "legacy completed roads restore their authored surface stage"
            );
        }
        assert!(restored.resources.colony.buildings.zone_causes.is_empty());
        let current = save(&mut restored).expect("migrated24 save");
        let mut twin = load(&current).expect("migrated24 load");
        assert_eq!(save(&mut twin).expect("stable migrated24"), current);
        for _ in 0..60 {
            restored.tick();
            twin.tick();
        }
        assert_eq!(
            save(&mut restored).expect("continued C3"),
            save(&mut twin).expect("continued24")
        );
    }
}
