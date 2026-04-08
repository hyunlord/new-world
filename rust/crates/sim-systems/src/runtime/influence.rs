use std::collections::HashMap;

use hecs::World;
use sim_core::components::{InfluenceEmitter, Position};
use sim_core::config;
use sim_core::{
    assign_room_ids, detect_rooms, BuildingId, ChannelId, EffectEntry, EffectPrimitive,
    EffectSource, EffectStat, EmitterRecord, EntityId, FalloffType, ResourceType, RoomId,
    RoomRole, SettlementId, TerrainType,
};
use sim_data::{DataRegistry, InfluenceEmission, StructureRequirement};
use sim_engine::{SimResources, SimSystem};

const BUILDING_TYPE_CAMPFIRE: &str = "campfire";
const BUILDING_TYPE_SHELTER: &str = "shelter";
const DEFAULT_WALL_MATERIAL_ID: &str = "oak";
const DEFAULT_ROOF_MATERIAL_ID: &str = "oak";

/// Runtime system that rebuilds spatial influence fields from world state.
#[derive(Debug, Clone)]
pub struct InfluenceRuntimeSystem {
    priority: u32,
    tick_interval: u64,
    last_structure_signature: Option<u64>,
}

impl InfluenceRuntimeSystem {
    /// Creates a new influence runtime system with deterministic cadence metadata.
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
            last_structure_signature: None,
        }
    }
}

impl SimSystem for InfluenceRuntimeSystem {
    fn name(&self) -> &'static str {
        "influence_system"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, world: &mut World, resources: &mut SimResources, _tick: u64) {
        let structure_signature = shelter_structure_signature(resources);
        if self.last_structure_signature != Some(structure_signature) {
            refresh_structural_context(resources);
            self.last_structure_signature = Some(structure_signature);
        }
        apply_room_effects(world, resources);
        let emitters = collect_runtime_emitters(world, resources);
        resources.influence_grid.replace_emitters(emitters);
    }
}

fn shelter_structure_signature(resources: &SimResources) -> u64 {
    let mut signature = 0_u64;
    let mut building_ids: Vec<BuildingId> = resources.buildings.keys().copied().collect();
    building_ids.sort_by_key(|building_id| building_id.0);
    for building_id in building_ids {
        let Some(building) = resources.buildings.get(&building_id) else {
            continue;
        };
        if !building.is_complete || building.building_type != BUILDING_TYPE_SHELTER {
            continue;
        }
        signature = signature
            .wrapping_mul(131)
            .wrapping_add(building_id.0)
            .wrapping_add(building.x as u64)
            .wrapping_add((building.y as u64) << 16);
    }
    signature
}

fn collect_runtime_emitters(world: &World, resources: &SimResources) -> Vec<EmitterRecord> {
    let mut emitters = Vec::new();
    collect_map_emitters(resources, &mut emitters);
    collect_building_emitters(resources, &mut emitters);
    collect_component_emitters(world, &mut emitters);
    collect_settlement_authority_emitters(resources, &mut emitters);
    emitters
}

fn collect_map_emitters(resources: &SimResources, emitters: &mut Vec<EmitterRecord>) {
    for y in 0..resources.map.height {
        for x in 0..resources.map.width {
            let tile = resources.map.get(x, y);
            let food_intensity = tile_food_intensity(tile);
            if food_intensity > 0.0 {
                emitters.push(EmitterRecord {
                    x,
                    y,
                    channel: ChannelId::Food,
                    radius: 0.0,
                    base_intensity: food_intensity,
                    falloff: FalloffType::Gaussian,
                    decay_rate: None,
                    tags: vec!["map_food".to_string()],
                    dirty: true,
                });
            }

            let danger_intensity = tile_danger_intensity(resources, x, y);
            if danger_intensity > 0.0 {
                emitters.push(EmitterRecord {
                    x,
                    y,
                    channel: ChannelId::Danger,
                    radius: 0.0,
                    base_intensity: danger_intensity,
                    falloff: FalloffType::Exponential,
                    decay_rate: None,
                    tags: vec!["terrain_danger".to_string()],
                    dirty: true,
                });
            }
        }
    }
}

fn tile_food_intensity(tile: &sim_core::world::Tile) -> f64 {
    let food_ratio = tile
        .resources
        .iter()
        .filter(|deposit| deposit.resource_type == ResourceType::Food && deposit.amount > 0.0)
        .map(|deposit| {
            let capacity = deposit.max_amount.max(1.0);
            (deposit.amount / capacity).clamp(0.0, 1.0)
        })
        .sum::<f64>();
    (food_ratio * config::INFLUENCE_FOOD_TILE_BASE_INTENSITY)
        .clamp(0.0, config::INFLUENCE_FOOD_TILE_BASE_INTENSITY)
}

