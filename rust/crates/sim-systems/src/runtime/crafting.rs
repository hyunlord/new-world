use std::cmp::Ordering;
use std::collections::BTreeMap;

use hecs::World;
use sim_core::components::{Behavior, Identity, Inventory};
use sim_core::config;
use sim_core::{
    ActionType, CausalEvent, CauseRef, EntityId, ItemDerivedStats, ItemId, ItemInstance, ItemOwner,
    Settlement,
};
use sim_data::defs::RecipeDef;
use sim_data::DataRegistry;
use sim_engine::{SimResources, SimSystem};

#[derive(Debug, Clone)]
pub struct CraftingRuntimeSystem {
    priority: u32,
    tick_interval: u64,
}

#[derive(Debug, Clone)]
struct CraftSelection {
    recipe_id: String,
    material_id: String,
    duration_ticks: i32,
}

impl CraftingRuntimeSystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
        }
    }
}

impl SimSystem for CraftingRuntimeSystem {
    fn name(&self) -> &'static str {
        "crafting_system"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, world: &mut World, resources: &mut SimResources, _tick: u64) {
        let Some(registry_arc) = resources.data_registry.clone() else {
            return;
        };
        let registry = registry_arc.as_ref();
        let mut query = world.query::<(&mut Behavior, &Identity, Option<&Inventory>)>();

        for (_, (behavior, identity, inventory_opt)) in &mut query {
            if behavior.current_action != ActionType::Craft {
                continue;
            }
            if behavior.craft_recipe_id.is_some() {
                continue;
            }
            if inventory_has_tool(inventory_opt, resources) {
                clear_craft_behavior(behavior);
                continue;
            }

            let Some(settlement_id) = identity.settlement_id else {
                clear_craft_behavior(behavior);
                continue;
            };
            let Some(settlement) = resources.settlements.get_mut(&settlement_id) else {
                clear_craft_behavior(behavior);
                continue;
            };

            let Some(selection) = select_best_recipe(registry, settlement) else {
                clear_craft_behavior(behavior);
                continue;
            };
            let Some(recipe) = registry.recipes.get(selection.recipe_id.as_str()) else {
                clear_craft_behavior(behavior);
                continue;
            };

            if !deduct_recipe_costs(settlement, recipe.as_ref()) {
                clear_craft_behavior(behavior);
                continue;
            }

            behavior.craft_recipe_id = Some(selection.recipe_id);
            behavior.craft_material_id = Some(selection.material_id);
            behavior.action_duration = selection.duration_ticks;
            behavior.action_timer = selection.duration_ticks;
            behavior.action_progress = 0.0;
        }
    }
}

pub(crate) fn inventory_has_tool(
    inventory_opt: Option<&Inventory>,
    resources: &SimResources,
) -> bool {
    let Some(inventory) = inventory_opt else {
        return false;
    };
    inventory.items.iter().any(|item_id| {
        resources
            .item_store
            .get(*item_id)
            .map(|item| is_tool_template(item.template_id.as_str()))
            .unwrap_or(false)
    })
}

pub fn action_tool_tag(action: ActionType) -> Option<&'static str> {
    match action {
        ActionType::Forage | ActionType::GatherHerbs | ActionType::Hunt => Some("knife"),
        ActionType::GatherWood => Some("axe"),
        _ => None,
    }
}

pub fn find_best_tool(
    inventory: &Inventory,
    item_store: &sim_core::ItemStore,
    tool_tag: &str,
) -> Option<(ItemId, ItemDerivedStats)> {
    let mut best: Option<(ItemId, ItemDerivedStats)> = None;

    for &item_id in &inventory.items {
        let Some(item) = item_store.get(item_id) else {
            continue;
        };
        if item.is_broken() || item.template_id != tool_tag {
            continue;
        }

        let candidate = (item_id, item.derived_stats);
        let replace = match best {
            None => true,
            Some((best_id, best_stats)) => {
                candidate.1.damage > best_stats.damage + f64::EPSILON
                    || ((candidate.1.damage - best_stats.damage).abs() <= f64::EPSILON
                        && (candidate.1.speed > best_stats.speed + f64::EPSILON
                            || ((candidate.1.speed - best_stats.speed).abs() <= f64::EPSILON
                                && candidate.0 < best_id)))
            }
        };

        if replace {
            best = Some(candidate);
        }
    }

    best
}

