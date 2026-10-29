//! Save files (M1.5 §7, contract clause C1): the whole simulation state as
//! bytes, and an engine rebuilt from them that ticks exactly as the saved
//! one would have.
//!
//! Format: the 8-byte magic `WORLDSIM`, [`FORMAT_VERSION`] as a little-endian
//! `u32`, then a `bincode` 1.x envelope (varint integers, little endian)
//! containing the catalog revision and opaque body bytes. Revision validation
//! precedes typed body decoding. The body contains current tick, resources,
//! world, systems, Steward state and sparse content identity:
//! - `resources`: every [`SimResources`] field (rules, seed, command log,
//!   colony, chronicle, grids, …) by serde; hash maps and sets are written in
//!   sorted key order (`sim_core::save_serde`). The material blocking cache
//!   is derived and rebuilt on load.
//! - `world`: every entity of every archetype, archetypes in creation order
//!   (empty ones included) and entities in their dense order, each component
//!   column named through the component registry in [`world`], plus the hecs
//!   entity allocator (id range and free list with generations).
//! - `systems`: each registered system's `(name, state)` in execution order
//!   ([`sim_engine::RuntimeSystem::save_state`]).
//!
//! Why this lives in sim-systems and why [`save`] takes `&mut`: the
//! component registry must name `MovementRng` and `InfluenceSample`, which
//! are sim-systems types, and a ready engine needs
//! [`crate::register_default_runtime_systems`]. hecs 0.10 exposes its
//! allocator's free list only through `World::reserve_entity`, which leaves
//! reserved ids in the world; so [`save`] reads the free list that way and
//! then replaces the engine's world by one rebuilt from the bytes it just
//! wrote (the same code path as [`load`]). The engine after a save is
//! therefore the loaded engine; the round-trip tests prove it ticks the same.

mod content_validation;
mod food_catalog;
#[cfg(test)]
mod food_tests;
mod legacy_steward;
mod legacy_v9;
#[cfg(test)]
mod mind_tests;
mod mind_validation;
pub mod world;

#[cfg(test)]
mod legacy_codec_tests;

#[cfg(test)]
mod legacy_id_codec_tests;

#[cfg(test)]
mod legacy_pressure_codec_tests;

use std::fmt;

use bincode::Options;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use sim_core::influence::MaterialBlockingCache;
use sim_core::MaterialRegistry;
use sim_engine::{SimEngine, SimResources};

/// The first eight bytes of every save file.
pub const MAGIC: [u8; 8] = *b"WORLDSIM";

/// Save format version; historical versions 6 through 12 are migrated. Bump it
/// whenever a saved type or the layout changes. 2 = M2 (item specs, nodes,
/// workshops and bills, multi-line material needs, tools, skills, priorities);
/// 3 = M3 (door and dug tile columns, navigation paths, space sites, rooms);
/// 4 = M4 (`Knowledge`, seven skills, the ford column, apprenticeship orders,
/// the technology aggregate, new item, recipe, blueprint and job kinds);
/// 5 = M5 (the storyteller, `Sick`, `Apparel`, new item and chronicle kinds,
/// the chronicle's cause table, animals, fights, drafting, mood, bonds); 6 = M5.5 (`Fleeing`, neighbours,
/// migrants, caravans, the trade ledger, need samples, new chronicle kinds and the knowledge source `Migrant`);
/// 7 = optional Steward state; 8 = catalog-bound envelope, full-width content
/// codes and Chronicle causes, sparse completed-construction/tool identity;
/// 9 = full-width pressure provenance and persistent novel illness origins;
/// 10 = tree material identity; 11 = independent steward defence and ownership;
/// 12 = authored mind policy, running choices and independent decision history;
/// 13 = independently persisted seasonal plants and physical food ownership.
pub const FORMAT_VERSION: u32 = 13;

/// Upper bound for one decode (guards against corrupt length prefixes).
const DECODE_LIMIT: u64 = 1 << 31;

