//! Pass 12 Part 1 (test-only): the J6 bisection scenes.
//!
//! Each `abl_<name>` scene is `game_v17_nofission` (rules and worldgen) with
//! one Pass 11 difference reverted to `game_v16` (the reverse half); each
//! `fwd_<name>` scene is `game_v16` with one Pass 11 difference added (the
//! forward half). Nothing here is a `sim-core` constructor: the scenes edit
//! the v16 / v17 rules and worldgen configs they are built from.
use sim_core::rules::WorldRules;
use sim_core::variety::WorldVariety;
use sim_engine::SimEngine;
use sim_systems::worldgen::{build_world_with, WorldGenConfig};

/// Every ablation scene name the probe accepts.
pub const SCENES: [&str; 13] = [
    "abl_no_regions",
    "abl_v16_damage",
    "abl_no_hunger_weight",
    "abl_no_hardiness",
    "abl_no_regional_wood",
    "abl_no_rain_soak",
    "abl_v16_exposure",
    "fwd_regions",
    "fwd_regions_regional_wood",
    "fwd_v17_damage",
    "fwd_hunger_weight",
    "fwd_hardiness",
    "fwd_exposure_v2",
];

/// Whether `scene` is an ablation scene.
pub fn is_ablation(scene: &str) -> bool {
    SCENES.contains(&scene)
}

fn edit_exposure(rules: &mut WorldRules, f: impl FnOnce(&mut sim_core::rules::ExposureRule)) {
    if let Some(mut exposure) = rules.society.exposure {
        f(&mut exposure);
        rules.society.exposure = Some(exposure);
    }
}

fn edit_firewood(rules: &mut WorldRules, f: impl FnOnce(&mut sim_core::rules::FirewoodRule)) {
    if let Some(mut firewood) = rules.society.firewood {
        f(&mut firewood);
        rules.society.firewood = Some(firewood);
    }
}

/// The engine of ablation scene `scene` for world `seed` (`None` for any
/// other name). The recorded `world_variety` is the one the worldgen used
/// (slots 0..7 only when the world has no regions, as `game_v16`).
pub fn engine(scene: &str, seed: u64) -> Option<SimEngine> {
    let variety = WorldVariety::draw(seed);
    let v16 = WorldRules::game_v16_with(&variety);
    let v17n = WorldRules::game_v17_nofission_with(&variety);
    let v16_exposure = v16.society.exposure;
    let v17_exposure = v17n.society.exposure;
    let gen16 = WorldGenConfig::game_v16_with(seed, &variety);
    let gen17 = WorldGenConfig::game_v17_with(seed, &variety);
    let (gen, mut rules) = match scene {
        n if n.starts_with("abl_") => (if n == "abl_no_regions" { gen16 } else { gen17 }, v17n),
        n if n.starts_with("fwd_") => (if n.starts_with("fwd_regions") { gen17 } else { gen16 }, v16),
        _ => return None,
    };
    match scene {
        "abl_no_regions" => {}
        "abl_v16_damage" => edit_exposure(&mut rules, |e| e.damage_per_cold_hour = v16_exposure.map_or([0.0; 3], |x| x.damage_per_cold_hour)),
        "abl_no_hunger_weight" => edit_exposure(&mut rules, |e| e.hunger_weight = 0.0),
        "abl_no_hardiness" => edit_exposure(&mut rules, |e| e.use_hardiness = false),
        "abl_no_regional_wood" => edit_firewood(&mut rules, |f| f.regional_wood = false),
        "abl_no_rain_soak" => edit_firewood(&mut rules, |f| f.rain_soak_permille = 0),
        "abl_v16_exposure" => rules.society.exposure = v16_exposure,
        "fwd_regions" => {}
        "fwd_regions_regional_wood" => edit_firewood(&mut rules, |f| f.regional_wood = true),
        "fwd_v17_damage" => edit_exposure(&mut rules, |e| e.damage_per_cold_hour = v17_exposure.map_or([0.0; 3], |x| x.damage_per_cold_hour)),
        "fwd_hunger_weight" => edit_exposure(&mut rules, |e| e.hunger_weight = v17_exposure.map_or(0.0, |x| x.hunger_weight)),
        "fwd_hardiness" => edit_exposure(&mut rules, |e| e.use_hardiness = true),
        "fwd_exposure_v2" => rules.society.exposure = v17_exposure,
        _ => return None,
    }
    let regions = gen.wild_regions > 0;
    let mut engine = build_world_with(&gen, rules);
    engine.resources.world_variety = Some(if regions { variety } else { variety.without_regions() });
    Some(engine)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_scene_differs_from_its_base_in_one_place() {
        let variety = WorldVariety::draw(5);
        let v16 = WorldRules::game_v16_with(&variety);
        let v17n = WorldRules::game_v17_nofission_with(&variety);
        for scene in SCENES {
            let e = engine(scene, 5).expect("ablation scene");
            let rules = e.resources.rules;
            let regions = !e.resources.wild_regions.is_empty();
            if scene.starts_with("abl_") {
                assert_eq!(regions, scene != "abl_no_regions", "{scene}");
                assert_eq!(rules == v17n, scene == "abl_no_regions", "{scene}");
            } else {
                assert_eq!(regions, scene.starts_with("fwd_regions"), "{scene}");
                assert_eq!(rules == v16, scene == "fwd_regions", "{scene}");
            }
        }
        assert!(engine("game_v17", 5).is_none());
    }

    #[test]
    fn reverting_exposure_gives_v16_exposure() {
        let e = engine("abl_v16_exposure", 9).expect("scene");
        assert_eq!(e.resources.rules.society.exposure, WorldRules::game_v16(9).society.exposure);
        let f = engine("fwd_exposure_v2", 9).expect("scene");
        assert_eq!(f.resources.rules.society.exposure, WorldRules::game_v17_nofission(9).society.exposure);
    }
}