pub fn tool_adjusted_action_timer(base_timer: i32, tool_speed: f64) -> i32 {
    let adjusted = ((base_timer as f64)
        / (1.0
            + (tool_speed.max(config::TOOL_BASE_SPEED) - config::TOOL_BASE_SPEED)
                * config::TOOL_SPEED_EFFECT_SCALE))
        .round() as i32;
    adjusted.max(config::TOOL_MIN_ACTION_TIMER)
}

pub fn use_tool(
    entity_id: EntityId,
    item_id: ItemId,
    resources: &mut SimResources,
    tick: u64,
) -> bool {
    let Some((template_id, material_id)) = ({
        let Some(item) = resources.item_store.get_mut(item_id) else {
            return false;
        };
        item.current_durability -= config::TOOL_DURABILITY_COST_PER_USE;
        if !item.is_broken() {
            None
        } else {
            Some((item.template_id.clone(), item.material_id.clone()))
        }
    }) else {
        return false;
    };

    if resources.item_store.remove(item_id).is_none() {
        return false;
    }

    resources.causal_log.push(
        entity_id,
        CausalEvent {
            tick,
            cause: CauseRef {
                system: "tool_system".to_string(),
                kind: "tool_broken".to_string(),
                entity: Some(entity_id),
                building: None,
                settlement: None,
            },
            effect_key: format!("item_destroyed:{template_id}:{material_id}"),
            summary_key: "TOOL_BROKEN".to_string(),
            magnitude: -1.0,
        },
    );

    true
}

pub fn craft_complete(
    entity_id: EntityId,
    recipe_id: &str,
    material_id: &str,
    resources: &mut SimResources,
    tick: u64,
) -> Vec<ItemId> {
    let Some(registry_arc) = resources.data_registry.clone() else {
        return Vec::new();
    };
    let registry = registry_arc.as_ref();
    let Some(recipe) = registry.recipes.get(recipe_id) else {
        return Vec::new();
    };
    let Some(material) = registry.materials.get(material_id) else {
        return Vec::new();
    };

    let stats = registry.derive_item_stats(recipe.output.template.as_str(), material);
    let derived =
        ItemDerivedStats::from_material_stats(stats.damage, stats.speed, stats.durability);
    let count = recipe.output.count.unwrap_or(1);
    let mut created_ids = Vec::with_capacity(count as usize);

    for _ in 0..count {
        let item_id = resources.item_store.allocate_id();
        let item = ItemInstance {
            id: item_id,
            template_id: recipe.output.template.clone(),
            material_id: material_id.to_string(),
            derived_stats: derived,
            current_durability: derived.max_durability,
            quality: 0.5,
            owner: ItemOwner::Agent(entity_id),
            stack_count: 1,
            created_tick: tick,
            creator_id: Some(entity_id),
            equipped_slot: None,
        };
        resources.item_store.insert(item);
        created_ids.push(item_id);
    }

    if !created_ids.is_empty() {
        resources.causal_log.push(
            entity_id,
            CausalEvent {
                tick,
                cause: CauseRef {
                    system: "crafting_system".to_string(),
                    kind: format!("craft_{recipe_id}"),
                    entity: Some(entity_id),
                    building: None,
                    settlement: None,
                },
                effect_key: format!("item_created:{}", recipe.output.template),
                summary_key: format!("CRAFT_{}", recipe_id.to_uppercase()),
                magnitude: f64::from(count),
            },
        );
    }

    created_ids
}