/// Why a save or a load failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SaveError {
    /// The bytes do not start with [`MAGIC`] (not a save file or truncated).
    NotASave,
    /// The file has a different [`FORMAT_VERSION`].
    Version {
        /// The version in the file.
        found: u32,
        /// The version this build reads.
        expected: u32,
    },
    /// Encoding failed.
    Encode(String),
    /// Decoding failed (corrupt or truncated body).
    Decode(String),
    /// The ECS world could not be captured or rebuilt.
    World(String),
    /// A system's state could not be saved or loaded, or the registered
    /// systems differ from the saved list.
    Systems(String),
}

impl fmt::Display for SaveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotASave => write!(f, "not a WorldSim save file"),
            Self::Version { found, expected } => {
                write!(
                    f,
                    "save format version {found}, this build reads {expected}"
                )
            }
            Self::Encode(e) => write!(f, "save encode failed: {e}"),
            Self::Decode(e) => write!(f, "save decode failed: {e}"),
            Self::World(e) => write!(f, "save world: {e}"),
            Self::Systems(e) => write!(f, "save systems: {e}"),
        }
    }
}

impl std::error::Error for SaveError {}

/// The bincode options of every save encoding (body, columns, system state).
fn options() -> impl Options {
    bincode::DefaultOptions::new().with_limit(DECODE_LIMIT)
}

/// Encode `value` with the save options (used by systems for their state).
pub fn encode<T: Serialize + ?Sized>(value: &T) -> Result<Vec<u8>, String> {
    options().serialize(value).map_err(|e| e.to_string())
}

/// Decode a value written by [`encode`]; trailing bytes are an error.
pub fn decode<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, String> {
    options().deserialize(bytes).map_err(|e| e.to_string())
}

/// The body after the header, as written.
#[derive(Serialize)]
struct BodyRef<'a> {
    current_tick: u64,
    resources: &'a SimResources,
    world: &'a world::WorldSave,
    systems: &'a [(String, Vec<u8>)],
    steward: &'a sim_core::colony::steward::StewardState,
    content_state: &'a sim_core::colony::content_state::ContentState,
    mind: &'a sim_core::colony::mind_state::MindState,
    food: &'a sim_core::colony::food::FoodState,
}

/// The body after the header, as read (same fields, same order).
#[derive(Deserialize)]
struct LegacyBody {
    current_tick: u64,
    resources: sim_engine::legacy_v7::LegacySimResources,
    world: world::WorldSave,
    systems: Vec<(String, Vec<u8>)>,
}

/// The published v7 body has the v6 resource wire layout plus Steward history.
#[derive(Deserialize)]
struct LegacyBodyV7 {
    current_tick: u64,
    resources: sim_engine::legacy_v7::LegacySimResources,
    world: world::WorldSave,
    systems: Vec<(String, Vec<u8>)>,
    steward: legacy_steward::LegacyStewardState,
}

/// Format8 stores the original two sparse content maps, with no illness origins.
#[derive(Deserialize)]
struct LegacyContentStateV8 {
    completed: std::collections::BTreeMap<
        ((u32, u32), sim_core::colony::content_state::CompletedLayer),
        sim_core::colony::content_state::CompletedIdentity,
    >,
    workshop_cutters: std::collections::BTreeMap<u32, sim_core::colony::ItemKind>,
}
impl From<LegacyContentStateV8> for sim_core::colony::content_state::ContentState {
    fn from(old: LegacyContentStateV8) -> Self {
        Self {
            completed: old.completed,
            workshop_cutters: old.workshop_cutters,
            illness_origins: Default::default(),
            tree_material_default: None,
            tree_materials: Default::default(),
        }
    }
}
#[derive(Deserialize)]
struct LegacyBodyV8 {
    current_tick: u64,
    resources: sim_engine::legacy_v7::LegacySimResourcesV8,
    world: world::WorldSave,
    systems: Vec<(String, Vec<u8>)>,
    steward: legacy_steward::LegacyStewardState,
    content_state: LegacyContentStateV8,
}
const LEGACY_V8_BUNDLED_REVISION: &str = "catalog-v1-fnv1a64:12e812cb24fbd015:38733";