fn tile_danger_intensity(resources: &SimResources, x: u32, y: u32) -> f64 {
    let tile = resources.map.get(x, y);
    let terrain_multiplier = match tile.terrain {
        TerrainType::DeepWater => Some(1.0),
        TerrainType::ShallowWater => Some(0.65),
        TerrainType::Mountain => Some(0.75),
        _ if !tile.passable => Some(0.5),
        _ => None,
    };
    let Some(multiplier) = terrain_multiplier else {
        return 0.0;
    };
    if !has_passable_neighbor(resources, x as i32, y as i32) {
        return 0.0;
    }
    (config::INFLUENCE_DANGER_TILE_BASE_INTENSITY * multiplier)
        .clamp(0.0, config::INFLUENCE_DANGER_TILE_BASE_INTENSITY)
}

fn has_passable_neighbor(resources: &SimResources, x: i32, y: i32) -> bool {
    [(0, -1), (1, 0), (0, 1), (-1, 0)]
        .iter()
        .copied()
        .any(|(dx, dy)| {
            let next_x = x + dx;
            let next_y = y + dy;
            resources.map.in_bounds(next_x, next_y)
                && resources.map.get(next_x as u32, next_y as u32).passable
        })
}

fn collect_building_emitters(resources: &SimResources, emitters: &mut Vec<EmitterRecord>) {
    let mut building_ids: Vec<BuildingId> = resources.buildings.keys().copied().collect();
    building_ids.sort_by_key(|building_id| building_id.0);

    for building_id in building_ids {
        let Some(building) = resources.buildings.get(&building_id) else {
            continue;
        };
        if !building.is_complete || !resources.map.in_bounds(building.x, building.y) {
            continue;
        }

        let mut emitted_any = false;
        if let Some(registry) = resources.data_registry.as_deref() {
            if let Some(registry_emissions) =
                registry.structure_completion_influence(building.building_type.as_str())
            {
                append_registry_emissions(
                    emitters,
                    building.x as u32,
                    building.y as u32,
                    registry_emissions,
                    &[building.building_type.as_str(), "registry_structure"],
                );
                emitted_any = !registry_emissions.is_empty() || emitted_any;
            }
            if let Some(furniture_id) = furniture_registry_id(building.building_type.as_str()) {
                if let Some(registry_emissions) = registry.furniture_influence_emissions(furniture_id)
                {
                    append_registry_emissions(
                        emitters,
                        building.x as u32,
                        building.y as u32,
                        registry_emissions,
                        &[building.building_type.as_str(), "registry_furniture"],
                    );
                    emitted_any = !registry_emissions.is_empty() || emitted_any;
                }
            }
        }

        if emitted_any {
            continue;
        }

        match building.building_type.as_str() {
            BUILDING_TYPE_CAMPFIRE => {
                emitters.push(EmitterRecord {
                    x: building.x as u32,
                    y: building.y as u32,
                    channel: ChannelId::Warmth,
                    radius: f64::from(config::BUILDING_CAMPFIRE_RADIUS.max(1)),
                    base_intensity: config::WARMTH_CAMPFIRE_EMITTER_INTENSITY,
                    falloff: FalloffType::Linear,
                    decay_rate: None,
                    tags: vec!["campfire".to_string(), "fallback".to_string()],
                    dirty: true,
                });
                emitters.push(EmitterRecord {
                    x: building.x as u32,
                    y: building.y as u32,
                    channel: ChannelId::Social,
                    radius: config::INFLUENCE_SOCIAL_DEFAULT_RADIUS.max(1.0),
                    base_intensity: config::INFLUENCE_CAMPFIRE_SOCIAL_INTENSITY,
                    falloff: FalloffType::Linear,
                    decay_rate: None,
                    tags: vec!["campfire".to_string(), "fallback".to_string(), "social".to_string()],
                    dirty: true,
                });
                emitters.push(EmitterRecord {
                    x: building.x as u32,
                    y: building.y as u32,
                    channel: ChannelId::Danger,
                    radius: f64::from(config::BUILDING_CAMPFIRE_RADIUS.max(1)),
                    base_intensity: config::INFLUENCE_CAMPFIRE_DANGER_INTENSITY,
                    falloff: FalloffType::Exponential,
                    decay_rate: None,
                    tags: vec!["campfire".to_string(), "fallback".to_string(), "danger".to_string()],
                    dirty: true,
                });
            }
            BUILDING_TYPE_SHELTER => emitters.push(EmitterRecord {
                x: building.x as u32,
                y: building.y as u32,
                channel: ChannelId::Warmth,
                radius: f64::from(config::BUILDING_SHELTER_RADIUS.max(1)),
                base_intensity: config::INFLUENCE_SHELTER_BASE_INTENSITY,
                falloff: FalloffType::Gaussian,
                decay_rate: None,
                tags: vec!["shelter".to_string(), "fallback".to_string()],
                dirty: true,
            }),
            _ => {}
        }
    }
}

fn append_registry_emissions(
    emitters: &mut Vec<EmitterRecord>,
    x: u32,
    y: u32,
    emissions: &[InfluenceEmission],
    extra_tags: &[&str],
) {
    for emission in emissions {
        let Some(channel) = ChannelId::from_key(&emission.channel) else {
            continue;
        };
        let mut tags: Vec<String> = extra_tags.iter().map(|tag| (*tag).to_string()).collect();
        tags.push(channel.key().to_string());
        emitters.push(EmitterRecord {
            x,
            y,
            channel,
            radius: emission.radius.max(0.0),
            base_intensity: emission.intensity.max(0.0),
            falloff: default_falloff_for_channel(channel),
            decay_rate: None,
            tags,
            dirty: true,
        });
    }
}