fn clear_craft_behavior(behavior: &mut Behavior) {
    behavior.current_action = ActionType::Idle;
    behavior.action_target_entity = None;
    behavior.action_target_x = None;
    behavior.action_target_y = None;
    behavior.action_progress = 0.0;
    behavior.action_duration = 0;
    behavior.action_timer = 0;
    behavior.craft_recipe_id = None;
    behavior.craft_material_id = None;
}

fn is_tool_template(template_id: &str) -> bool {
    matches!(
        template_id,
        "knife" | "axe" | "scraper" | "spear" | "hammer" | "pick"
    )
}

fn tag_to_stockpile_resource(tag: &str) -> Option<&'static str> {
    match tag {
        "sharp" | "hard" | "stone" | "knappable" | "tool_material" => Some("stone"),
        "binding" | "plant_fiber" | "cordage" => Some("wood"),
        "bone" | "animal" => Some("food"),
        "wood" | "heavy" | "building_material" => Some("wood"),
        _ => None,
    }
}

fn stockpile_amount(settlement: &Settlement, resource: &str) -> f64 {
    match resource {
        "stone" => settlement.stockpile_stone,
        "wood" => settlement.stockpile_wood,
        "food" => settlement.stockpile_food,
        _ => 0.0,
    }
}

fn deduct_stockpile(settlement: &mut Settlement, resource: &str, amount: f64) -> bool {
    match resource {
        "stone" if settlement.stockpile_stone >= amount => {
            settlement.stockpile_stone -= amount;
            true
        }
        "wood" if settlement.stockpile_wood >= amount => {
            settlement.stockpile_wood -= amount;
            true
        }
        "food" if settlement.stockpile_food >= amount => {
            settlement.stockpile_food -= amount;
            true
        }
        _ => false,
    }
}

fn recipe_stockpile_costs(recipe: &RecipeDef) -> Option<BTreeMap<String, f64>> {
    let mut costs = BTreeMap::new();
    for req in &recipe.inputs {
        let resource = tag_to_stockpile_resource(req.tag.as_str())?;
        *costs.entry(resource.to_string()).or_insert(0.0) += f64::from(req.amount);
    }
    Some(costs)
}

fn select_best_recipe(registry: &DataRegistry, settlement: &Settlement) -> Option<CraftSelection> {
    let mut recipe_ids: Vec<&str> = registry.recipes.keys().map(String::as_str).collect();
    recipe_ids.sort_unstable();

    let mut best: Option<(f64, String, String, i32)> = None;
    for recipe_id in recipe_ids {
        let Some(recipe) = registry.recipes.get(recipe_id) else {
            continue;
        };
        let Some(costs) = recipe_stockpile_costs(recipe.as_ref()) else {
            continue;
        };
        if costs
            .iter()
            .any(|(resource, amount)| stockpile_amount(settlement, resource.as_str()) < *amount)
        {
            continue;
        }

        let material_idx = recipe.output.material_from_input.min(recipe.inputs.len());
        if material_idx >= recipe.inputs.len() {
            continue;
        }
        let req = &recipe.inputs[material_idx];
        let mut materials = registry.find_materials_by_tag(req);
        if materials.is_empty() {
            continue;
        }

        materials.sort_by(|left, right| {
            let left_stats = registry.derive_item_stats(recipe.output.template.as_str(), left);
            let right_stats = registry.derive_item_stats(recipe.output.template.as_str(), right);
            right_stats
                .damage
                .partial_cmp(&left_stats.damage)
                .unwrap_or(Ordering::Equal)
                .then_with(|| {
                    right
                        .properties
                        .hardness
                        .partial_cmp(&left.properties.hardness)
                        .unwrap_or(Ordering::Equal)
                })
                .then_with(|| left.id.cmp(&right.id))
        });

        let material = &materials[0];
        let score = if is_tool_template(recipe.output.template.as_str()) {
            registry
                .derive_item_stats(recipe.output.template.as_str(), material)
                .damage
        } else {
            0.0
        };
        let duration_ticks = recipe.time_ticks.min(i32::MAX as u32) as i32;

        let replace = match best.as_ref() {
            None => true,
            Some((best_score, best_recipe_id, best_material_id, _)) => {
                score > *best_score + f64::EPSILON
                    || ((score - *best_score).abs() <= f64::EPSILON
                        && (recipe.id.as_str(), material.id.as_str())
                            < (best_recipe_id.as_str(), best_material_id.as_str()))
            }
        };

        if replace {
            best = Some((
                score,
                recipe.id.clone(),
                material.id.clone(),
                duration_ticks.max(1),
            ));
        }
    }

    best.map(
        |(_, recipe_id, material_id, duration_ticks)| CraftSelection {
            recipe_id,
            material_id,
            duration_ticks,
        },
    )
}