/// The identity is decoded before any selected-pack typed IDs in the body.
#[derive(Serialize, Deserialize)]
struct Envelope {
    revision: String,
    payload: Vec<u8>,
}

/// Current body decoded only after the envelope's catalog revision matches.
#[derive(Deserialize)]
struct Body {
    current_tick: u64,
    resources: SimResources,
    world: world::WorldSave,
    systems: Vec<(String, Vec<u8>)>,
    steward: sim_core::colony::steward::StewardState,
    content_state: sim_core::colony::content_state::ContentState,
    mind: sim_core::colony::mind_state::MindState,
    food: sim_core::colony::food::FoodState,
}

/// Exact format12 body, before independently persisted food state.
#[derive(Serialize, Deserialize)]
struct LegacyBodyV12 {
    current_tick: u64,
    resources: SimResources,
    world: world::WorldSave,
    systems: Vec<(String, Vec<u8>)>,
    steward: sim_core::colony::steward::StewardState,
    content_state: sim_core::colony::content_state::ContentState,
    mind: sim_core::colony::mind_state::MindState,
}

/// Exact published format11 body, before the explicit mind field.
#[derive(Serialize, Deserialize)]
struct LegacyBodyV11 {
    current_tick: u64,
    resources: SimResources,
    world: world::WorldSave,
    systems: Vec<(String, Vec<u8>)>,
    steward: sim_core::colony::steward::StewardState,
    content_state: sim_core::colony::content_state::ContentState,
}

/// Published format10 body, before independent defence state.
#[derive(Deserialize)]
struct LegacyBodyV10 {
    current_tick: u64,
    resources: SimResources,
    world: world::WorldSave,
    systems: Vec<(String, Vec<u8>)>,
    steward: legacy_steward::LegacyStewardState,
    content_state: sim_core::colony::content_state::ContentState,
}

/// Save `engine` to bytes. Two saves of one state are byte-identical.
///
/// Takes `&mut` because reading hecs's free list reserves ids; the engine's
/// world is then replaced by the world rebuilt from the saved bytes (see the
/// module docs). On error the engine may hold reserved ids and must be
/// discarded or reloaded.
pub fn save(engine: &mut SimEngine) -> Result<Vec<u8>, SaveError> {
    mind_validation::validate(&mut engine.resources).map_err(SaveError::Encode)?;
    crate::runtime::food::validate(&engine.world, &engine.resources).map_err(SaveError::Encode)?;
    let world = world::capture(&engine.world).map_err(SaveError::World)?;
    let systems = engine.system_states().map_err(SaveError::Systems)?;
    let body = BodyRef {
        current_tick: engine.current_tick(),
        resources: &engine.resources,
        world: &world,
        systems: &systems,
        steward: &engine.resources.steward,
        content_state: &engine.resources.content_state,
        mind: &engine.resources.mind,
        food: &engine.resources.food,
    };
    let mut out = Vec::with_capacity(1 << 20);
    out.extend_from_slice(&MAGIC);
    out.extend_from_slice(&FORMAT_VERSION.to_le_bytes());
    let catalog = sim_core::colony::content::installed()
        .map_err(|error| SaveError::Encode(error.to_string()))?;
    let envelope = Envelope {
        revision: catalog.revision().to_owned(),
        payload: encode(&body).map_err(SaveError::Encode)?,
    };
    out.extend_from_slice(&encode(&envelope).map_err(SaveError::Encode)?);
    engine.world = world::build(&world).map_err(SaveError::World)?;
    Ok(out)
}

/// Load an engine saved by [`save`], with the production system list
/// selected from the decoded world rules.
pub fn load(bytes: &[u8]) -> Result<SimEngine, SaveError> {
    load_with(bytes, |engine| {
        let world = if engine.resources.rules.colony.is_some() {
            sim_engine::registration::WorldKind::Colony
        } else {
            sim_engine::registration::WorldKind::Baseline
        };
        crate::register_world_runtime_systems(engine, world);
    })
}