fn default_falloff_for_channel(channel: ChannelId) -> FalloffType {
    match channel {
        ChannelId::Food => FalloffType::Gaussian,
        ChannelId::Danger | ChannelId::Disease | ChannelId::Noise => FalloffType::Exponential,
        _ => FalloffType::Linear,
    }
}

fn furniture_registry_id(building_type: &str) -> Option<&'static str> {
    match building_type {
        BUILDING_TYPE_CAMPFIRE => Some("fire_pit"),
        _ => None,
    }
}

fn preferred_structure_material_tag(settlement: Option<&sim_core::Settlement>) -> &'static str {
    if settlement
        .map(|known| known.stockpile_stone > known.stockpile_wood)
        .unwrap_or(false)
    {
        "stone"
    } else {
        "wood"
    }
}

fn structure_requirement_tags(
    registry: Option<&DataRegistry>,
    structure_id: &str,
    predicate: impl Fn(&StructureRequirement) -> Option<&Vec<String>>,
    fallback_tag: &str,
) -> Vec<String> {
    registry
        .and_then(|loaded| loaded.structure_def(structure_id))
        .and_then(|structure| {
            structure
                .required_components
                .iter()
                .find_map(predicate)
                .filter(|tags| !tags.is_empty())
                .cloned()
        })
        .unwrap_or_else(|| vec![fallback_tag.to_string()])
}

fn select_material_for_tags(
    registry: &DataRegistry,
    required_tags: &[String],
    preferred_tag: Option<&str>,
) -> Option<String> {
    let mut material_ids: Vec<&str> = registry.materials.keys().map(|id| id.as_str()).collect();
    material_ids.sort_unstable();

    let mut fallback_match: Option<String> = None;
    for material_id in material_ids {
        let Some(material) = registry.materials.get(material_id) else {
            continue;
        };
        if !required_tags.iter().all(|tag| material.tags.contains(tag)) {
            continue;
        }
        if preferred_tag
            .map(|tag| material.tags.contains(tag))
            .unwrap_or(false)
        {
            return Some(material_id.to_string());
        }
        if fallback_match.is_none() {
            fallback_match = Some(material_id.to_string());
        }
    }
    fallback_match
}

fn resolve_shelter_wall_material(resources: &SimResources, settlement_id: SettlementId) -> String {
    let registry = resources.data_registry.as_deref();
    let wall_tags = structure_requirement_tags(
        registry,
        BUILDING_TYPE_SHELTER,
        |requirement| match requirement {
            StructureRequirement::Wall { tags, .. } => Some(tags),
            _ => None,
        },
        "building_material",
    );
    let preferred_tag =
        preferred_structure_material_tag(resources.settlements.get(&settlement_id));
    registry
        .and_then(|loaded| select_material_for_tags(loaded, &wall_tags, Some(preferred_tag)))
        .unwrap_or_else(|| DEFAULT_WALL_MATERIAL_ID.to_string())
}

fn resolve_shelter_roof_material(resources: &SimResources) -> String {
    let registry = resources.data_registry.as_deref();
    let roof_tags = structure_requirement_tags(
        registry,
        BUILDING_TYPE_SHELTER,
        |requirement| match requirement {
            StructureRequirement::Roof { tags } => Some(tags),
            _ => None,
        },
        "roof_material",
    );
    registry
        .and_then(|loaded| select_material_for_tags(loaded, &roof_tags, Some("wood")))
        .unwrap_or_else(|| DEFAULT_ROOF_MATERIAL_ID.to_string())
}

fn wall_hp_from_material(registry: Option<&DataRegistry>, material_id: &str) -> f64 {
    registry
        .and_then(|loaded| loaded.material_wall_hit_points(material_id))
        .unwrap_or(10.0)
}

/// Authority: each settlement center emits authority influence scaled by population.
fn collect_settlement_authority_emitters(
    resources: &SimResources,
    emitters: &mut Vec<EmitterRecord>,
) {
    for settlement in resources.settlements.values() {
        if settlement.members.is_empty() {
            continue;
        }
        let member_count = settlement.members.len() as f64;
        let radius =
            (8.0 + member_count.sqrt() * 3.0).min(f64::from(config::INFLUENCE_AUTHORITY_MAX_RADIUS));
        let intensity = (member_count * config::INFLUENCE_AUTHORITY_PER_MEMBER)
            .clamp(0.0, config::INFLUENCE_AUTHORITY_MAX_INTENSITY);
        emitters.push(EmitterRecord {
            x: settlement.x.max(0) as u32,
            y: settlement.y.max(0) as u32,
            channel: ChannelId::Authority,
            base_intensity: intensity,
            radius,
            falloff: FalloffType::Gaussian,
            decay_rate: None,
            tags: Vec::new(),
            dirty: true,
        });
    }
}