fn deduct_recipe_costs(settlement: &mut Settlement, recipe: &RecipeDef) -> bool {
    let Some(costs) = recipe_stockpile_costs(recipe) else {
        return false;
    };
    if costs
        .iter()
        .any(|(resource, amount)| stockpile_amount(settlement, resource.as_str()) < *amount)
    {
        return false;
    }
    for (resource, amount) in costs {
        if !deduct_stockpile(settlement, resource.as_str(), amount) {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use std::path::PathBuf;
    use std::sync::{Arc, OnceLock};

    use sim_core::components::{Inventory, Needs};
    use sim_core::config::GameConfig;
    use sim_core::{GameCalendar, Settlement, SettlementId, WorldMap};

    fn authoritative_ron_data_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("sim-systems crate should live in rust/crates")
            .join("sim-data")
            .join("data")
    }

    fn test_registry() -> Arc<DataRegistry> {
        static REGISTRY: OnceLock<Arc<DataRegistry>> = OnceLock::new();
        REGISTRY
            .get_or_init(|| {
                let base = authoritative_ron_data_dir();
                Arc::new(
                    DataRegistry::load_from_directory(Path::new(&base))
                        .expect("authoritative recipe/material data should load"),
                )
            })
            .clone()
    }

    fn make_resources() -> SimResources {
        let config = GameConfig::default();
        let calendar = GameCalendar::new(&config);
        let map = WorldMap::new(8, 8, 17);
        let mut resources = SimResources::new(calendar, map, 19);
        resources.data_registry = Some(test_registry());
        resources
    }

    fn seeded_settlement() -> Settlement {
        let mut settlement = Settlement::new(SettlementId(1), "Forge".to_string(), 2, 2, 0);
        settlement.stockpile_food = 6.0;
        settlement.stockpile_wood = 6.0;
        settlement.stockpile_stone = 6.0;
        settlement
    }

    #[test]
    fn craft_system_selects_recipe_for_available_materials() {
        let mut world = World::new();
        let mut resources = make_resources();
        let mut settlement = seeded_settlement();
        settlement.stockpile_wood = 0.0;
        resources.settlements.insert(settlement.id, settlement);

        let entity = world.spawn((
            Behavior {
                current_action: ActionType::Craft,
                action_timer: 30,
                action_duration: 30,
                ..Behavior::default()
            },
            Identity {
                settlement_id: Some(SettlementId(1)),
                ..Identity::default()
            },
            Inventory::new(),
        ));

        let mut system = CraftingRuntimeSystem::new(29, 10);
        system.run(&mut world, &mut resources, 10);

        let behavior = world
            .get::<&Behavior>(entity)
            .expect("behavior should remain queryable");
        assert_eq!(behavior.craft_recipe_id.as_deref(), Some("stone_knife"));
        assert!(behavior.craft_material_id.is_some());
        assert_eq!(behavior.action_duration, 60);
        assert_eq!(behavior.action_timer, 60);
    }

    #[test]
    fn craft_complete_creates_item_in_store() {
        let mut resources = make_resources();
        let created = craft_complete(EntityId(7), "stone_knife", "flint", &mut resources, 25);
        assert_eq!(created.len(), 1);
        assert_eq!(resources.item_store.len(), 1);
        let item = resources
            .item_store
            .get(created[0])
            .expect("crafted item should be stored");
        assert_eq!(item.template_id, "knife");
        assert_eq!(item.material_id, "flint");
    }

    #[test]
    fn craft_complete_item_has_correct_material_stats() {
        let mut resources = make_resources();
        let flint = craft_complete(EntityId(7), "stone_knife", "flint", &mut resources, 25);
        let granite = craft_complete(EntityId(7), "stone_knife", "granite", &mut resources, 30);
        let flint_item = resources
            .item_store
            .get(flint[0])
            .expect("flint craft should exist");
        let granite_item = resources
            .item_store
            .get(granite[0])
            .expect("granite craft should exist");
        assert!(flint_item.derived_stats.damage > granite_item.derived_stats.damage);
    }

    #[test]
    fn craft_deducts_settlement_stockpile() {
        let mut world = World::new();
        let mut resources = make_resources();
        resources
            .settlements
            .insert(SettlementId(1), seeded_settlement());

        world.spawn((
            Behavior {
                current_action: ActionType::Craft,
                action_timer: 30,
                action_duration: 30,
                ..Behavior::default()
            },
            Identity {
                settlement_id: Some(SettlementId(1)),
                ..Identity::default()
            },
            Inventory::new(),
        ));

        let mut system = CraftingRuntimeSystem::new(29, 10);
        let before = resources
            .settlements
            .get(&SettlementId(1))
            .expect("settlement should exist")
            .stockpile_stone;
        system.run(&mut world, &mut resources, 10);
        let after = resources
            .settlements
            .get(&SettlementId(1))
            .expect("settlement should exist")
            .stockpile_stone;
        assert!(after < before);
    }

    #[test]
    fn craft_writes_causal_log() {
        let mut resources = make_resources();
        let entity_id = EntityId(9);
        let _ = craft_complete(entity_id, "stone_knife", "obsidian", &mut resources, 44);
        let recent = resources.causal_log.recent(entity_id, 4);
        assert_eq!(recent.len(), 1);
        assert_eq!(recent[0].summary_key, "CRAFT_STONE_KNIFE");
    }

    #[test]
    fn craft_system_clears_action_when_recipe_unavailable() {
        let mut world = World::new();
        let mut resources = make_resources();
        let mut settlement = seeded_settlement();
        settlement.stockpile_food = 0.0;
        settlement.stockpile_wood = 0.0;
        settlement.stockpile_stone = 0.0;
        resources.settlements.insert(settlement.id, settlement);

        let entity = world.spawn((
            Behavior {
                current_action: ActionType::Craft,
                action_timer: 30,
                action_duration: 30,
                ..Behavior::default()
            },
            Identity {
                settlement_id: Some(SettlementId(1)),
                ..Identity::default()
            },
            Needs::default(),
        ));

        let mut system = CraftingRuntimeSystem::new(29, 10);
        system.run(&mut world, &mut resources, 10);

        let behavior = world
            .get::<&Behavior>(entity)
            .expect("behavior should remain queryable");
        assert_eq!(behavior.current_action, ActionType::Idle);
        assert!(behavior.craft_recipe_id.is_none());
        assert!(behavior.craft_material_id.is_none());
    }

    #[test]
    fn tool_lookup_finds_best_by_damage() {
        let mut resources = make_resources();
        let mut inventory = Inventory::new();

        let weaker_id = resources.item_store.allocate_id();
        resources.item_store.insert(ItemInstance {
            id: weaker_id,
            template_id: "knife".to_string(),
            material_id: "granite".to_string(),
            derived_stats: ItemDerivedStats::from_material_stats(7.8, 1.8, 175.5),
            current_durability: 175.5,
            quality: 0.5,
            owner: ItemOwner::Agent(EntityId(4)),
            stack_count: 1,
            created_tick: 0,
            creator_id: Some(EntityId(4)),
            equipped_slot: None,
        });
        inventory.add(weaker_id);

        let stronger_id = resources.item_store.allocate_id();
        resources.item_store.insert(ItemInstance {
            id: stronger_id,
            template_id: "knife".to_string(),
            material_id: "obsidian".to_string(),
            derived_stats: ItemDerivedStats::from_material_stats(9.0, 2.2, 184.0),
            current_durability: 184.0,
            quality: 0.5,
            owner: ItemOwner::Agent(EntityId(4)),
            stack_count: 1,
            created_tick: 0,
            creator_id: Some(EntityId(4)),
            equipped_slot: None,
        });
        inventory.add(stronger_id);

        let found = find_best_tool(&inventory, &resources.item_store, "knife");
        assert_eq!(found.map(|(item_id, _)| item_id), Some(stronger_id));
    }

    #[test]
    fn tool_speed_reduces_action_timer() {
        let base_timer = 30;
        let adjusted = tool_adjusted_action_timer(base_timer, 2.0);
        assert!(adjusted < base_timer);
        assert!(adjusted >= config::TOOL_MIN_ACTION_TIMER);
    }

    #[test]
    fn tool_use_decreases_durability() {
        let mut resources = make_resources();
        let item_id = resources.item_store.allocate_id();
        resources.item_store.insert(ItemInstance {
            id: item_id,
            template_id: "axe".to_string(),
            material_id: "flint".to_string(),
            derived_stats: ItemDerivedStats::from_material_stats(8.4, 2.0, 50.0),
            current_durability: 10.0,
            quality: 0.5,
            owner: ItemOwner::Agent(EntityId(11)),
            stack_count: 1,
            created_tick: 0,
            creator_id: Some(EntityId(11)),
            equipped_slot: None,
        });

        let broken = use_tool(EntityId(11), item_id, &mut resources, 12);
        assert!(!broken);
        let item = resources
            .item_store
            .get(item_id)
            .expect("tool should still exist after first use");
        assert_eq!(
            item.current_durability,
            10.0 - config::TOOL_DURABILITY_COST_PER_USE
        );
    }

    #[test]
    fn tool_destruction_removes_from_store() {
        let mut resources = make_resources();
        let item_id = resources.item_store.allocate_id();
        resources.item_store.insert(ItemInstance {
            id: item_id,
            template_id: "knife".to_string(),
            material_id: "obsidian".to_string(),
            derived_stats: ItemDerivedStats::from_material_stats(9.0, 2.2, 1.0),
            current_durability: 1.0,
            quality: 0.5,
            owner: ItemOwner::Agent(EntityId(13)),
            stack_count: 1,
            created_tick: 0,
            creator_id: Some(EntityId(13)),
            equipped_slot: None,
        });

        let broken = use_tool(EntityId(13), item_id, &mut resources, 21);
        assert!(broken);
        assert!(resources.item_store.get(item_id).is_none());
    }

    #[test]
    fn tool_break_writes_causal_log() {
        let mut resources = make_resources();
        let item_id = resources.item_store.allocate_id();
        resources.item_store.insert(ItemInstance {
            id: item_id,
            template_id: "axe".to_string(),
            material_id: "native_copper".to_string(),
            derived_stats: ItemDerivedStats::from_material_stats(4.8, 1.5, 1.0),
            current_durability: 1.0,
            quality: 0.5,
            owner: ItemOwner::Agent(EntityId(17)),
            stack_count: 1,
            created_tick: 0,
            creator_id: Some(EntityId(17)),
            equipped_slot: None,
        });

        assert!(use_tool(EntityId(17), item_id, &mut resources, 33));
        let recent = resources.causal_log.recent(EntityId(17), 4);
        assert_eq!(recent.len(), 1);
        assert_eq!(recent[0].summary_key, "TOOL_BROKEN");
    }
}