/// [`load`] with a caller-chosen system registration; the registered
/// systems must match the saved list by position and name.
pub fn load_with(
    bytes: &[u8],
    register: impl FnOnce(&mut SimEngine),
) -> Result<SimEngine, SaveError> {
    let encoded = read_header(bytes)?;
    let version = bytes.get(MAGIC.len()..MAGIC.len() + 4);
    let legacy = version == Some(6u32.to_le_bytes().as_slice());
    let legacy_v7 = version == Some(7u32.to_le_bytes().as_slice());
    let legacy_v8 = version == Some(8u32.to_le_bytes().as_slice());
    let legacy_v9 = version == Some(9u32.to_le_bytes().as_slice());
    let legacy_v10 = version == Some(10u32.to_le_bytes().as_slice());
    let legacy_v11 = version == Some(11u32.to_le_bytes().as_slice());
    let legacy_v12 = version == Some(12u32.to_le_bytes().as_slice());
    let legacy_food = version != Some(FORMAT_VERSION.to_le_bytes().as_slice());
    if legacy || legacy_v7 || legacy_v8 || legacy_v9 {
        sim_core::colony::content::bootstrap(sim_core::colony::content::BootSelection::Bundled)
            .map_err(|error| SaveError::Decode(error.to_string()))?;
    }
    let mut body: Body = if legacy {
        let old: LegacyBody = decode(encoded).map_err(SaveError::Decode)?;
        Body {
            current_tick: old.current_tick,
            resources: old.resources.into(),
            world: old.world,
            systems: old.systems,
            steward: Default::default(),
            content_state: Default::default(),
            mind: Default::default(),
            food: Default::default(),
        }
    } else if legacy_v7 {
        let old: LegacyBodyV7 = decode(encoded).map_err(SaveError::Decode)?;
        Body {
            current_tick: old.current_tick,
            resources: old.resources.into(),
            world: old.world,
            systems: old.systems,
            steward: old.steward.into(),
            content_state: Default::default(),
            mind: Default::default(),
            food: Default::default(),
        }
    } else {
        let envelope: Envelope = decode(encoded).map_err(SaveError::Decode)?;
        let catalog = sim_core::colony::content::installed()
            .map_err(|error| SaveError::Decode(error.to_string()))?;
        if legacy_v8 {
            if envelope.revision != LEGACY_V8_BUNDLED_REVISION {
                return Err(SaveError::Decode(format!(
                    "unsupported format8 content revision {}",
                    envelope.revision
                )));
            }
            let old: LegacyBodyV8 = decode(&envelope.payload).map_err(SaveError::Decode)?;
            Body {
                current_tick: old.current_tick,
                resources: old.resources.into(),
                world: old.world,
                systems: old.systems,
                steward: old.steward.into(),
                content_state: old.content_state.into(),
                mind: Default::default(),
                food: Default::default(),
            }
        } else if legacy_v9 {
            if envelope.revision != legacy_v9::REVISION {
                return Err(SaveError::Decode(format!(
                    "unsupported format9 content revision {}",
                    envelope.revision
                )));
            }
            let old: legacy_v9::LegacyBodyV9 =
                decode(&envelope.payload).map_err(SaveError::Decode)?;
            Body {
                current_tick: old.current_tick,
                resources: old.resources,
                world: old.world,
                systems: old.systems,
                steward: old.steward.into(),
                content_state: old.content_state.into(),
                mind: Default::default(),
                food: Default::default(),
            }
        } else {
            if envelope.revision != catalog.revision()
                && !(legacy_food && food_catalog::accepts(&envelope.revision, catalog)?)
            {
                return Err(SaveError::Decode(format!(
                    "save content revision {} does not match installed {}",
                    envelope.revision,
                    catalog.revision()
                )));
            }
            if legacy_v10 {
                let old: LegacyBodyV10 = decode(&envelope.payload).map_err(SaveError::Decode)?;
                Body {
                    current_tick: old.current_tick,
                    resources: old.resources,
                    world: old.world,
                    systems: old.systems,
                    steward: old.steward.into(),
                    content_state: old.content_state,
                    mind: Default::default(),
                    food: Default::default(),
                }
            } else if legacy_v11 {
                let old: LegacyBodyV11 = decode(&envelope.payload).map_err(SaveError::Decode)?;
                Body {
                    current_tick: old.current_tick,
                    resources: old.resources,
                    world: old.world,
                    systems: old.systems,
                    steward: old.steward,
                    content_state: old.content_state,
                    mind: Default::default(),
                    food: Default::default(),
                }
            } else if legacy_v12 {
                let old: LegacyBodyV12 = decode(&envelope.payload).map_err(SaveError::Decode)?;
                Body {
                    current_tick: old.current_tick,
                    resources: old.resources,
                    world: old.world,
                    systems: old.systems,
                    steward: old.steward,
                    content_state: old.content_state,
                    mind: old.mind,
                    food: Default::default(),
                }
            } else {
                decode(&envelope.payload).map_err(SaveError::Decode)?
            }
        }
    };
    let mut resources = body.resources;
    resources.steward = body.steward;
    resources.content_state = body.content_state;
    resources.mind = body.mind;
    resources.food = body.food;
    mind_validation::validate(&mut resources).map_err(SaveError::Decode)?;
    if legacy || legacy_v7 || legacy_v8 || legacy_v9 {
        legacy_v9::migrate_trees(&mut resources).map_err(SaveError::Decode)?;
    }
    content_validation::validate(&resources).map_err(SaveError::Decode)?;
    resources.material_blocking_cache = MaterialBlockingCache::build(&resources.material_registry);
    // M3: the colony's cost map is derived (not saved); build it now so a
    // command applied before the first tick sees the same regions as an
    // uninterrupted run (code review M-1).
    crate::runtime::nav::ensure(&mut resources);
    let mut world = world::build(&body.world).map_err(SaveError::World)?;
    if legacy_food && resources.rules.colony.is_some() {
        crate::runtime::food::migration::migrate_legacy(&mut world, &mut resources)
            .map_err(SaveError::Decode)?;
        crate::runtime::settlement::fission::migrate_pending_food(
            &world,
            &mut resources,
            &mut body.systems,
        )
        .map_err(SaveError::Decode)?;
    }
    crate::runtime::food::validate(&world, &resources).map_err(SaveError::Decode)?;
    content_validation::validate_illness(&resources, &world).map_err(SaveError::Decode)?;
    let mut engine = SimEngine::new(1, 1, MaterialRegistry::new());
    let legacy_colony_brain =
        resources.rules.colony.is_some() && resources.mind.policy_source.is_none();
    // Registration must see saved policy/history and retain any legacy initialization.
    std::mem::swap(&mut engine.resources, &mut resources);
    register(&mut engine);
    std::mem::swap(&mut engine.resources, &mut resources);
    let registered = engine.system_states().map_err(SaveError::Systems)?;
    insert_legacy_systems(
        &mut body.systems,
        &registered,
        LegacySystemAdditions {
            steward: legacy,
            defence: legacy || legacy_v7 || legacy_v8 || legacy_v9 || legacy_v10,
            food: legacy_food && resources.rules.colony.is_some(),
        },
    )?;
    if legacy_colony_brain {
        let registered = engine.system_states().map_err(SaveError::Systems)?;
        migrate_colony_decision_slot(&mut body.systems, &registered)?;
    }
    engine
        .install_saved(world, resources, body.current_tick, &body.systems)
        .map_err(SaveError::Systems)?;
    Ok(engine)
}