fn collect_component_emitters(world: &World, emitters: &mut Vec<EmitterRecord>) {
    let mut component_emitters: Vec<(u64, EmitterRecord)> = world
        .query::<(&Position, &InfluenceEmitter)>()
        .iter()
        .filter_map(|(entity, (position, emitter))| {
            if !emitter.enabled {
                return None;
            }
            let tile_x = position.tile_x();
            let tile_y = position.tile_y();
            if tile_x < 0 || tile_y < 0 {
                return None;
            }
            Some((entity.id() as u64, emitter.to_record(tile_x as u32, tile_y as u32)))
        })
        .collect();
    component_emitters.sort_by_key(|(entity_id, _)| *entity_id);
    emitters.extend(component_emitters.into_iter().map(|(_, emitter)| emitter));
}

fn refresh_structural_context(resources: &mut SimResources) {
    resources.tile_grid.clear();
    resources.influence_grid.clear_wall_blocking();

    let mut building_ids: Vec<BuildingId> = resources.buildings.keys().copied().collect();
    building_ids.sort_by_key(|building_id| building_id.0);
    for building_id in building_ids {
        let Some(building) = resources.buildings.get(&building_id) else {
            continue;
        };
        if !building.is_complete {
            continue;
        }
        if building.building_type == BUILDING_TYPE_SHELTER {
            // building.x/y is the top-left corner; compute the center of the
            // footprint so the wall ring aligns with the reserved tiles.
            let center_x = building.x + (building.width as i32) / 2;
            let center_y = building.y + (building.height as i32) / 2;
            stamp_shelter_structure(resources, center_x, center_y, building.settlement_id);
        }
    }

    let rooms = detect_rooms(&resources.tile_grid);
    assign_room_ids(&mut resources.tile_grid, &rooms);
    resources.rooms = rooms;
    assign_room_roles_from_buildings(resources);
    apply_wall_blocking_from_tile_grid(resources);
}

fn stamp_shelter_structure(
    resources: &mut SimResources,
    center_x: i32,
    center_y: i32,
    settlement_id: SettlementId,
) {
    if !resources.map.in_bounds(center_x, center_y) {
        return;
    }
    let wall_material = resolve_shelter_wall_material(resources, settlement_id);
    let floor_material = wall_material.clone();
    let roof_material = resolve_shelter_roof_material(resources);
    let wall_hp = wall_hp_from_material(resources.data_registry.as_deref(), wall_material.as_str());

    let wall_radius = config::BUILDING_SHELTER_WALL_RING_RADIUS.max(1);
    let interior_radius = wall_radius - 1;

    // 1. Stamp floor + roof on ALL interior tiles (inside the wall ring).
    //    For wall_radius=2 this covers a 3x3 interior = 9 floor tiles.
    //    For wall_radius=1 this covers a 1x1 interior = just the center.
    for offset_y in -interior_radius..=interior_radius {
        for offset_x in -interior_radius..=interior_radius {
            let tile_x = center_x + offset_x;
            let tile_y = center_y + offset_y;
            if !resources.map.in_bounds(tile_x, tile_y) {
                continue;
            }
            let tx = tile_x as u32;
            let ty = tile_y as u32;
            resources.tile_grid.set_floor(tx, ty, floor_material.clone());
            resources.tile_grid.set_roof(tx, ty, roof_material.clone());
        }
    }

    // 2. Stamp perimeter walls, marking the door tile as a door instead of wall.
    for offset_y in -wall_radius..=wall_radius {
        for offset_x in -wall_radius..=wall_radius {
            let is_perimeter =
                offset_x.abs() == wall_radius || offset_y.abs() == wall_radius;
            if !is_perimeter {
                continue;
            }
            let tile_x = center_x + offset_x;
            let tile_y = center_y + offset_y;
            if !resources.map.in_bounds(tile_x, tile_y) {
                continue;
            }
            let tx = tile_x as u32;
            let ty = tile_y as u32;
            if offset_x == config::BUILDING_SHELTER_DOOR_OFFSET_X
                && offset_y == config::BUILDING_SHELTER_DOOR_OFFSET_Y
            {
                resources.tile_grid.set_door(tx, ty);
                continue;
            }
            resources.tile_grid.set_wall(
                tx,
                ty,
                wall_material.clone(),
                wall_hp,
            );
        }
    }
}

fn apply_wall_blocking_from_tile_grid(resources: &mut SimResources) {
    let registry = resources.data_registry.as_deref();
    let default_wall_block = registry
        .and_then(|loaded| loaded.material_wall_blocking_hint(DEFAULT_WALL_MATERIAL_ID))
        .unwrap_or(config::BUILDING_SHELTER_WALL_BLOCK);
    let width = resources.map.width;
    let height = resources.map.height;

    for y in 0..height {
        for x in 0..width {
            let tile = resources.tile_grid.get(x, y);
            let blocking = tile
                .wall_material
                .as_deref()
                .map(|material_id| {
                    registry
                        .and_then(|loaded| loaded.material_wall_blocking_hint(material_id))
                        .unwrap_or(default_wall_block)
                })
                .unwrap_or(0.0);
            if blocking > 0.0 {
                resources
                    .influence_grid
                    .set_wall_blocking(x, y, blocking);
            }
        }
    }
}

