//! Save files (M1.5 §7, contract clause C1): the whole simulation state as
//! bytes, and an engine rebuilt from them that ticks exactly as the saved
//! one would have.
//!
//! Format: the 8-byte magic `WORLDSIM`, [`FORMAT_VERSION`] as a little-endian
//! `u32`, then a `bincode` 1.x body (varint integers, little endian) of
//! `{ current_tick, resources, world, systems }`:
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

pub mod world;

use std::fmt;

use bincode::Options;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use sim_core::influence::MaterialBlockingCache;
use sim_core::MaterialRegistry;
use sim_engine::{SimEngine, SimResources};

/// The first eight bytes of every save file.
pub const MAGIC: [u8; 8] = *b"WORLDSIM";

/// Save format version; a file of any other version is rejected. Bump it
/// whenever a saved type or the layout changes. 2 = M2 (item specs, nodes,
/// workshops and bills, multi-line material needs, tools, skills, priorities);
/// 3 = M3 (door and dug tile columns, navigation paths, space sites, rooms);
/// 4 = M4 (`Knowledge`, seven skills, the ford column, apprenticeship orders,
/// the technology aggregate, new item, recipe, blueprint and job kinds);
/// 5 = M5 (the storyteller, `Sick`, `Apparel`, new item and chronicle kinds,
/// the chronicle's cause table, animals, fights, drafting, mood, bonds); 6 = M5.5 (`Fleeing`, neighbours,
/// migrants, caravans, the trade ledger, need samples, new chronicle kinds and the knowledge source `Migrant`).
pub const FORMAT_VERSION: u32 = 6;

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
}

/// The body after the header, as read (same fields, same order).
#[derive(Deserialize)]
struct Body {
    current_tick: u64,
    resources: SimResources,
    world: world::WorldSave,
    systems: Vec<(String, Vec<u8>)>,
}

/// Save `engine` to bytes. Two saves of one state are byte-identical.
///
/// Takes `&mut` because reading hecs's free list reserves ids; the engine's
/// world is then replaced by the world rebuilt from the saved bytes (see the
/// module docs). On error the engine may hold reserved ids and must be
/// discarded or reloaded.
pub fn save(engine: &mut SimEngine) -> Result<Vec<u8>, SaveError> {
    let world = world::capture(&engine.world).map_err(SaveError::World)?;
    let systems = engine.system_states().map_err(SaveError::Systems)?;
    let body = BodyRef {
        current_tick: engine.current_tick(),
        resources: &engine.resources,
        world: &world,
        systems: &systems,
    };
    let mut out = Vec::with_capacity(1 << 20);
    out.extend_from_slice(&MAGIC);
    out.extend_from_slice(&FORMAT_VERSION.to_le_bytes());
    out.extend_from_slice(&encode(&body).map_err(SaveError::Encode)?);
    engine.world = world::build(&world).map_err(SaveError::World)?;
    Ok(out)
}

/// Load an engine saved by [`save`], with the production system list
/// ([`crate::register_default_runtime_systems`]).
pub fn load(bytes: &[u8]) -> Result<SimEngine, SaveError> {
    load_with(bytes, crate::register_default_runtime_systems)
}

/// [`load`] with a caller-chosen system registration; the registered
/// systems must match the saved list by position and name.
pub fn load_with(
    bytes: &[u8],
    register: impl FnOnce(&mut SimEngine),
) -> Result<SimEngine, SaveError> {
    let body: Body = decode(read_header(bytes)?).map_err(SaveError::Decode)?;
    let mut resources = body.resources;
    resources.material_blocking_cache = MaterialBlockingCache::build(&resources.material_registry);
    // M3: the colony's cost map is derived (not saved); build it now so a
    // command applied before the first tick sees the same regions as an
    // uninterrupted run (code review M-1).
    crate::runtime::nav::ensure(&mut resources);
    let world = world::build(&body.world).map_err(SaveError::World)?;
    let mut engine = SimEngine::new(1, 1, MaterialRegistry::new());
    register(&mut engine);
    engine
        .install_saved(world, resources, body.current_tick, &body.systems)
        .map_err(SaveError::Systems)?;
    Ok(engine)
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
    if found != FORMAT_VERSION {
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
}