/// Explicit additions for the known source formats, never arbitrary reconciliation.
struct LegacySystemAdditions {
    steward: bool,
    defence: bool,
    food: bool,
}

fn insert_legacy_systems(
    saved: &mut Vec<(String, Vec<u8>)>,
    registered: &[(String, Vec<u8>)],
    additions: LegacySystemAdditions,
) -> Result<(), SaveError> {
    // All additions share registration order: inserting a later slot first
    // would displace it when an earlier new slot is subsequently inserted.
    for (index, (name, state)) in registered.iter().enumerate() {
        let insert = match name.as_str() {
            "StewardSystem" => additions.steward,
            "StewardDefenceSystem" => additions.defence,
            "FoodDailySystem" => additions.food,
            _ => false,
        };
        if !insert {
            continue;
        }
        if saved.iter().any(|entry| entry.0 == *name) || index > saved.len() {
            return Err(SaveError::Systems(format!(
                "invalid legacy {name} insertion slot"
            )));
        }
        saved.insert(index, (name.clone(), state.clone()));
    }
    Ok(())
}

/// Only the known empty legacy lifecycle slot may change; all other slots stay exact.
fn migrate_colony_decision_slot(
    saved: &mut [(String, Vec<u8>)],
    registered: &[(String, Vec<u8>)],
) -> Result<(), SaveError> {
    if !registered.iter().any(|entry| entry.0 == "ColonyMindSystem") {
        return Ok(());
    }
    if saved.len() != registered.len() {
        return Err(SaveError::Systems(
            "legacy colony system count mismatch".into(),
        ));
    }
    let mut replacement = None;
    for (index, (old, new)) in saved.iter().zip(registered).enumerate() {
        if old.0 == new.0 {
            continue;
        }
        if old.0 != "AgentDecisionSystem"
            || new.0 != "ColonyMindSystem"
            || !old.1.is_empty()
            || !new.1.is_empty()
            || replacement.is_some()
        {
            return Err(SaveError::Systems(
                "legacy colony decision slot mismatch".into(),
            ));
        }
        replacement = Some(index);
    }
    let index = replacement.ok_or_else(|| {
        SaveError::Systems("colony mind system has no saved policy or legacy decision slot".into())
    })?;
    saved[index].0.clone_from(&registered[index].0);
    Ok(())
}