/// Assigns room roles based on which buildings occupy each room's tiles.
/// Called after `detect_rooms` + `assign_room_ids` in `refresh_structural_context`.
///
/// Exposed as `pub` so fixture tests can invoke the real role-assignment
/// pipeline after populating `resources.rooms` and `resources.buildings`
/// manually.
pub fn assign_room_roles_from_buildings(resources: &mut SimResources) {
    let mut role_votes: HashMap<RoomId, Vec<&'static str>> = HashMap::new();

    let mut building_ids: Vec<BuildingId> = resources.buildings.keys().copied().collect();
    building_ids.sort_by_key(|id| id.0);
    for building_id in building_ids {
        let Some(building) = resources.buildings.get(&building_id) else {
            continue;
        };
        if !building.is_complete {
            continue;
        }
        let bx = building.x.max(0) as u32;
        let by = building.y.max(0) as u32;
        let (grid_w, grid_h) = resources.tile_grid.dimensions();
        if bx >= grid_w || by >= grid_h {
            continue;
        }
        let Some(room_id) = resources.tile_grid.get(bx, by).room_id else {
            continue;
        };
        let role = match building.building_type.as_str() {
            BUILDING_TYPE_CAMPFIRE => Some("hearth"),
            BUILDING_TYPE_SHELTER => Some("shelter"),
            "stockpile" => Some("storage"),
            "workbench" => Some("crafting"),
            _ => None,
        };
        if let Some(role_str) = role {
            role_votes.entry(room_id).or_default().push(role_str);
        }
    }

    for room in &mut resources.rooms {
        if !room.enclosed {
            room.role = RoomRole::Unknown;
            continue;
        }
        room.role = match role_votes.get(&room.id) {
            None => RoomRole::Shelter,
            Some(votes) => majority_role(votes),
        };
    }
}

fn majority_role(votes: &[&str]) -> RoomRole {
    let mut counts: HashMap<&str, usize> = HashMap::new();
    for &vote in votes {
        *counts.entry(vote).or_default() += 1;
    }
    match counts.into_iter().max_by_key(|(_, c)| *c).map(|(r, _)| r) {
        Some("hearth") => RoomRole::Hearth,
        Some("shelter") => RoomRole::Shelter,
        Some("storage") => RoomRole::Storage,
        Some("crafting") => RoomRole::Crafting,
        _ => RoomRole::Shelter,
    }
}