/// Check the magic and the version; the body follows.
fn read_header(bytes: &[u8]) -> Result<&[u8], SaveError> {
    let (magic, rest) = bytes
        .split_at_checked(MAGIC.len())
        .ok_or(SaveError::NotASave)?;
    if magic != MAGIC {
        return Err(SaveError::NotASave);
    }
    let (version, body) = rest.split_at_checked(4).ok_or(SaveError::NotASave)?;
    let mut word = [0u8; 4];
    word.copy_from_slice(version);
    let found = u32::from_le_bytes(word);
    if found != FORMAT_VERSION
        && found != 6
        && found != 7
        && found != 8
        && found != 9
        && found != 10
        && found != 11
        && found != 12
    {
        return Err(SaveError::Version {
            found,
            expected: FORMAT_VERSION,
        });
    }
    Ok(body)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn envelope_revision_is_checked_before_invalid_payload() {
        sim_core::colony::content::bootstrap(sim_core::colony::content::BootSelection::Bundled)
            .expect("bundled catalog");
        let envelope = Envelope {
            revision: "foreign-catalog".into(),
            payload: vec![255],
        };
        let mut bytes = MAGIC.to_vec();
        bytes.extend_from_slice(&FORMAT_VERSION.to_le_bytes());
        bytes.extend(encode(&envelope).expect("encode envelope"));
        let error = load(&bytes).err().expect("wrong catalog must fail");
        assert!(matches!(error, SaveError::Decode(ref message)
            if message.contains("does not match installed")));
    }

    #[test]
    fn legacy_v8_revision_allowlist_precedes_payload_decoding() {
        sim_core::colony::content::bootstrap(sim_core::colony::content::BootSelection::Bundled)
            .expect("bundled catalog");
        for revision in ["unknown-v8-pack", LEGACY_V8_BUNDLED_REVISION] {
            let envelope = Envelope {
                revision: revision.into(),
                payload: vec![255],
            };
            let mut bytes = MAGIC.to_vec();
            bytes.extend_from_slice(&8u32.to_le_bytes());
            bytes.extend(encode(&envelope).expect("envelope"));
            let error = load(&bytes).err().expect("invalid save").to_string();
            assert_eq!(
                error.contains("unsupported format8 content revision"),
                revision != "catalog-v1-fnv1a64:12e812cb24fbd015:38733"
            );
        }
    }

    #[test]
    fn numeric_recipe_slots_reject_unregistered_codes() {
        use sim_core::colony::content_state::{deserialize_craft_quality, deserialize_recipe_code};
        use std::collections::BTreeMap;

        #[derive(Deserialize)]
        struct RecipeSlot(#[serde(deserialize_with = "deserialize_recipe_code")] u32);
        #[derive(Deserialize)]
        struct Quality(
            #[serde(deserialize_with = "deserialize_craft_quality")]
            BTreeMap<(u64, u32), (u64, u64)>,
        );

        let catalog =
            sim_core::colony::content::bootstrap(sim_core::colony::content::BootSelection::Bundled)
                .expect("bundled catalog");
        let known = catalog.recipes().next().expect("bundled recipe").id.code();
        assert_eq!(
            decode::<RecipeSlot>(&encode(&known).expect("encode"))
                .expect("known code")
                .0,
            known
        );
        assert!(decode::<RecipeSlot>(&encode(&u32::MAX).expect("encode")).is_err());
        let quality = BTreeMap::from([((7u64, known), (9u64, 11u64))]);
        assert_eq!(
            decode::<Quality>(&encode(&quality).expect("encode"))
                .expect("known quality")
                .0,
            quality
        );
        let unknown = BTreeMap::from([((7u64, u32::MAX), (9u64, 11u64))]);
        assert!(decode::<Quality>(&encode(&unknown).expect("encode")).is_err());
    }

    #[test]
    fn header_checks_magic_and_version() {
        assert_eq!(read_header(b"WORLD"), Err(SaveError::NotASave));
        assert_eq!(read_header(b"XORLDSIM\x01\0\0\0"), Err(SaveError::NotASave));
        let mut other = MAGIC.to_vec();
        other.extend_from_slice(&(FORMAT_VERSION + 1).to_le_bytes());
        assert_eq!(
            read_header(&other),
            Err(SaveError::Version {
                found: FORMAT_VERSION + 1,
                expected: FORMAT_VERSION
            })
        );
        let mut ok = MAGIC.to_vec();
        ok.extend_from_slice(&FORMAT_VERSION.to_le_bytes());
        ok.push(7);
        assert_eq!(read_header(&ok), Ok(&[7u8][..]));
    }
    #[test]
    fn format9_rejects_nonallowlisted_revision_before_decoding_body() {
        let envelope = Envelope {
            revision: "unregistered-old-pack".into(),
            payload: vec![255],
        };
        let mut bytes = MAGIC.to_vec();
        bytes.extend_from_slice(&9u32.to_le_bytes());
        bytes.extend_from_slice(&encode(&envelope).expect("envelope"));
        let error = load(&bytes).err().expect("unknown legacy revision");
        assert!(
            error
                .to_string()
                .contains("unsupported format9 content revision"),
            "{error}"
        );
    }
    #[test]
    fn allowlisted_format9_revision_reaches_and_rejects_malformed_body() {
        let envelope = Envelope {
            revision: legacy_v9::REVISION.into(),
            payload: vec![255],
        };
        let mut bytes = MAGIC.to_vec();
        bytes.extend_from_slice(&9u32.to_le_bytes());
        bytes.extend_from_slice(&encode(&envelope).expect("envelope"));
        let error = load(&bytes).err().expect("malformed legacy body");
        assert!(matches!(error, SaveError::Decode(_)));
        assert!(
            !error
                .to_string()
                .contains("unsupported format9 content revision"),
            "{error}"
        );
    }
}