/// Enqueues per-tick stat bonuses for agents standing inside enclosed rooms.
/// Uses EffectQueue so CausalLog records the source of each bonus.
///
/// Exposed as `pub` so fixture tests can invoke the real room-effect pass
/// directly without going through the full InfluenceRuntimeSystem.
pub fn apply_room_effects(world: &World, resources: &mut SimResources) {
    if resources.rooms.is_empty() {
        return;
    }

    // Build room lookup: tile position → (role, enclosed)
    let room_map: HashMap<(u32, u32), (RoomRole, bool)> = resources
        .rooms
        .iter()
        .flat_map(|room| {
            room.tiles
                .iter()
                .map(move |&tile| (tile, (room.role, room.enclosed)))
        })
        .collect();

    let (grid_w, grid_h) = resources.tile_grid.dimensions();
    let mut entries: Vec<EffectEntry> = Vec::new();

    for (entity, position) in world.query::<&Position>().iter() {
        let tx = position.tile_x().max(0) as u32;
        let ty = position.tile_y().max(0) as u32;
        if tx >= grid_w || ty >= grid_h {
            continue;
        }
        let Some(&(role, enclosed)) = room_map.get(&(tx, ty)) else {
            continue;
        };
        if !enclosed {
            continue;
        }
        let entity_id = EntityId(entity.id() as u64);
        match role {
            RoomRole::Shelter => {
                entries.push(EffectEntry {
                    entity: entity_id,
                    effect: EffectPrimitive::AddStat {
                        stat: EffectStat::Safety,
                        amount: 0.02,
                    },
                    source: EffectSource {
                        system: "room_effect".to_string(),
                        kind: "shelter_safety".to_string(),
                    },
                });
            }
            RoomRole::Hearth => {
                entries.push(EffectEntry {
                    entity: entity_id,
                    effect: EffectPrimitive::AddStat {
                        stat: EffectStat::Warmth,
                        amount: 0.03,
                    },
                    source: EffectSource {
                        system: "room_effect".to_string(),
                        kind: "hearth_warmth".to_string(),
                    },
                });
            }
            RoomRole::Unknown | RoomRole::Storage | RoomRole::Crafting => {}
        }
    }

    for entry in entries {
        resources.effect_queue.push(entry);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim_core::components::Position;
    use sim_core::config::GameConfig;
    use sim_core::{Building, GameCalendar, SettlementId, WorldMap};

    fn resources() -> SimResources {
        let config = GameConfig::default();
        let calendar = GameCalendar::new(&config);
        SimResources::new(calendar, WorldMap::new(12, 12, 77), 99)
    }

    fn registry_data_path() -> std::path::PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../sim-data/data")
            .canonicalize()
            .expect("registry data path should resolve")
    }

    #[test]
    fn influence_runtime_system_builds_food_and_danger_emitters() {
        let mut world = World::new();
        let mut resources = resources();
        resources.map.get_mut(2, 2).resources.push(sim_core::world::TileResource {
            resource_type: ResourceType::Food,
            amount: 3.0,
            max_amount: 3.0,
            regen_rate: 0.0,
        });
        resources.map.get_mut(4, 4).terrain = TerrainType::DeepWater;
        resources.map.get_mut(4, 4).passable = false;

        let mut system =
            InfluenceRuntimeSystem::new(config::INFLUENCE_SYSTEM_PRIORITY, config::INFLUENCE_SYSTEM_INTERVAL);
        system.run(&mut world, &mut resources, 1);
        resources.influence_grid.tick_update();

        assert!(resources.influence_grid.sample(2, 2, ChannelId::Food) > 0.0);
        assert!(resources.influence_grid.sample(3, 4, ChannelId::Danger) > 0.0);
        assert_eq!(resources.influence_grid.active_emitter_count(), 2);
    }

    #[test]
    fn influence_runtime_system_refreshes_food_emitters_without_stale_leaks() {
        let mut world = World::new();
        let mut resources = resources();
        resources.map.get_mut(2, 2).resources.push(sim_core::world::TileResource {
            resource_type: ResourceType::Food,
            amount: 3.0,
            max_amount: 3.0,
            regen_rate: 0.0,
        });

        let mut system =
            InfluenceRuntimeSystem::new(config::INFLUENCE_SYSTEM_PRIORITY, config::INFLUENCE_SYSTEM_INTERVAL);
        system.run(&mut world, &mut resources, 1);
        assert_eq!(resources.influence_grid.active_emitter_count(), 1);

        resources.map.get_mut(2, 2).resources.clear();
        system.run(&mut world, &mut resources, 2);
        assert_eq!(resources.influence_grid.active_emitter_count(), 0);
    }

    #[test]
    fn influence_runtime_system_food_signal_is_stronger_near_source_than_far() {
        let mut world = World::new();
        let mut resources = resources();
        resources.map.get_mut(2, 2).resources.push(sim_core::world::TileResource {
            resource_type: ResourceType::Food,
            amount: 3.0,
            max_amount: 3.0,
            regen_rate: 0.0,
        });

        let mut system =
            InfluenceRuntimeSystem::new(config::INFLUENCE_SYSTEM_PRIORITY, config::INFLUENCE_SYSTEM_INTERVAL);
        system.run(&mut world, &mut resources, 1);
        resources.influence_grid.tick_update();

        let near = resources.influence_grid.sample(3, 2, ChannelId::Food);
        let far = resources.influence_grid.sample(7, 2, ChannelId::Food);
        assert!(near > far);
    }

    #[test]
    fn influence_runtime_system_builds_shelter_rooms_and_walls() {
        let mut world = World::new();
        let mut resources = resources();
        resources.buildings.insert(
            BuildingId(9),
            Building {
                id: BuildingId(9),
                building_type: "shelter".to_string(),
                settlement_id: SettlementId(1),
                x: 5,
                y: 5,
                construction_progress: 1.0,
                is_complete: true,
                construction_started_tick: 0,
                width: 1,
                height: 1,
                condition: 1.0,
            },
        );

        let mut system =
            InfluenceRuntimeSystem::new(config::INFLUENCE_SYSTEM_PRIORITY, config::INFLUENCE_SYSTEM_INTERVAL);
        system.run(&mut world, &mut resources, 1);

        // P2-B2: with wall_radius=2, a 5x5 shelter centered on (5,5) has a
        // 3x3 interior (4..=6, 4..=6) and perimeter walls at 3..=7. The door
        // sits at (5, 7). This forms an enclosed room of 9 floor tiles.
        assert_eq!(resources.rooms.len(), 1);
        assert!(
            resources.rooms[0].enclosed,
            "5x5 shelter with sealed door should produce enclosed room"
        );
        assert_eq!(resources.rooms[0].tiles.len(), 9);
        assert_eq!(resources.tile_grid.get(5, 5).room_id, Some(resources.rooms[0].id));
        // East wall at (7, 5) should apply the configured wall blocking.
        assert!(
            (resources.influence_grid.wall_blocking_at(7, 5) - config::BUILDING_SHELTER_WALL_BLOCK)
                .abs()
                < 1e-6
        );
        // Door tile (5, 7) should NOT stamp wall blocking (doors are passable).
        assert_eq!(resources.influence_grid.wall_blocking_at(5, 7), 0.0);
        assert!(resources.tile_grid.get(5, 7).is_door);
    }

    #[test]
    fn influence_runtime_system_uses_registry_and_component_emitters() {
        let mut world = World::new();
        let mut resources = resources();
        resources.data_registry = Some(std::sync::Arc::new(
            sim_data::DataRegistry::load_from_directory(&registry_data_path())
                .expect("registry should load for influence test"),
        ));
        resources.buildings.insert(
            BuildingId(10),
            Building {
                id: BuildingId(10),
                building_type: "campfire".to_string(),
                settlement_id: SettlementId(1),
                x: 4,
                y: 4,
                construction_progress: 1.0,
                is_complete: true,
                construction_started_tick: 0,
                width: 1,
                height: 1,
                condition: 1.0,
            },
        );
        world.spawn((
            Position::new(8, 8),
            InfluenceEmitter {
                channel: ChannelId::Social,
                radius: 3.0,
                base_intensity: 0.6,
                falloff: FalloffType::Linear,
                decay_rate: None,
                tags: vec!["test".to_string()],
                enabled: true,
            },
        ));

        let mut system =
            InfluenceRuntimeSystem::new(config::INFLUENCE_SYSTEM_PRIORITY, config::INFLUENCE_SYSTEM_INTERVAL);
        system.run(&mut world, &mut resources, 1);
        resources.influence_grid.tick_update();

        assert!(resources.influence_grid.sample(4, 4, ChannelId::Warmth) > 0.0);
        assert!(resources.influence_grid.sample(4, 4, ChannelId::Danger) > 0.0);
        assert!(resources.influence_grid.sample(8, 8, ChannelId::Social) > 0.0);
        assert!(resources.influence_grid.sample(4, 4, ChannelId::Social) > 0.0);
    }

    #[test]
    fn influence_runtime_system_registry_campfire_uses_registry_social_emission() {
        let mut world = World::new();
        let mut resources = resources();
        resources.data_registry = Some(std::sync::Arc::new(
            sim_data::DataRegistry::load_from_directory(&registry_data_path())
                .expect("registry should load for influence test"),
        ));
        resources.buildings.insert(
            BuildingId(11),
            Building {
                id: BuildingId(11),
                building_type: "campfire".to_string(),
                settlement_id: SettlementId(1),
                x: 4,
                y: 4,
                construction_progress: 1.0,
                is_complete: true,
                construction_started_tick: 0,
                width: 1,
                height: 1,
                condition: 1.0,
            },
        );

        let mut system =
            InfluenceRuntimeSystem::new(config::INFLUENCE_SYSTEM_PRIORITY, config::INFLUENCE_SYSTEM_INTERVAL);
        system.run(&mut world, &mut resources, 1);
        resources.influence_grid.tick_update();

        assert!(resources.influence_grid.sample(4, 4, ChannelId::Light) > 0.0);
        assert!(resources.influence_grid.sample(4, 4, ChannelId::Social) > 0.0);
    }

    #[test]
    fn influence_runtime_system_fallback_campfire_emits_social_and_danger() {
        let mut world = World::new();
        let mut resources = resources();
        resources.buildings.insert(
            BuildingId(12),
            Building {
                id: BuildingId(12),
                building_type: "campfire".to_string(),
                settlement_id: SettlementId(1),
                x: 4,
                y: 4,
                construction_progress: 1.0,
                is_complete: true,
                construction_started_tick: 0,
                width: 1,
                height: 1,
                condition: 1.0,
            },
        );

        let mut system =
            InfluenceRuntimeSystem::new(config::INFLUENCE_SYSTEM_PRIORITY, config::INFLUENCE_SYSTEM_INTERVAL);
        system.run(&mut world, &mut resources, 1);
        resources.influence_grid.tick_update();

        assert!(resources.influence_grid.sample(4, 4, ChannelId::Social) > 0.0);
        assert!(resources.influence_grid.sample(4, 4, ChannelId::Danger) > 0.0);
        assert_eq!(resources.influence_grid.sample(4, 4, ChannelId::Light), 0.0);
    }

    #[test]
    fn influence_runtime_system_campfire_social_attenuates_with_distance() {
        let mut world = World::new();
        let mut resources = resources();
        resources.buildings.insert(
            BuildingId(16),
            Building {
                id: BuildingId(16),
                building_type: "campfire".to_string(),
                settlement_id: SettlementId(1),
                x: 4,
                y: 4,
                construction_progress: 1.0,
                is_complete: true,
                construction_started_tick: 0,
                width: 1,
                height: 1,
                condition: 1.0,
            },
        );

        let mut system =
            InfluenceRuntimeSystem::new(config::INFLUENCE_SYSTEM_PRIORITY, config::INFLUENCE_SYSTEM_INTERVAL);
        system.run(&mut world, &mut resources, 1);
        resources.influence_grid.tick_update();

        let near_signal = resources.influence_grid.sample(5, 4, ChannelId::Social);
        let far_signal = resources.influence_grid.sample(8, 4, ChannelId::Social);
        assert!(near_signal > far_signal);
    }

    #[test]
    fn influence_runtime_system_campfire_danger_attenuates_with_distance() {
        let mut world = World::new();
        let mut resources = resources();
        resources.buildings.insert(
            BuildingId(13),
            Building {
                id: BuildingId(13),
                building_type: "campfire".to_string(),
                settlement_id: SettlementId(1),
                x: 4,
                y: 4,
                construction_progress: 1.0,
                is_complete: true,
                construction_started_tick: 0,
                width: 1,
                height: 1,
                condition: 1.0,
            },
        );

        let mut system =
            InfluenceRuntimeSystem::new(config::INFLUENCE_SYSTEM_PRIORITY, config::INFLUENCE_SYSTEM_INTERVAL);
        system.run(&mut world, &mut resources, 1);
        resources.influence_grid.tick_update();

        let near_signal = resources.influence_grid.sample(5, 4, ChannelId::Danger);
        let far_signal = resources.influence_grid.sample(8, 4, ChannelId::Danger);
        assert!(near_signal > far_signal);
    }

    #[test]
    fn influence_runtime_system_walls_reduce_campfire_danger_signal() {
        let mut world = World::new();
        let mut resources = resources();
        resources.buildings.insert(
            BuildingId(14),
            Building {
                id: BuildingId(14),
                building_type: "shelter".to_string(),
                settlement_id: SettlementId(1),
                x: 5,
                y: 5,
                construction_progress: 1.0,
                is_complete: true,
                construction_started_tick: 0,
                width: 1,
                height: 1,
                condition: 1.0,
            },
        );
        resources.buildings.insert(
            BuildingId(15),
            Building {
                id: BuildingId(15),
                building_type: "campfire".to_string(),
                settlement_id: SettlementId(1),
                x: 5,
                y: 5,
                construction_progress: 1.0,
                is_complete: true,
                construction_started_tick: 0,
                width: 1,
                height: 1,
                condition: 1.0,
            },
        );

        let mut system =
            InfluenceRuntimeSystem::new(config::INFLUENCE_SYSTEM_PRIORITY, config::INFLUENCE_SYSTEM_INTERVAL);
        system.run(&mut world, &mut resources, 1);
        resources.influence_grid.tick_update();

        let open_signal = resources.influence_grid.sample(5, 6, ChannelId::Danger);
        let blocked_signal = resources.influence_grid.sample(7, 5, ChannelId::Danger);
        assert!(open_signal > blocked_signal);
    }

    #[test]
    fn influence_runtime_system_wall_hp_derived_from_material_properties() {
        let mut world = World::new();
        let mut resources = resources();
        resources.data_registry = Some(std::sync::Arc::new(
            sim_data::DataRegistry::load_from_directory(&registry_data_path())
                .expect("registry should load for influence test"),
        ));
        let settlement_id = SettlementId(1);
        let mut settlement =
            sim_core::Settlement::new(settlement_id, "alpha".to_string(), 5, 5, 0);
        settlement.stockpile_wood = 4.0;
        settlement.stockpile_stone = 0.0;
        resources.settlements.insert(settlement_id, settlement);
        resources.buildings.insert(
            BuildingId(17),
            Building {
                id: BuildingId(17),
                building_type: BUILDING_TYPE_SHELTER.to_string(),
                settlement_id,
                x: 5,
                y: 5,
                construction_progress: 1.0,
                is_complete: true,
                construction_started_tick: 0,
                width: 1,
                height: 1,
                condition: 1.0,
            },
        );

        let mut system = InfluenceRuntimeSystem::new(
            config::INFLUENCE_SYSTEM_PRIORITY,
            config::INFLUENCE_SYSTEM_INTERVAL,
        );
        system.run(&mut world, &mut resources, 1);

        // P2-B2: wall ring radius=2 — walls sit on the 5x5 perimeter. Probe
        // the east wall at (7, 5), which is a perimeter position and not the
        // door (door is at offset (+0, +2) = (5, 7)).
        let stamped_material = resources.tile_grid.get(7, 5).wall_material.clone();
        let expected_hp = resources
            .data_registry
            .as_ref()
            .and_then(|registry| {
                stamped_material
                    .as_deref()
                    .and_then(|material_id| registry.material_wall_hit_points(material_id))
            })
            .expect("selected wall material should derive hit points");

        assert!(stamped_material.is_some());
        assert!((resources.tile_grid.get(7, 5).wall_hp - expected_hp).abs() < 1e-6);
    }

    #[test]
    fn influence_runtime_system_structure_completion_influence_stamps_grid() {
        let mut world = World::new();
        let mut resources = resources();
        resources.data_registry = Some(std::sync::Arc::new(
            sim_data::DataRegistry::load_from_directory(&registry_data_path())
                .expect("registry should load for influence test"),
        ));
        resources.buildings.insert(
            BuildingId(18),
            Building {
                id: BuildingId(18),
                building_type: BUILDING_TYPE_SHELTER.to_string(),
                settlement_id: SettlementId(1),
                x: 5,
                y: 5,
                construction_progress: 1.0,
                is_complete: true,
                construction_started_tick: 0,
                width: 1,
                height: 1,
                condition: 1.0,
            },
        );

        let mut system = InfluenceRuntimeSystem::new(
            config::INFLUENCE_SYSTEM_PRIORITY,
            config::INFLUENCE_SYSTEM_INTERVAL,
        );
        system.run(&mut world, &mut resources, 1);
        resources.influence_grid.tick_update();

        assert!(resources.influence_grid.sample(5, 5, ChannelId::Warmth) > 0.0);
    }
}
