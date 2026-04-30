//! sim-bridge: Boundary adapters between external callers and simulation crates.
//!
//! Phase R-3 will expose this through Godot GDExtension.
//! For now, this module provides pure-Rust conversion helpers that can be
//! reused by the future FFI layer.

// Bridge functions mirror GDScript signatures which naturally have many parameters.
#![allow(clippy::too_many_arguments)]

mod body_bindings;
mod debug_api;
mod locale_bindings;
mod narrative_display;
mod pathfinding_bindings;
mod pathfinding_core;
mod runtime_bindings;
mod runtime_commands;
mod runtime_dict;
mod runtime_events;
mod runtime_queries;
mod runtime_registry;
mod runtime_system;
mod snapshot_buffer;
pub mod temperament_detail;
pub mod tile_info;
mod ws2_codec;

use body_bindings::{
    build_step_pairs, packed_f32_to_vec, packed_i32_to_vec, packed_u8_to_vec, vec_f32_to_packed,
    vec_i32_to_packed, vec_u8_to_packed,
};
use godot::prelude::*;
#[cfg(test)]
use locale_bindings::format_fluent_from_source_args;
use locale_bindings::{
    clear_fluent_source, format_active_fluent_message, format_fluent_message, store_fluent_source,
};
use narrative_display::{build_narrative_display, narrative_display_to_dict, NarrativeDisplayData};
#[cfg(test)]
use pathfinding_bindings::parse_pathfind_backend;
use pathfinding_bindings::{
    backend_mode_to_str, encode_path_groups_vec2, encode_path_groups_xy, encode_path_vec2,
    encode_path_xy, normalize_max_steps, resolve_backend_mode, resolve_backend_mode_code,
};
use pathfinding_core::{
    get_backend_mode, has_gpu_backend, read_dispatch_counts, reset_dispatch_counts,
    PATHFIND_BACKEND_GPU,
};
use sim_core::components::{
    Age, AgentKnowledge, Behavior, Body, BodyHealth, Coping, Economic, Emotion, Faith,
    FamilyComponent, Identity, Intelligence, Inventory, LlmCapable, LlmPending, LlmRequestType,
    Memory, NarrativeCache, Needs, Personality, Position, Skills, Social, Stress, Traits, Values,
    PART_NAMES, PART_TO_GROUP, PART_VITAL,
};
use sim_core::enums::{ActionType, GrowthStage, NeedType, Sex};
use sim_core::{BandId, CausalEvent, ChannelId, EntityId, EquipSlot, Settlement, SettlementId, Temperament};
use sim_engine::{
    build_agent_multimesh_buffer, build_wildlife_snapshots, AgentSnapshot, ChronicleEntryDetailSnapshot,
    ChronicleEntryId, ChronicleEntryLite, ChronicleEvent, ChronicleFeedItemSnapshot,
    ChronicleFeedResponse, ChronicleHistorySliceResponse, ChronicleRecallSliceResponse,
    ChronicleSnapshotRevision, ChronicleThreadListResponse, EngineSnapshot, GameEvent,
    LlmPromptVariant, LlmRequest, SimEvent, SimEventType,
};
use sim_systems::{body, drain_and_apply_llm_responses, entity_spawner, stat_curve};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
pub(crate) use ws2_codec::{decode_ws2_blob, encode_ws2_blob};

use pathfinding_core::{
    dispatch_pathfind_grid_batch_vec2_bytes, dispatch_pathfind_grid_batch_xy_bytes,
    dispatch_pathfind_grid_bytes,
};
pub use runtime_system::default_runtime_system_registry_names;
pub use runtime_system::{
    default_runtime_systems_count, spec_tier, systems_by_tier, tier_distribution,
};
pub use pathfinding_core::{
    get_pathfind_backend_mode, has_gpu_pathfind_backend, pathfind_backend_dispatch_counts,
    pathfind_from_flat, pathfind_grid_batch_bytes, pathfind_grid_batch_dispatch_bytes,
    pathfind_grid_batch_vec2_bytes, pathfind_grid_batch_xy_bytes,
    pathfind_grid_batch_xy_dispatch_bytes, pathfind_grid_bytes,
    reset_pathfind_backend_dispatch_counts, resolve_pathfind_backend_mode,
    set_pathfind_backend_mode, PathfindError, PathfindInput,
};
use runtime_commands::{apply_commands_v2, registry_snapshot};
use runtime_registry::{
    clamp_speed_index, load_legacy_runtime_bootstrap, parse_runtime_config,
    register_default_runtime_systems, runtime_speed_multiplier, RuntimeState,
};
use snapshot_buffer::SnapshotBuffer;

use runtime_events::game_event_to_v2_dict;
use runtime_queries::{
    bootstrap_world as bridge_bootstrap_world, building_detail as bridge_building_detail,
    minimap_snapshot as bridge_minimap_snapshot, runtime_bits_from_raw_id,
    settlement_detail as bridge_settlement_detail, world_summary as bridge_world_summary,
};

/// JSON-deserializable entry for runtime_spawn_agents.
#[derive(serde::Deserialize)]
struct SpawnEntry {
    x: i32,
    y: i32,
    age_ticks: Option<u64>,
    settlement_id: Option<u64>,
    settlement_x: Option<i32>,
    settlement_y: Option<i32>,
}

#[derive(GodotClass)]
#[class(base=Object)]
pub struct WorldSimRuntime {
    base: Base<Object>,
    state: Option<RuntimeState>,
    snapshot_buffer: SnapshotBuffer,
    render_alpha: f64,
    /// Cached per-frame agent snapshot array to avoid rebuilding every render frame.
    cached_agent_snapshots: Array<VarDictionary>,
    /// Tick number when `cached_agent_snapshots` was last rebuilt.
    last_agent_snapshot_tick: u64,
}

#[godot_api]
impl IObject for WorldSimRuntime {
    fn init(base: Base<Object>) -> Self {
        Self {
            base,
            state: None,
            snapshot_buffer: SnapshotBuffer::new(),
            render_alpha: 0.0,
            cached_agent_snapshots: Array::new(),
            last_agent_snapshot_tick: 0,
        }
    }
}

fn encode_snapshot_bytes(snapshots: &[AgentSnapshot]) -> PackedByteArray {
    let mut bytes: Vec<u8> = Vec::with_capacity(std::mem::size_of_val(snapshots));
    for snapshot in snapshots {
        snapshot.write_bytes(&mut bytes);
    }
    PackedByteArray::from(bytes)
}

fn authoritative_ron_data_dir() -> PathBuf {
    PathBuf::from("rust")
        .join("crates")
        .join("sim-data")
        .join("data")
}

fn legacy_json_data_dir() -> PathBuf {
    PathBuf::from("data")
}

fn resolve_runtime_entity(world: &hecs::World, entity_id_raw_or_bits: i64) -> Option<hecs::Entity> {
    let raw_or_bits = entity_id_raw_or_bits.max(0) as u64;
    if let Some(entity) = hecs::Entity::from_bits(raw_or_bits) {
        if world.contains(entity) {
            return Some(entity);
        }
    }
    let raw_lookup = runtime_queries::build_raw_entity_id_lookup(world);
    let runtime_bits = runtime_bits_from_raw_id(&raw_lookup, raw_or_bits)?;
    let entity = hecs::Entity::from_bits(runtime_bits as u64)?;
    if world.contains(entity) {
        return Some(entity);
    }
    None
}

fn archetype_label_key_from_axes(axes: [f64; 6]) -> &'static str {
    const HIGH_KEYS: [&str; 6] = [
        "ARCHETYPE_PRINCIPLED_GUARDIAN",
        "ARCHETYPE_SENSITIVE_SOUL",
        "ARCHETYPE_BOLD_EXPLORER",
        "ARCHETYPE_GENTLE_PEACEMAKER",
        "ARCHETYPE_DILIGENT_PLANNER",
        "ARCHETYPE_CURIOUS_DREAMER",
    ];
    const LOW_KEYS: [&str; 6] = [
        "ARCHETYPE_CUNNING_OPPORTUNIST",
        "ARCHETYPE_STOIC_SURVIVOR",
        "ARCHETYPE_QUIET_OBSERVER",
        "ARCHETYPE_SHARP_CHALLENGER",
        "ARCHETYPE_FREE_SPIRIT",
        "ARCHETYPE_PRACTICAL_REALIST",
    ];

    let mut max_index: usize = 0;
    let mut max_deviation: f64 = -1.0;
    let mut is_high = true;
    for (index, value) in axes.into_iter().enumerate() {
        let deviation = (value - 0.5).abs();
        if deviation > max_deviation {
            max_deviation = deviation;
            max_index = index;
            is_high = value >= 0.5;
        }
    }

    if is_high {
        HIGH_KEYS[max_index]
    } else {
        LOW_KEYS[max_index]
    }
}

fn humanize_status_text(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return "Idle".to_string();
    }

    let normalized = trimmed.replace('_', " ");
    let mut output = String::with_capacity(normalized.len() + 8);
    let mut previous_is_lowercase = false;
    for character in normalized.chars() {
        let is_uppercase = character.is_uppercase();
        if previous_is_lowercase && is_uppercase {
            output.push(' ');
        }
        if output.is_empty() {
            output.push(character.to_ascii_uppercase());
        } else {
            output.push(character);
        }
        previous_is_lowercase = character.is_lowercase();
    }
    output
}

fn identifier_to_upper_snake(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    let mut output = String::with_capacity(trimmed.len() + 8);
    let mut previous_is_lowercase = false;
    for character in trimmed.chars() {
        if matches!(character, ' ' | '-' | '_') {
            if !output.ends_with('_') && !output.is_empty() {
                output.push('_');
            }
            previous_is_lowercase = false;
            continue;
        }
        let is_uppercase = character.is_uppercase();
        if previous_is_lowercase && is_uppercase && !output.ends_with('_') {
            output.push('_');
        }
        output.push(character.to_ascii_uppercase());
        previous_is_lowercase = character.is_lowercase();
    }
    output
}

fn active_fluent_or_humanized(key: &str, fallback: &str) -> String {
    format_active_fluent_message(key).unwrap_or_else(|| fallback.to_string())
}

fn localized_need_label(cause: &str) -> String {
    let key = format!("NEED_{}", identifier_to_upper_snake(cause));
    active_fluent_or_humanized(&key, &humanize_status_text(cause))
}

fn localized_emotion_label(cause: &str) -> String {
    let key = format!("EMO_{}", identifier_to_upper_snake(cause));
    active_fluent_or_humanized(&key, &humanize_status_text(cause))
}

fn localized_status_label(cause: &str) -> String {
    let key = format!("STATUS_{}", identifier_to_upper_snake(cause));
    active_fluent_or_humanized(&key, &humanize_status_text(cause))
}

fn localized_stage_label(cause: &str) -> String {
    let key = format!("STAGE_{}", identifier_to_upper_snake(cause));
    active_fluent_or_humanized(&key, &humanize_status_text(cause))
}

fn localized_death_label(cause: &str) -> String {
    let key = match cause.trim() {
        "mortality_hazard" => "DEATH_BACKGROUND".to_string(),
        other => format!("DEATH_{}", identifier_to_upper_snake(other)),
    };
    active_fluent_or_humanized(&key, &humanize_status_text(cause))
}

fn top_need_key_and_value(needs: &Needs) -> (&'static str, f64) {
    let mut best_key = "NEED_HUNGER";
    let mut best_value = needs.values.first().copied().unwrap_or(1.0);
    let need_keys = [
        "NEED_HUNGER",
        "NEED_THIRST",
        "NEED_SLEEP",
        "NEED_WARMTH",
        "NEED_SAFETY",
        "NEED_BELONGING",
        "NEED_INTIMACY",
        "NEED_RECOGNITION",
        "NEED_AUTONOMY",
        "NEED_COMPETENCE",
        "NEED_SELF_ACTUALIZATION",
        "NEED_MEANING",
        "NEED_TRANSCENDENCE",
    ];

    for (index, key) in need_keys.iter().enumerate().skip(1) {
        let value = needs.values.get(index).copied().unwrap_or(1.0);
        if value < best_value {
            best_key = key;
            best_value = value;
        }
    }

    if needs.energy < best_value {
        ("NEED_ENERGY", needs.energy)
    } else {
        (best_key, best_value)
    }
}

fn diagnostic_comfort_score(warmth: f64, safety: f64, sleep: f64) -> f64 {
    ((warmth + safety + sleep) / 3.0).clamp(0.0, 1.0)
}

fn action_target_resource_key(action: ActionType) -> &'static str {
    match action {
        ActionType::Forage | ActionType::GatherHerbs => "food",
        ActionType::GatherWood => "wood",
        ActionType::GatherStone => "stone",
        ActionType::Build => "building",
        _ => "",
    }
}

fn chronicle_event_to_dict(event: &ChronicleEvent) -> VarDictionary {
    let mut dict = VarDictionary::new();
    dict.set("tick", event.tick as i64);
    dict.set("event_type", format!("{:?}", event.event_type));
    dict.set("entity_id", event.entity_id.0 as i64);
    dict.set("influence_channel_id", event.cause.id());
    dict.set("summary_key", event.summary_key.clone());
    dict.set("effect_key", event.effect_key.clone());
    dict.set("tile_x", event.tile_x);
    dict.set("tile_y", event.tile_y);
    dict.set("influence_magnitude", event.magnitude.influence as f32);
    dict.set("steering_magnitude", event.magnitude.steering as f32);
    dict.set("significance", event.magnitude.significance as f32);
    dict
}

/// Converts causal events into a Godot Array of VarDictionaries for GDScript consumption.
fn causal_events_to_vardict(events: Vec<&CausalEvent>) -> Array<VarDictionary> {
    let mut arr: Array<VarDictionary> = Array::new();
    for event in events {
        let mut entry = VarDictionary::new();
        entry.set("tick", event.tick as i64);
        entry.set("system", event.cause.system.clone());
        entry.set("kind", event.cause.kind.clone());
        entry.set("effect_key", event.effect_key.clone());
        entry.set("summary_key", event.summary_key.clone());
        entry.set("magnitude", event.magnitude as f32);
        entry.set(
            "source_entity_id",
            event.cause.entity.map(|e| e.0 as i64).unwrap_or(-1_i64),
        );
        entry.set(
            "source_building_id",
            event.cause.building.map(|b| b.0 as i64).unwrap_or(-1_i64),
        );
        entry.set(
            "source_settlement_id",
            event.cause.settlement.map(|s| s.0 as i64).unwrap_or(-1_i64),
        );
        arr.push(&entry);
    }
    arr
}

/// Temporary migration adapter for legacy Chronicle UI consumers.
///
/// Runtime chronicle authority now lives in `ChronicleEntryLite`. This adapter preserves the
/// existing dictionary shape while bridge/UI callers migrate off summary-centric contracts.
fn chronicle_layer_params_to_dict(
    params: &std::collections::BTreeMap<String, String>,
) -> VarDictionary {
    let mut dict = VarDictionary::new();
    for (key, value) in params {
        dict.set(key.as_str(), value.as_str());
    }
    dict
}

fn chronicle_detail_tags_to_array(tags: &[String]) -> PackedStringArray {
    let mut out = PackedStringArray::new();
    for tag in tags {
        out.push(tag.as_str());
    }
    out
}

fn chronicle_snapshot_revision_from_arg(value: i64) -> Option<ChronicleSnapshotRevision> {
    (value >= 0).then_some(ChronicleSnapshotRevision(value as u64))
}

fn chronicle_entry_status_id(entry: &ChronicleEntryLite) -> &'static str {
    match entry.status {
        sim_engine::ChronicleEntryStatus::Pending => "pending",
        sim_engine::ChronicleEntryStatus::Published => "published",
        sim_engine::ChronicleEntryStatus::Suppressed => "suppressed",
        sim_engine::ChronicleEntryStatus::Archived => "archived",
    }
}

fn chronicle_subject_ref_lite_to_dict(
    subject: &sim_engine::ChronicleSubjectRefLite,
) -> VarDictionary {
    let mut dict = VarDictionary::new();
    dict.set(
        "entity_id",
        subject
            .entity_id
            .map(|entity_id| entity_id.0 as i64)
            .unwrap_or(-1),
    );
    dict.set(
        "display_name",
        subject.display_name.as_deref().unwrap_or_default(),
    );
    dict.set("ref_state", subject.ref_state.id());
    dict
}

fn chronicle_location_ref_lite_to_dict(
    location: &sim_engine::ChronicleLocationRefLite,
) -> VarDictionary {
    let mut dict = VarDictionary::new();
    dict.set("tile_x", location.tile_x);
    dict.set("tile_y", location.tile_y);
    dict.set(
        "region_label",
        location.region_label.as_deref().unwrap_or_default(),
    );
    dict
}

fn chronicle_feed_item_snapshot_to_dict(item: &ChronicleFeedItemSnapshot) -> VarDictionary {
    let mut dict = VarDictionary::new();
    let mut primary_subjects: Array<VarDictionary> = Array::new();
    for subject in &item.primary_subjects {
        primary_subjects.push(&chronicle_subject_ref_lite_to_dict(subject));
    }
    let mut render_hint = VarDictionary::new();
    render_hint.set("icon_id", item.render_hint.icon_id.as_str());
    render_hint.set("color_token", item.render_hint.color_token.as_str());

    dict.set("entry_id", item.entry_id.0 as i64);
    dict.set(
        "thread_id",
        item.thread_id
            .map(|thread_id| thread_id as i64)
            .unwrap_or(-1),
    );
    dict.set("event_type", format!("{:?}", item.event_type));
    dict.set("cause_id", item.cause.id());
    dict.set("queue_bucket", item.queue_bucket.id());
    dict.set("category_id", item.category.id());
    dict.set("significance", item.significance as f32);
    dict.set("start_tick", item.start_tick as i64);
    dict.set("end_tick", item.end_tick as i64);
    dict.set("tick", item.end_tick as i64);
    dict.set("headline_key", item.headline.locale_key.as_str());
    dict.set(
        "headline_params",
        chronicle_layer_params_to_dict(&item.headline.params),
    );
    dict.set("capsule_key", item.capsule.locale_key.as_str());
    dict.set(
        "capsule_params",
        chronicle_layer_params_to_dict(&item.capsule.params),
    );
    dict.set("tile_x", item.location_ref.tile_x);
    dict.set("tile_y", item.location_ref.tile_y);
    dict.set(
        "location",
        chronicle_location_ref_lite_to_dict(&item.location_ref),
    );
    if let Some(subject) = item.primary_subjects.first() {
        dict.set("ref_state", subject.ref_state.id());
    }
    if let Some(region_label) = item.location_ref.region_label.as_deref() {
        dict.set("region_label", region_label);
    }
    dict.set("primary_subjects", primary_subjects);
    dict.set("render_hint", render_hint);
    let entity_id = item
        .primary_subjects
        .first()
        .and_then(|subject| subject.entity_id)
        .map(|entity_id| entity_id.0 as i64)
        .unwrap_or(-1);
    dict.set("entity_id", entity_id);
    if let Some(subject) = item.primary_subjects.first() {
        if let Some(display_name) = subject.display_name.as_deref() {
            dict.set("entity_name", display_name);
        } else if let Some(agent_name) = item.capsule.params.get("agent") {
            dict.set("entity_name", agent_name.as_str());
        } else if let Some(agent_name) = item.headline.params.get("agent") {
            dict.set("entity_name", agent_name.as_str());
        }
    } else if let Some(agent_name) = item.capsule.params.get("agent") {
        dict.set("entity_name", agent_name.as_str());
    } else if let Some(agent_name) = item.headline.params.get("agent") {
        dict.set("entity_name", agent_name.as_str());
    }
    dict
}

fn chronicle_telemetry_to_dict(telemetry: &sim_engine::ChronicleTelemetry) -> VarDictionary {
    let mut dict = VarDictionary::new();
    dict.set("total_routed", telemetry.total_routed as i64);
    dict.set("visible_count", telemetry.visible_count as i64);
    dict.set("background_count", telemetry.background_count as i64);
    dict.set("recall_count", telemetry.recall_count as i64);
    dict.set("drop_count", telemetry.drop_count as i64);
    dict.set("displacement_count", telemetry.displacement_count as i64);
    dict.set("archive_count", telemetry.archive_count as i64);
    dict.set("promotion_count", telemetry.promotion_count as i64);
    dict.set("thread_create_count", telemetry.thread_create_count as i64);
    dict.set("thread_evict_count", telemetry.thread_evict_count as i64);
    dict
}

/// Temporary migration adapter for legacy timeline consumers.
///
/// The legacy timeline endpoint now delegates to the new feed snapshot family and derives the
/// old flat summary fields from feed-level payloads.
fn chronicle_feed_item_snapshot_to_legacy_dict(item: &ChronicleFeedItemSnapshot) -> VarDictionary {
    let mut dict = VarDictionary::new();
    let primary_subject = item.primary_subjects.first();
    let entity_id = primary_subject
        .and_then(|subject| subject.entity_id)
        .map(|id| id.0 as i64)
        .unwrap_or(-1);
    dict.set("entry_id", item.entry_id.0 as i64);
    dict.set("event_type", format!("{:?}", item.event_type));
    dict.set("cause_id", item.cause.id());
    dict.set("tick", item.end_tick as i64);
    dict.set("start_tick", item.start_tick as i64);
    dict.set("end_tick", item.end_tick as i64);
    dict.set("entity_id", entity_id);
    dict.set("significance", item.significance as f32);
    dict.set("title_key", item.headline.locale_key.as_str());
    dict.set("description", item.capsule.locale_key.as_str());
    dict.set("l10n_key", item.capsule.locale_key.as_str());
    dict.set("headline_key", item.headline.locale_key.as_str());
    dict.set(
        "headline_params",
        chronicle_layer_params_to_dict(&item.headline.params),
    );
    dict.set("capsule_key", item.capsule.locale_key.as_str());
    dict.set(
        "capsule_params",
        chronicle_layer_params_to_dict(&item.capsule.params),
    );
    dict.set("tile_x", item.location_ref.tile_x);
    dict.set("tile_y", item.location_ref.tile_y);
    dict.set("category_id", item.category.id());
    dict.set("queue_bucket", item.queue_bucket.id());
    dict.set(
        "l10n_params",
        chronicle_layer_params_to_dict(&item.capsule.params),
    );
    if let Some(subject) = primary_subject {
        if let Some(agent_name) = subject.display_name.as_deref() {
            dict.set("entity_name", agent_name);
        } else if let Some(agent_name) = item.capsule.params.get("agent") {
            dict.set("entity_name", agent_name.as_str());
        } else if let Some(agent_name) = item.headline.params.get("agent") {
            dict.set("entity_name", agent_name.as_str());
        }
    } else if let Some(agent_name) = item.capsule.params.get("agent") {
        dict.set("entity_name", agent_name.as_str());
    } else if let Some(agent_name) = item.headline.params.get("agent") {
        dict.set("entity_name", agent_name.as_str());
    }
    dict
}

fn chronicle_entry_lite_to_legacy_dict(entry: &ChronicleEntryLite) -> VarDictionary {
    let mut dict = VarDictionary::new();
    dict.set("entry_id", entry.entry_id.0 as i64);
    dict.set("tick", entry.end_tick as i64);
    dict.set("start_tick", entry.start_tick as i64);
    dict.set("end_tick", entry.end_tick as i64);
    dict.set("event_family", entry.event_family.as_str());
    dict.set("event_type", format!("{:?}", entry.event_type));
    dict.set(
        "entity_id",
        entry
            .entity_ref
            .entity_id
            .map(|entity_id| entity_id.0 as i64)
            .unwrap_or(-1),
    );
    dict.set("cause_id", entry.cause.id());
    dict.set("title_key", entry.headline.locale_key.as_str());
    dict.set("description", entry.capsule.locale_key.as_str());
    dict.set("l10n_key", entry.capsule.locale_key.as_str());
    dict.set("headline_key", entry.headline.locale_key.as_str());
    dict.set(
        "headline_params",
        chronicle_layer_params_to_dict(&entry.headline.params),
    );
    dict.set("capsule_key", entry.capsule.locale_key.as_str());
    dict.set(
        "capsule_params",
        chronicle_layer_params_to_dict(&entry.capsule.params),
    );
    dict.set("dossier_stub_key", entry.dossier_stub.locale_key.as_str());
    dict.set(
        "dossier_stub_params",
        chronicle_layer_params_to_dict(&entry.dossier_stub.params),
    );
    dict.set(
        "dossier_stub_tags",
        chronicle_detail_tags_to_array(&entry.dossier_stub.detail_tags),
    );
    dict.set("tile_x", entry.location_ref.tile_x);
    dict.set("tile_y", entry.location_ref.tile_y);
    dict.set("significance", entry.significance as f32);
    dict.set("category_id", entry.significance_category.id());
    dict.set("base_score", entry.significance_meta.base_score as f32);
    dict.set("cause_bonus", entry.significance_meta.cause_bonus as f32);
    dict.set("group_bonus", entry.significance_meta.group_bonus as f32);
    dict.set(
        "repeat_penalty",
        entry.significance_meta.repeat_penalty as f32,
    );
    dict.set("final_score", entry.significance_meta.final_score as f32);
    dict.set(
        "significance_reason_tags",
        chronicle_detail_tags_to_array(&entry.significance_meta.reason_tags),
    );
    dict.set("queue_bucket", entry.queue_bucket.id());
    dict.set("status", chronicle_entry_status_id(entry));
    dict.set(
        "surfaced_tick",
        entry.surfaced_tick.map(|tick| tick as i64).unwrap_or(-1),
    );
    dict.set(
        "displacement_reason",
        entry.displacement_reason.as_deref().unwrap_or_default(),
    );
    dict.set(
        "l10n_params",
        chronicle_layer_params_to_dict(&entry.capsule.params),
    );
    if let Some(agent_name) = entry.entity_ref.display_name.as_deref() {
        dict.set("entity_name", agent_name);
    } else if let Some(agent_name) = entry.capsule.params.get("agent") {
        dict.set("entity_name", agent_name.as_str());
    } else if let Some(agent_name) = entry.headline.params.get("agent") {
        dict.set("entity_name", agent_name.as_str());
    }
    dict
}

fn chronicle_entry_detail_snapshot_to_dict(
    snapshot: &ChronicleEntryDetailSnapshot,
) -> VarDictionary {
    let mut dict = VarDictionary::new();
    dict.set("snapshot_revision", snapshot.snapshot_revision.0 as i64);
    dict.set("revision_unavailable", snapshot.revision_unavailable);
    dict.set("available", snapshot.entry.is_some());
    if let Some(entry) = snapshot.entry.as_ref() {
        let mut subjects: Array<VarDictionary> = Array::new();
        subjects.push(&chronicle_subject_ref_lite_to_dict(&entry.entity_ref));
        dict.set("entry_id", entry.entry_id.0 as i64);
        dict.set("start_tick", entry.start_tick as i64);
        dict.set("end_tick", entry.end_tick as i64);
        dict.set("event_family", entry.event_family.as_str());
        dict.set("event_type", format!("{:?}", entry.event_type));
        dict.set("cause_id", entry.cause.id());
        dict.set("headline_key", entry.headline.locale_key.as_str());
        dict.set(
            "headline_params",
            chronicle_layer_params_to_dict(&entry.headline.params),
        );
        dict.set("capsule_key", entry.capsule.locale_key.as_str());
        dict.set(
            "capsule_params",
            chronicle_layer_params_to_dict(&entry.capsule.params),
        );
        dict.set("dossier_stub_key", entry.dossier_stub.locale_key.as_str());
        dict.set(
            "dossier_stub_params",
            chronicle_layer_params_to_dict(&entry.dossier_stub.params),
        );
        dict.set(
            "dossier_stub_tags",
            chronicle_detail_tags_to_array(&entry.dossier_stub.detail_tags),
        );
        dict.set("subjects", subjects);
        dict.set(
            "location",
            chronicle_location_ref_lite_to_dict(&entry.location_ref),
        );
        dict.set("significance", entry.significance as f32);
        dict.set("category_id", entry.significance_category.id());
        let mut significance_meta = VarDictionary::new();
        significance_meta.set("base_score", entry.significance_meta.base_score as f32);
        significance_meta.set("cause_bonus", entry.significance_meta.cause_bonus as f32);
        significance_meta.set("group_bonus", entry.significance_meta.group_bonus as f32);
        significance_meta.set(
            "repeat_penalty",
            entry.significance_meta.repeat_penalty as f32,
        );
        significance_meta.set("final_score", entry.significance_meta.final_score as f32);
        significance_meta.set(
            "reason_tags",
            chronicle_detail_tags_to_array(&entry.significance_meta.reason_tags),
        );
        dict.set("significance_meta", significance_meta);
        dict.set("queue_bucket", entry.queue_bucket.id());
        dict.set("status", chronicle_entry_status_id(entry));
        dict.set(
            "surfaced_tick",
            entry.surfaced_tick.map(|tick| tick as i64).unwrap_or(-1),
        );
        dict.set(
            "displacement_reason",
            entry.displacement_reason.as_deref().unwrap_or_default(),
        );
        let mut queue_transitions: Array<VarDictionary> = Array::new();
        for transition in &entry.queue_transitions {
            let mut transition_dict = VarDictionary::new();
            transition_dict.set("from", transition.from.id());
            transition_dict.set("to", transition.to.id());
            transition_dict.set("tick", transition.tick as i64);
            transition_dict.set("reason", transition.reason.as_str());
            queue_transitions.push(&transition_dict);
        }
        dict.set("queue_transitions", queue_transitions);
        let causal_links: Array<VarDictionary> = Array::new();
        dict.set("causal_links", causal_links);
    }
    dict
}

fn chronicle_feed_response_to_dict(response: &ChronicleFeedResponse) -> VarDictionary {
    let mut dict = VarDictionary::new();
    let mut items: Array<VarDictionary> = Array::new();
    for item in &response.items {
        items.push(&chronicle_feed_item_snapshot_to_dict(item));
    }
    dict.set("snapshot_revision", response.snapshot_revision.0 as i64);
    dict.set("revision_unavailable", response.revision_unavailable);
    dict.set("items", items);
    dict.set(
        "telemetry",
        chronicle_telemetry_to_dict(&response.telemetry),
    );
    dict
}

fn chronicle_feed_response_to_legacy_array(
    response: &ChronicleFeedResponse,
) -> Array<VarDictionary> {
    let mut out: Array<VarDictionary> = Array::new();
    for item in &response.items {
        out.push(&chronicle_feed_item_snapshot_to_legacy_dict(item));
    }
    out
}

fn chronicle_recall_slice_response_to_dict(
    response: &ChronicleRecallSliceResponse,
) -> VarDictionary {
    let mut dict = VarDictionary::new();
    let mut items: Array<VarDictionary> = Array::new();
    for item in &response.items {
        let mut item_dict = VarDictionary::new();
        item_dict.set("entry_id", item.entry_id.0 as i64);
        item_dict.set("queue_bucket", item.queue_bucket.id());
        item_dict.set("suppression_reason", item.suppression_reason.as_str());
        item_dict.set("suppressed_tick", item.suppressed_tick as i64);
        item_dict.set("recall_priority", item.recall_priority as f32);
        item_dict.set("cause_id", item.cause.id());
        item_dict.set("headline_key", item.headline.locale_key.as_str());
        item_dict.set(
            "headline_params",
            chronicle_layer_params_to_dict(&item.headline.params),
        );
        item_dict.set(
            "location",
            chronicle_location_ref_lite_to_dict(&item.location_ref),
        );
        items.push(&item_dict);
    }
    dict.set("snapshot_revision", response.snapshot_revision.0 as i64);
    dict.set("revision_unavailable", response.revision_unavailable);
    dict.set("items", items);
    dict
}

fn chronicle_history_slice_response_to_dict(
    response: &ChronicleHistorySliceResponse,
) -> VarDictionary {
    let mut dict = VarDictionary::new();
    let mut items: Array<VarDictionary> = Array::new();
    for item in &response.items {
        items.push(&chronicle_feed_item_snapshot_to_dict(item));
    }
    dict.set("snapshot_revision", response.snapshot_revision.0 as i64);
    dict.set("revision_unavailable", response.revision_unavailable);
    dict.set("items", items);
    dict.set(
        "telemetry",
        chronicle_telemetry_to_dict(&response.telemetry),
    );
    dict.set(
        "next_cursor_before_tick",
        response
            .next_cursor_before_tick
            .map(|tick| tick as i64)
            .unwrap_or(-1),
    );
    dict.set(
        "next_cursor_before_entry_id",
        response
            .next_cursor_before_entry_id
            .map(|entry_id| entry_id.0 as i64)
            .unwrap_or(-1),
    );
    dict
}

fn chronicle_thread_list_response_to_dict(response: &ChronicleThreadListResponse) -> VarDictionary {
    let mut dict = VarDictionary::new();
    let mut items: Array<VarDictionary> = Array::new();
    for item in &response.items {
        let mut item_dict = VarDictionary::new();
        let mut entry_ids = PackedInt64Array::new();
        for entry_id in &item.entry_ids {
            entry_ids.push(entry_id.0 as i64);
        }
        item_dict.set("thread_id", item.thread_id as i64);
        item_dict.set("state_id", item.state_id.as_str());
        item_dict.set("tension_score", item.tension_score as f32);
        item_dict.set("headline_key", item.headline.locale_key.as_str());
        item_dict.set(
            "headline_params",
            chronicle_layer_params_to_dict(&item.headline.params),
        );
        item_dict.set("entry_ids", entry_ids);
        item_dict.set("scope", item.scope.as_str());
        item_dict.set("started_tick", item.started_tick as i64);
        item_dict.set("last_entry_tick", item.last_entry_tick as i64);
        item_dict.set("entry_count", item.entry_count as i64);
        items.push(&item_dict);
    }
    dict.set("snapshot_revision", response.snapshot_revision.0 as i64);
    dict.set("revision_unavailable", response.revision_unavailable);
    dict.set("items", items);
    dict
}

fn chronicle_summary_keys(events: &[&ChronicleEvent]) -> PackedStringArray {
    let mut out = PackedStringArray::new();
    for event in events {
        out.push(&GString::from(event.summary_key.as_str()));
    }
    out
}

fn dominant_emotion_adjective(emotion: &Emotion) -> &'static str {
    let dominant_index = emotion
        .primary
        .iter()
        .enumerate()
        .max_by(|left, right| {
            left.1
                .partial_cmp(right.1)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(index, _)| index)
        .unwrap_or(0);
    match dominant_index {
        0 => "joyful",
        1 => "trusting",
        2 => "afraid",
        3 => "surprised",
        4 => "sad",
        5 => "disgusted",
        6 => "angry",
        7 => "restless",
        _ => "calm",
    }
}

fn need_motivation_sentence(needs: &Needs) -> Option<String> {
    let (need_key, value) = top_need_key_and_value(needs);
    if value >= sim_core::config::THOUGHT_TEXT_NEED_THRESHOLD {
        return None;
    }

    let sentence = match need_key {
        "NEED_HUNGER" => "Hunger is starting to bite.",
        "NEED_THIRST" => "Thirst keeps getting harder to ignore.",
        "NEED_SLEEP" => "Rest feels overdue.",
        "NEED_WARMTH" => "Cold is creeping in.",
        "NEED_SAFETY" => "Nothing feels completely safe right now.",
        "NEED_BELONGING" => "Being near others suddenly matters more.",
        "NEED_INTIMACY" => "Closeness feels painfully distant.",
        "NEED_RECOGNITION" => "Recognition still feels out of reach.",
        "NEED_AUTONOMY" => "Too much feels out of their control.",
        "NEED_COMPETENCE" => "They want to prove they can handle this.",
        "NEED_SELF_ACTUALIZATION" => "A larger purpose keeps tugging at them.",
        "NEED_MEANING" => "They keep searching for meaning in the moment.",
        "NEED_TRANSCENDENCE" => "Something beyond daily survival calls to them.",
        "NEED_ENERGY" => "Energy is running thin.",
        _ => "Something feels out of balance.",
    };
    Some(sentence.to_string())
}

fn entity_name_from_raw_id(
    world: &hecs::World,
    raw_lookup: &std::collections::HashMap<u64, u64>,
    raw_id: u64,
) -> Option<String> {
    let runtime_bits = runtime_bits_from_raw_id(raw_lookup, raw_id)?;
    let entity = hecs::Entity::from_bits(runtime_bits as u64)?;
    let identity = world.get::<&Identity>(entity).ok()?;
    Some(identity.name.clone())
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct StoryEventMessage {
    locale_key: String,
    params: Vec<(String, String)>,
}

fn format_story_event_locale(
    event: &SimEvent,
    actor_name: &str,
    target_name: Option<&str>,
) -> StoryEventMessage {
    let actor = actor_name.to_string();
    let target = target_name.unwrap_or("someone").to_string();
    match &event.event_type {
        SimEventType::NeedCritical => StoryEventMessage {
            locale_key: "STORY_NEED_CRITICAL".to_string(),
            params: vec![
                ("actor".to_string(), actor),
                ("need".to_string(), localized_need_label(&event.cause)),
            ],
        },
        SimEventType::NeedSatisfied => StoryEventMessage {
            locale_key: "STORY_NEED_SATISFIED".to_string(),
            params: vec![
                ("actor".to_string(), actor),
                ("need".to_string(), localized_need_label(&event.cause)),
            ],
        },
        SimEventType::EmotionShift => StoryEventMessage {
            locale_key: "STORY_EMOTION_SHIFT".to_string(),
            params: vec![
                ("actor".to_string(), actor),
                ("emotion".to_string(), localized_emotion_label(&event.cause)),
            ],
        },
        SimEventType::MoodChanged => StoryEventMessage {
            locale_key: "STORY_MOOD_CHANGED".to_string(),
            params: vec![("actor".to_string(), actor)],
        },
        SimEventType::StressEscalated => StoryEventMessage {
            locale_key: "STORY_STRESS_ESCALATED".to_string(),
            params: vec![("actor".to_string(), actor)],
        },
        SimEventType::MentalBreakStart => StoryEventMessage {
            locale_key: "STORY_MENTAL_BREAK".to_string(),
            params: vec![("actor".to_string(), actor)],
        },
        SimEventType::MentalBreakEnd => StoryEventMessage {
            locale_key: "STORY_MENTAL_BREAK_END".to_string(),
            params: vec![("actor".to_string(), actor)],
        },
        SimEventType::RelationshipFormed => StoryEventMessage {
            locale_key: "STORY_RELATIONSHIP_FORMED".to_string(),
            params: vec![
                ("actor".to_string(), actor),
                ("target".to_string(), target),
            ],
        },
        SimEventType::RelationshipBroken => StoryEventMessage {
            locale_key: "STORY_RELATIONSHIP_BROKEN".to_string(),
            params: vec![
                ("actor".to_string(), actor),
                ("target".to_string(), target),
            ],
        },
        SimEventType::SocialConflict => StoryEventMessage {
            locale_key: "STORY_SOCIAL_CONFLICT".to_string(),
            params: vec![
                ("actor".to_string(), actor),
                ("target".to_string(), target),
            ],
        },
        SimEventType::SocialCooperation => StoryEventMessage {
            locale_key: "STORY_SOCIAL_COOPERATION".to_string(),
            params: vec![
                ("actor".to_string(), actor),
                ("target".to_string(), target),
            ],
        },
        SimEventType::BandFormed => StoryEventMessage {
            locale_key: "STORY_BAND_FORMED".to_string(),
            params: vec![("actor".to_string(), actor)],
        },
        SimEventType::BandPromoted => StoryEventMessage {
            locale_key: "STORY_BAND_PROMOTED".to_string(),
            params: vec![("actor".to_string(), actor)],
        },
        SimEventType::BandSplit => StoryEventMessage {
            locale_key: "STORY_BAND_SPLIT".to_string(),
            params: vec![("actor".to_string(), actor)],
        },
        SimEventType::BandDissolved => StoryEventMessage {
            locale_key: "STORY_BAND_DISSOLVED".to_string(),
            params: vec![("actor".to_string(), actor)],
        },
        SimEventType::BandLeaderElected => StoryEventMessage {
            locale_key: "STORY_BAND_LEADER".to_string(),
            params: vec![("actor".to_string(), actor)],
        },
        SimEventType::LonerJoinedBand => StoryEventMessage {
            locale_key: "STORY_LONER_JOINED".to_string(),
            params: vec![("actor".to_string(), actor)],
        },
        SimEventType::ActionChanged => {
            let action = action_transition_parts(&event.cause)
                .map(|(_, to)| localized_status_label(to))
                .unwrap_or_else(|| localized_status_label(&event.cause));
            StoryEventMessage {
                locale_key: "STORY_ACTION_CHANGED".to_string(),
                params: vec![
                    ("actor".to_string(), actor),
                    ("action".to_string(), action),
                ],
            }
        }
        SimEventType::TaskCompleted => StoryEventMessage {
            locale_key: "STORY_TASK_COMPLETED".to_string(),
            params: vec![
                ("actor".to_string(), actor),
                ("task".to_string(), localized_status_label(&event.cause)),
            ],
        },
        SimEventType::Birth => StoryEventMessage {
            locale_key: "STORY_BIRTH".to_string(),
            params: vec![("actor".to_string(), actor)],
        },
        SimEventType::Death => StoryEventMessage {
            locale_key: "STORY_DEATH".to_string(),
            params: vec![
                ("actor".to_string(), actor),
                ("cause".to_string(), localized_death_label(&event.cause)),
            ],
        },
        SimEventType::AgeTransition => StoryEventMessage {
            locale_key: "STORY_AGE_TRANSITION".to_string(),
            params: vec![
                ("actor".to_string(), actor),
                ("stage".to_string(), localized_stage_label(&event.cause)),
            ],
        },
        SimEventType::FirstOccurrence => StoryEventMessage {
            locale_key: "STORY_FIRST_OCCURRENCE".to_string(),
            params: vec![(
                "detail".to_string(),
                humanize_status_text(&event.cause),
            )],
        },
        SimEventType::Custom(label) => StoryEventMessage {
            locale_key: "STORY_GENERIC".to_string(),
            params: vec![
                ("actor".to_string(), actor),
                ("detail".to_string(), humanize_status_text(label.as_str())),
            ],
        },
    }
}

fn format_story_event_message(
    event: &SimEvent,
    actor_name: &str,
    target_name: Option<&str>,
) -> String {
    match &event.event_type {
        SimEventType::NeedCritical => {
            format!(
                "{actor_name} is struggling with {}.",
                event.cause.replace('_', " ")
            )
        }
        SimEventType::NeedSatisfied => {
            format!(
                "{actor_name} recovered from {}.",
                event.cause.replace('_', " ")
            )
        }
        SimEventType::EmotionShift => {
            format!(
                "{actor_name}'s mood shifted toward {}.",
                event.cause.replace('_', " ")
            )
        }
        SimEventType::MoodChanged => format!("{actor_name} feels different now."),
        SimEventType::StressEscalated => format!("{actor_name}'s stress is rising."),
        SimEventType::MentalBreakStart => format!("{actor_name} broke down under the strain."),
        SimEventType::MentalBreakEnd => format!("{actor_name} is starting to recover."),
        SimEventType::RelationshipFormed => format!(
            "{actor_name} grew closer to {}.",
            target_name.unwrap_or("someone")
        ),
        SimEventType::RelationshipBroken => format!(
            "{actor_name} drifted apart from {}.",
            target_name.unwrap_or("someone")
        ),
        SimEventType::SocialConflict => format!(
            "{actor_name} clashed with {}.",
            target_name.unwrap_or("someone")
        ),
        SimEventType::SocialCooperation => format!(
            "{actor_name} worked with {}.",
            target_name.unwrap_or("someone")
        ),
        SimEventType::BandFormed => format!("{actor_name} helped form a new band."),
        SimEventType::BandPromoted => {
            format!("{actor_name}'s band is now established.")
        }
        SimEventType::BandSplit => format!("{actor_name}'s band split apart."),
        SimEventType::BandDissolved => format!("{actor_name}'s band dissolved."),
        SimEventType::BandLeaderElected => {
            format!("{actor_name} was chosen to lead the band.")
        }
        SimEventType::LonerJoinedBand => format!("{actor_name} joined a nearby band."),
        SimEventType::ActionChanged => {
            format!(
                "{actor_name} switched to {}.",
                humanize_status_text(&event.cause)
            )
        }
        SimEventType::TaskCompleted => {
            format!("{actor_name} finished {}.", event.cause.replace('_', " "))
        }
        SimEventType::Birth => format!("{actor_name} welcomed a new child."),
        SimEventType::Death => format!("{actor_name} died. {}", event.cause.replace('_', " ")),
        SimEventType::AgeTransition => {
            format!("{actor_name} entered a new stage of life.")
        }
        SimEventType::FirstOccurrence => {
            format!("A first happened: {}.", event.cause.replace('_', " "))
        }
        SimEventType::Custom(label) => format!("{actor_name}: {}", humanize_status_text(label)),
    }
}

fn recent_social_observation(
    store: &sim_engine::EventStore,
    current_tick: u64,
    raw_entity_id: u32,
    world: &hecs::World,
    raw_lookup: &std::collections::HashMap<u64, u64>,
) -> Option<String> {
    let since_tick =
        current_tick.saturating_sub(sim_core::config::THOUGHT_TEXT_EVENT_LOOKBACK_TICKS);
    for event in store.recent(store.len()) {
        if event.tick < since_tick {
            break;
        }
        let involves_entity = event.actor == raw_entity_id || event.target == Some(raw_entity_id);
        if !involves_entity {
            continue;
        }
        let other_raw_id = if event.actor == raw_entity_id {
            event.target.map(u64::from)
        } else {
            Some(u64::from(event.actor))
        };
        let other_name = other_raw_id
            .and_then(|raw_id| entity_name_from_raw_id(world, raw_lookup, raw_id))
            .unwrap_or_else(|| "someone nearby".to_string());
        let observation = match event.event_type {
            SimEventType::SocialConflict => {
                Some(format!("Tension with {other_name} still lingers."))
            }
            SimEventType::SocialCooperation => {
                Some(format!("Working with {other_name} felt steady."))
            }
            SimEventType::RelationshipFormed => {
                Some(format!("{other_name} feels a little closer now."))
            }
            SimEventType::RelationshipBroken => {
                Some(format!("A bond with {other_name} feels frayed."))
            }
            _ => None,
        };
        if observation.is_some() {
            return observation;
        }
    }
    None
}

fn is_low_significance_action_name(action: &str) -> bool {
    matches!(
        action,
        "Idle"
            | "Rest"
            | "Sleep"
            | "Eat"
            | "Drink"
            | "Wander"
            | "Forage"
            | "GatherWood"
            | "GatherStone"
            | "GatherHerbs"
            | "DeliverToStockpile"
            | "TakeFromStockpile"
            | "SeekShelter"
            | "SitByFire"
    )
}

fn action_transition_parts(cause: &str) -> Option<(&str, &str)> {
    let (from, to) = cause.split_once("->")?;
    Some((from.trim(), to.trim()))
}

fn is_significant_story_event(event: &SimEvent) -> bool {
    match event.event_type {
        SimEventType::ActionChanged => {
            let Some((from, to)) = action_transition_parts(&event.cause) else {
                return true;
            };
            if from == "Idle" || to == "Idle" {
                return false;
            }
            !(is_low_significance_action_name(from) && is_low_significance_action_name(to))
        }
        _ => true,
    }
}

fn recent_story_events_for_entity(
    store: &sim_engine::EventStore,
    raw_entity_id: u32,
    world: &hecs::World,
    raw_lookup: &std::collections::HashMap<u64, u64>,
) -> Array<VarDictionary> {
    let mut result: Array<VarDictionary> = Array::new();
    for event in store.recent(store.len()) {
        let involves_entity = event.actor == raw_entity_id || event.target == Some(raw_entity_id);
        if !involves_entity {
            continue;
        }
        if !is_significant_story_event(event) {
            continue;
        }
        let actor_name = entity_name_from_raw_id(world, raw_lookup, u64::from(event.actor))
            .unwrap_or_else(|| "Someone".to_string());
        let target_name = event
            .target
            .and_then(|raw_id| entity_name_from_raw_id(world, raw_lookup, u64::from(raw_id)));
        let localized_message =
            format_story_event_locale(event, actor_name.as_str(), target_name.as_deref());
        let mut row = VarDictionary::new();
        row.set("tick", event.tick as i64);
        row.set("kind", format!("{:?}", event.event_type));
        row.set("cause", event.cause.clone());
        row.set("message_key", localized_message.locale_key.as_str());
        let mut message_params = VarDictionary::new();
        for (key, value) in &localized_message.params {
            message_params.set(key.as_str(), value.as_str());
        }
        row.set("message_params", message_params);
        row.set(
            "message",
            format_story_event_message(event, actor_name.as_str(), target_name.as_deref()),
        );
        if let Some(target_raw_id) = event.target {
            if let Some(runtime_id) = runtime_bits_from_raw_id(raw_lookup, u64::from(target_raw_id))
            {
                row.set("target_id", runtime_id);
            }
        }
        result.push(&row);
        if result.len() >= sim_core::config::DETAIL_PANEL_RECENT_EVENT_LIMIT {
            break;
        }
    }
    result
}

#[derive(Debug, Clone, PartialEq)]
struct EntityListRowSnapshot {
    entity_id: i64,
    name: String,
    age_years: f32,
    sex: String,
    alive: bool,
    settlement_id: i64,
    band_id: i64,
    growth_stage: String,
    job: String,
    current_action: String,
    hunger: f32,
    is_leader: bool,
}

fn collect_entity_list_rows(
    world: &hecs::World,
    band_store: &sim_core::band::BandStore,
) -> Vec<EntityListRowSnapshot> {
    let mut result: Vec<EntityListRowSnapshot> = Vec::new();

    for (entity, (id, age, needs_opt, behavior_opt)) in world
        .query::<(&Identity, &Age, Option<&Needs>, Option<&Behavior>)>()
        .iter()
    {
        let job = behavior_opt.map(|behavior| behavior.job.clone()).unwrap_or_default();
        let current_action = behavior_opt
            .map(|behavior| format!("{:?}", behavior.current_action))
            .unwrap_or_else(|| "Idle".to_string());
        let entity_id = EntityId(entity.id() as u64);
        result.push(EntityListRowSnapshot {
            entity_id: entity.to_bits().get() as i64,
            name: id.name.clone(),
            age_years: age.years as f32,
            sex: runtime_sex_to_str(id.sex).to_string(),
            alive: age.alive,
            settlement_id: id.settlement_id.map(|settlement| settlement.0 as i64).unwrap_or(-1),
            band_id: runtime_queries::runtime_band_id_raw(id.band_id),
            growth_stage: runtime_growth_stage_to_str(id.growth_stage).to_string(),
            job,
            current_action,
            hunger: needs_opt
                .map(|needs| needs.get(NeedType::Hunger) as f32)
                .unwrap_or(0.0_f32),
            is_leader: id
                .band_id
                .and_then(|band_id| band_store.get(band_id))
                .map(|band| band.leader == Some(entity_id))
                .unwrap_or(false),
        });
    }

    result
}

fn bridge_entity_list(
    world: &hecs::World,
    band_store: &sim_core::band::BandStore,
) -> Array<VarDictionary> {
    let mut result: Array<VarDictionary> = Array::new();

    for row in collect_entity_list_rows(world, band_store) {
        let mut d = VarDictionary::new();
        d.set("entity_id", row.entity_id);
        d.set("name", row.name);
        d.set("age_years", row.age_years);
        d.set("sex", row.sex);
        d.set("alive", row.alive);
        d.set("settlement_id", row.settlement_id);
        d.set("band_id", row.band_id);
        d.set("growth_stage", row.growth_stage);
        d.set("job", row.job);
        d.set("current_action", row.current_action);
        d.set("hunger", row.hunger);
        d.set("is_leader", row.is_leader);
        result.push(&d);
    }

    result
}

fn build_thought_text(
    name: &str,
    emotion_adjective: Option<&str>,
    need_sentence: Option<&str>,
    social_sentence: Option<&str>,
    action_text: Option<&str>,
    stress_is_high: bool,
) -> String {
    let mut sentences: Vec<String> = Vec::new();
    if let Some(adjective) = emotion_adjective {
        sentences.push(format!("[b]{name} feels {adjective}.[/b]"));
    }
    if let Some(sentence) = need_sentence {
        sentences.push(sentence.to_string());
    }
    if stress_is_high {
        sentences.push("Tension is building. The weight is getting heavier.".to_string());
    }
    if let Some(sentence) = social_sentence {
        sentences.push(sentence.to_string());
    }
    if let Some(action) = action_text {
        if !action.trim().is_empty() {
            sentences.push(format!("Right now, {name} is {}.", action.to_lowercase()));
        }
    }
    if sentences.is_empty() {
        sentences.push(format!("[b]{name} feels steady for now.[/b]"));
        sentences.push("Nothing stands out enough to break the rhythm.".to_string());
    } else if sentences.len() == 1 {
        sentences.push(format!(
            "Right now, {name} is holding to the current rhythm."
        ));
    }
    sentences.into_iter().take(4).collect::<Vec<_>>().join(" ")
}

#[derive(Clone, Debug)]
struct NarrativeRecentEvent {
    event_type: SimEventType,
    cause: String,
    target_name: Option<String>,
}

#[derive(Clone, Debug)]
struct NarrativeRequestPlan {
    variant: LlmPromptVariant,
    recent_event_type: Option<String>,
    recent_event_cause: Option<String>,
    recent_target_name: Option<String>,
}

fn build_raw_name_lookup(world: &hecs::World) -> std::collections::HashMap<u32, String> {
    let mut lookup: std::collections::HashMap<u32, String> = std::collections::HashMap::new();
    let mut query = world.query::<&Identity>();
    for (entity, identity) in &mut query {
        lookup.insert(entity.id(), identity.name.clone());
    }
    lookup
}

fn latest_narrative_event_for_actor(
    store: &sim_engine::EventStore,
    actor: u32,
    since_tick: u64,
    names: &std::collections::HashMap<u32, String>,
) -> Option<NarrativeRecentEvent> {
    store
        .by_actor(actor, since_tick)
        .into_iter()
        .rev()
        .find(|event| {
            matches!(
                event.event_type,
                SimEventType::NeedCritical
                    | SimEventType::NeedSatisfied
                    | SimEventType::EmotionShift
                    | SimEventType::MoodChanged
                    | SimEventType::StressEscalated
                    | SimEventType::MentalBreakStart
                    | SimEventType::MentalBreakEnd
                    | SimEventType::RelationshipFormed
                    | SimEventType::RelationshipBroken
                    | SimEventType::SocialConflict
                    | SimEventType::SocialCooperation
                    | SimEventType::BandFormed
                    | SimEventType::BandPromoted
                    | SimEventType::BandSplit
                    | SimEventType::BandDissolved
                    | SimEventType::BandLeaderElected
                    | SimEventType::LonerJoinedBand
                    | SimEventType::ActionChanged
                    | SimEventType::TaskCompleted
                    | SimEventType::Birth
                    | SimEventType::Death
                    | SimEventType::AgeTransition
                    | SimEventType::FirstOccurrence
            )
        })
        .map(|event| NarrativeRecentEvent {
            event_type: event.event_type.clone(),
            cause: event.cause.clone(),
            target_name: event.target.and_then(|target| names.get(&target).cloned()),
        })
}

fn narrative_event_type_label(event_type: &SimEventType) -> &'static str {
    match event_type {
        SimEventType::NeedCritical => "need_critical",
        SimEventType::NeedSatisfied => "need_satisfied",
        SimEventType::EmotionShift => "emotion_shift",
        SimEventType::MoodChanged => "mood_changed",
        SimEventType::StressEscalated => "stress_escalated",
        SimEventType::MentalBreakStart => "mental_break_start",
        SimEventType::MentalBreakEnd => "mental_break_end",
        SimEventType::RelationshipFormed => "relationship_formed",
        SimEventType::RelationshipBroken => "relationship_broken",
        SimEventType::SocialConflict => "social_conflict",
        SimEventType::SocialCooperation => "social_cooperation",
        SimEventType::BandFormed => "band_formed",
        SimEventType::BandPromoted => "band_promoted",
        SimEventType::BandSplit => "band_split",
        SimEventType::BandDissolved => "band_dissolved",
        SimEventType::BandLeaderElected => "band_leader_elected",
        SimEventType::LonerJoinedBand => "loner_joined_band",
        SimEventType::ActionChanged => "action_changed",
        SimEventType::TaskCompleted => "task_completed",
        SimEventType::Birth => "birth",
        SimEventType::Death => "death",
        SimEventType::AgeTransition => "age_transition",
        SimEventType::FirstOccurrence => "first_occurrence",
        SimEventType::Custom(_) => "custom",
    }
}

fn narrative_cache_field_stale(
    cache: Option<&NarrativeCache>,
    current_tick: u64,
    field_present: impl Fn(&NarrativeCache) -> bool,
) -> bool {
    match cache {
        Some(value) if !field_present(value) => true,
        Some(value) => {
            current_tick.saturating_sub(value.cache_tick) >= u64::from(value.cache_ttl_ticks)
        }
        None => true,
    }
}

#[cfg(test)]
fn narrative_cache_has_any_text(cache: &NarrativeCache) -> bool {
    cache.personality_desc.is_some()
        || cache.last_event_narrative.is_some()
        || cache.last_inner_monologue.is_some()
}

fn narrative_cache_is_complete_and_fresh(cache: &NarrativeCache, current_tick: u64) -> bool {
    let has_full_cache = cache.personality_desc.is_some()
        && cache.last_event_narrative.is_some()
        && cache.last_inner_monologue.is_some();
    let cache_is_fresh =
        current_tick.saturating_sub(cache.cache_tick) < u64::from(cache.cache_ttl_ticks);
    has_full_cache && cache_is_fresh
}

fn pending_request_should_be_preempted(pending: &LlmPending, current_tick: u64) -> bool {
    let _ = current_tick;
    pending.request_type == LlmRequestType::Layer3Judgment
}

fn plan_narrative_request(
    cache: Option<&NarrativeCache>,
    current_tick: u64,
    recent_event: Option<&NarrativeRecentEvent>,
) -> Option<NarrativeRequestPlan> {
    if narrative_cache_field_stale(cache, current_tick, |value| {
        value.personality_desc.is_some()
    }) {
        return Some(NarrativeRequestPlan {
            variant: LlmPromptVariant::Personality,
            recent_event_type: None,
            recent_event_cause: None,
            recent_target_name: None,
        });
    }

    if let Some(event) = recent_event {
        if narrative_cache_field_stale(cache, current_tick, |value| {
            value.last_event_narrative.is_some()
        }) {
            return Some(NarrativeRequestPlan {
                variant: LlmPromptVariant::Narrative,
                recent_event_type: Some(narrative_event_type_label(&event.event_type).to_string()),
                recent_event_cause: if event.cause.is_empty() {
                    None
                } else {
                    Some(event.cause.clone())
                },
                recent_target_name: event.target_name.clone(),
            });
        }
    }

    if narrative_cache_field_stale(cache, current_tick, |value| {
        value.last_inner_monologue.is_some()
    }) {
        return Some(NarrativeRequestPlan {
            variant: LlmPromptVariant::Narrative,
            recent_event_type: None,
            recent_event_cause: None,
            recent_target_name: None,
        });
    }

    None
}

fn stress_state_code(stress: &Stress) -> u8 {
    match stress.state {
        sim_core::enums::StressState::Calm => 0,
        sim_core::enums::StressState::Alert => 1,
        sim_core::enums::StressState::Resistance => 2,
        sim_core::enums::StressState::Exhaustion => 3,
        sim_core::enums::StressState::Collapse => 4,
    }
}

fn build_click_narrative_request(
    world: &hecs::World,
    resources: &sim_engine::SimResources,
    entity: hecs::Entity,
    current_tick: u64,
) -> Option<LlmRequest> {
    let capable = world.get::<&LlmCapable>(entity).ok()?;
    let cache = world.get::<&NarrativeCache>(entity).ok();
    let lookback_ticks = cache
        .as_deref()
        .map(|value| u64::from(value.cache_ttl_ticks))
        .unwrap_or(u64::from(sim_core::config::LLM_CACHE_TTL_TICKS))
        .max(u64::from(capable.cooldown_ticks.max(60)));
    let name_lookup = build_raw_name_lookup(world);
    let recent_event = latest_narrative_event_for_actor(
        &resources.event_store,
        entity.id(),
        current_tick.saturating_sub(lookback_ticks),
        &name_lookup,
    );
    let plan = plan_narrative_request(cache.as_deref(), current_tick, recent_event.as_ref())?;

    let identity = world.get::<&Identity>(entity).ok()?;
    let personality = world.get::<&Personality>(entity).ok()?;
    let emotion = world.get::<&Emotion>(entity).ok()?;
    let behavior = world.get::<&Behavior>(entity).ok()?;
    let needs = world.get::<&Needs>(entity).ok()?;
    let stress = world.get::<&Stress>(entity).ok()?;
    let values = world.get::<&sim_core::components::Values>(entity).ok()?;

    Some(LlmRequest {
        request_id: 0,
        entity_id: entity.to_bits().get(),
        request_type: sim_core::components::LlmRequestType::Layer4Narrative,
        variant: plan.variant,
        entity_name: identity.name.clone(),
        role: capable.role,
        growth_stage: identity.growth_stage,
        sex: identity.sex,
        occupation: behavior.occupation.clone(),
        action_id: behavior.current_action as u32,
        action_label: behavior.current_action.to_string(),
        personality_axes: personality.axes,
        emotions: emotion.primary,
        needs: needs.values,
        values: values.values,
        stress_level: stress.level,
        stress_state: stress_state_code(&stress),
        recent_event_type: plan.recent_event_type,
        recent_event_cause: plan.recent_event_cause,
        recent_target_name: plan.recent_target_name,
    })
}

#[godot_api]
impl WorldSimRuntime {
    #[func]
    fn runtime_init(&mut self, seed: i64, config_json: GString) -> bool {
        let config = parse_runtime_config(&config_json.to_string());
        self.state = Some(RuntimeState::from_seed(seed.max(0) as u64, config));
        self.snapshot_buffer = SnapshotBuffer::new();
        self.render_alpha = 0.0;
        self.cached_agent_snapshots = Array::new();
        self.last_agent_snapshot_tick = 0;

        if let Some(state) = self.state.as_mut() {
            let registry_dir = authoritative_ron_data_dir();
            match sim_data::DataRegistry::load_from_directory(&registry_dir) {
                Ok(registry) => {
                    let registry = Arc::new(registry);
                    log::info!(
                        "[SimBridge] Authoritative RON registry loaded from {:?}: {} materials, {} recipes, {} furniture, {} structures, {} actions",
                        registry_dir,
                        registry.materials.len(),
                        registry.recipes.len(),
                        registry.furniture.len(),
                        registry.structures.len(),
                        registry.actions.len(),
                    );
                    state.engine.resources_mut().data_registry = Some(Arc::clone(&registry));
                    state.engine.resources_mut().apply_world_rules();
                    state.data_registry = Some(registry);
                }
                Err(errors) => {
                    log::warn!(
                        "[SimBridge] Could not load authoritative RON registry at {:?}: {:?}",
                        registry_dir,
                        errors
                    );
                }
            }

            let legacy_dir = legacy_json_data_dir();
            match load_legacy_runtime_bootstrap(&legacy_dir) {
                Ok(bootstrap) => {
                    state.engine.resources_mut().personality_distribution =
                        Some(bootstrap.personality_distribution);
                    state.engine.resources_mut().name_generator = Some(bootstrap.name_generator);
                    log::info!("[SimBridge] Legacy JSON compatibility bootstrap loaded");
                }
                Err(error) => {
                    log::warn!(
                        "[SimBridge] Could not load legacy JSON compatibility bootstrap at {:?}: {:?}",
                        legacy_dir,
                        error
                    );
                }
            }

            log::info!(
                "[SimBridge] Runtime initialized (empty world — awaiting runtime_spawn_agents)"
            );
        }

        true
    }

    #[func]
    fn runtime_is_initialized(&self) -> bool {
        self.state.is_some()
    }

    #[func]
    fn start_llm_server(&mut self) -> bool {
        let Some(state) = self.state.as_mut() else {
            return false;
        };
        state.engine.resources_mut().start_llm_server()
    }

    #[func]
    fn stop_llm_server(&mut self) {
        let Some(state) = self.state.as_mut() else {
            return;
        };
        state.engine.resources_mut().stop_llm_server();
    }

    #[func]
    fn is_llm_available(&self) -> bool {
        let Some(state) = self.state.as_ref() else {
            return false;
        };
        state.engine.resources().is_llm_available()
    }

    #[func]
    fn get_llm_status(&self) -> GString {
        let Some(state) = self.state.as_ref() else {
            return GString::from("{\"running\":false,\"model\":\"\",\"queue_depth\":0}");
        };
        let status = state.engine.resources().llm_status_json();
        GString::from(status.as_str())
    }

    #[func]
    fn drain_llm_debug_log(&self) -> Array<GString> {
        let mut output: Array<GString> = Array::new();
        let Some(state) = self.state.as_ref() else {
            return output;
        };
        for line in state.engine.resources().drain_llm_debug_log() {
            let godot_line = GString::from(line.as_str());
            output.push(&godot_line);
        }
        output
    }

    #[func]
    fn set_llm_quality(&mut self, quality: u8) {
        let Some(state) = self.state.as_mut() else {
            return;
        };
        state.engine.resources_mut().set_llm_quality(quality);
    }

    #[func]
    fn get_llm_quality(&self) -> u8 {
        let Some(state) = self.state.as_ref() else {
            return 0;
        };
        state.engine.resources().get_llm_quality()
    }

    /// Spawns agents into the Rust hecs world from a JSON array of spawn entries.
    /// Returns the number of agents successfully spawned.
    #[func]
    fn runtime_spawn_agents(&mut self, spawn_data_json: GString) -> i64 {
        let Some(state) = self.state.as_mut() else {
            return 0;
        };
        let json_str = spawn_data_json.to_string();
        let entries: Vec<SpawnEntry> = match serde_json::from_str(&json_str) {
            Ok(v) => v,
            Err(e) => {
                log::warn!("[SimBridge] runtime_spawn_agents parse error: {e}");
                return 0;
            }
        };

        let mut spawned_count: i64 = 0;
        for entry in &entries {
            let settlement_id = SettlementId(entry.settlement_id.unwrap_or(1));
            // Ensure settlement exists
            if !state
                .engine
                .resources()
                .settlements
                .contains_key(&settlement_id)
            {
                let sx = entry.settlement_x.unwrap_or(entry.x);
                let sy = entry.settlement_y.unwrap_or(entry.y);
                let mut settlement = Settlement::new(
                    settlement_id,
                    runtime_queries::generate_settlement_name(settlement_id),
                    sx,
                    sy,
                    0,
                );
                // Initialize all stone-age techs as Unknown so TechDiscovery can find them
                for tech_id in sim_core::STONE_AGE_TECH_IDS {
                    settlement.tech_states.insert(
                        tech_id.to_string(),
                        sim_core::TechState::Unknown,
                    );
                }
                state
                    .engine
                    .resources_mut()
                    .settlements
                    .insert(settlement_id, settlement);
            } else {
                // Patch existing settlements that have empty tech_states (created before init code)
                let sett = state.engine.resources_mut().settlements.get_mut(&settlement_id);
                if let Some(sett) = sett {
                    if sett.tech_states.is_empty() {
                        for tech_id in sim_core::STONE_AGE_TECH_IDS {
                            sett.tech_states.insert(
                                tech_id.to_string(),
                                sim_core::TechState::Unknown,
                            );
                        }
                        log::info!("[SimBridge] Patched settlement {} with {} stone-age techs",
                            settlement_id.0, sett.tech_states.len());
                    }
                }
            }

            let config = entity_spawner::SpawnConfig {
                settlement_id: Some(settlement_id),
                position: (entry.x, entry.y),
                initial_age_ticks: entry.age_ticks.unwrap_or(0),
                sex: None,
                parent_a: None,
                parent_b: None,
            };
            let (world, resources) = state.engine.world_and_resources_mut();
            entity_spawner::spawn_agent(world, resources, &config);
            spawned_count += 1;
        }

        state.engine.rebuild_frame_snapshots();
        self.snapshot_buffer
            .swap(state.engine.frame_snapshots().to_vec());

        log::info!(
            "[SimBridge] Spawned {} agents via runtime_spawn_agents",
            spawned_count
        );
        spawned_count
    }

    #[func]
    fn runtime_bootstrap_world(&mut self, setup_json: GString) -> VarDictionary {
        let Some(state) = self.state.as_mut() else {
            return VarDictionary::new();
        };
        let payload = match serde_json::from_str::<runtime_queries::RuntimeBootstrapPayload>(
            &setup_json.to_string(),
        ) {
            Ok(payload) => payload,
            Err(error) => {
                log::warn!("[SimBridge] runtime_bootstrap_world parse error: {error}");
                return VarDictionary::new();
            }
        };
        let out = bridge_bootstrap_world(state, payload);
        state.engine.rebuild_frame_snapshots();
        self.snapshot_buffer
            .swap(state.engine.frame_snapshots().to_vec());
        out
    }

    #[func]
    fn runtime_tick_frame(
        &mut self,
        delta_sec: f64,
        speed_index: i32,
        paused: bool,
    ) -> VarDictionary {
        let mut out = VarDictionary::new();
        let Some(state) = self.state.as_mut() else {
            out.set("initialized", false);
            out.set("current_tick", 0_i64);
            out.set("ticks_processed", 0_i64);
            out.set("speed_index", 0_i64);
            out.set("paused", true);
            out.set("accumulator", 0.0_f64);
            return out;
        };
        let tick_duration = 1.0_f64 / f64::from(state.ticks_per_second);

        if speed_index >= 0 {
            let clamped_speed = clamp_speed_index(speed_index);
            if clamped_speed != state.speed_index {
                state.speed_index = clamped_speed;
                state
                    .engine
                    .resources_mut()
                    .event_bus
                    .emit(GameEvent::SpeedChanged {
                        speed_index: clamped_speed,
                    });
            }
        }
        if paused != state.paused {
            state.paused = paused;
            if paused {
                state
                    .engine
                    .resources_mut()
                    .event_bus
                    .emit(GameEvent::SimulationPaused);
            } else {
                state
                    .engine
                    .resources_mut()
                    .event_bus
                    .emit(GameEvent::SimulationResumed);
            }
        }
        let mut ticks_processed: u32 = 0;

        if !paused {
            state.accumulator += delta_sec.max(0.0) * runtime_speed_multiplier(state.speed_index);
            while state.accumulator >= tick_duration && ticks_processed < state.max_ticks_per_frame
            {
                let emitted_tick = state.engine.current_tick() + 1;
                state
                    .engine
                    .resources_mut()
                    .event_bus
                    .emit(GameEvent::TickCompleted { tick: emitted_tick });
                state.engine.tick();
                state.accumulator -= tick_duration;
                ticks_processed += 1;
            }
            if state.accumulator > tick_duration * 3.0 {
                state.accumulator = 0.0;
            }
        }
        self.render_alpha = (state.accumulator / tick_duration).clamp(0.0, 1.0);
        if ticks_processed > 0 {
            self.snapshot_buffer
                .swap(state.engine.frame_snapshots().to_vec());
        } else if self.snapshot_buffer.agent_count() == 0 && !state.engine.world().is_empty() {
            state.engine.rebuild_frame_snapshots();
            self.snapshot_buffer
                .swap(state.engine.frame_snapshots().to_vec());
        }

        // Write debug snapshot for EditorPlugin every 60 ticks (file-based IPC).
        if state.engine.debug_mode && ticks_processed > 0 && state.engine.current_tick() % 60 == 0 {
            debug_api::write_debug_snapshot(state);
        }

        out.set("initialized", true);
        out.set("current_tick", state.engine.current_tick() as i64);
        out.set("ticks_processed", ticks_processed as i64);
        out.set("speed_index", state.speed_index as i64);
        out.set("paused", paused);
        out.set("accumulator", state.accumulator);

        // Build per-agent render snapshot for entity_renderer.gd
        // Only rebuild when ticks were processed or no cached snapshot exists yet.
        if ticks_processed > 0 || self.last_agent_snapshot_tick == 0 {
            let world = state.engine.world();
            let mut agent_arr = Array::<VarDictionary>::new();
            for (entity, (identity, pos, needs, behavior_opt, age_opt)) in world
                .query::<(
                    &Identity,
                    &Position,
                    &Needs,
                    Option<&Behavior>,
                    Option<&Age>,
                )>()
                .iter()
            {
                let mut d = VarDictionary::new();
                d.set("entity_id", entity.to_bits().get() as i64);
                d.set("x", pos.x);
                d.set("y", pos.y);
                d.set("name", GString::from(identity.name.as_str()));
                d.set("sex", GString::from(runtime_sex_to_str(identity.sex)));
                d.set(
                    "growth_stage",
                    GString::from(runtime_growth_stage_to_str(identity.growth_stage)),
                );
                if let Some(behavior) = behavior_opt {
                    d.set("job", GString::from(behavior.job.as_str()));
                    d.set(
                        "action",
                        GString::from(behavior.current_action.to_string().as_str()),
                    );
                } else {
                    d.set("job", GString::from("none"));
                    d.set("action", GString::from("idle"));
                }
                if let Some(age) = age_opt {
                    d.set("alive", age.alive);
                } else {
                    d.set("alive", true);
                }
                d.set("hunger", needs.get(NeedType::Hunger));
                agent_arr.push(&d);
            }
            self.cached_agent_snapshots = agent_arr;
            self.last_agent_snapshot_tick = state.engine.current_tick();
        }
        out.set("entity_count", state.engine.world().len() as i64);

        out
    }

    /// Returns cached agent snapshot Dictionaries (on-demand, not per-frame).
    #[func]
    fn get_agent_snapshots(&self) -> Array<VarDictionary> {
        self.cached_agent_snapshots.clone()
    }

    #[func]
    fn get_frame_snapshots(&self) -> PackedByteArray {
        encode_snapshot_bytes(self.snapshot_buffer.curr())
    }

    /// Returns all wildlife positions and state as a packed byte array (24 bytes per entity).
    #[func]
    fn get_wildlife_snapshots(&self) -> PackedByteArray {
        let Some(state) = self.state.as_ref() else {
            return PackedByteArray::new();
        };
        let snapshots = build_wildlife_snapshots(state.engine.world());
        let mut bytes: Vec<u8> = Vec::with_capacity(snapshots.len() * 24);
        for s in &snapshots {
            s.write_bytes(&mut bytes);
        }
        PackedByteArray::from(bytes)
    }

    #[func]
    fn get_prev_frame_snapshots(&self) -> PackedByteArray {
        encode_snapshot_bytes(self.snapshot_buffer.prev())
    }

    #[func]
    fn get_render_alpha(&self) -> f64 {
        self.render_alpha
    }

    #[func]
    fn get_agent_count(&self) -> i64 {
        self.snapshot_buffer.agent_count() as i64
    }

    #[func]
    fn runtime_get_snapshot(&self) -> PackedByteArray {
        let Some(state) = self.state.as_ref() else {
            return PackedByteArray::new();
        };
        let snapshot = state.engine.snapshot();
        let bytes = serde_json::to_vec(&snapshot).unwrap_or_default();
        PackedByteArray::from(bytes)
    }

    #[func]
    fn runtime_apply_snapshot(&mut self, snapshot_bytes: PackedByteArray) -> bool {
        let Some(state) = self.state.as_mut() else {
            return false;
        };
        let bytes = snapshot_bytes.as_slice();
        let Ok(snapshot) = serde_json::from_slice::<EngineSnapshot>(bytes) else {
            return false;
        };
        state.engine.restore_from_snapshot(&snapshot);
        true
    }

    #[func]
    fn runtime_save_ws2(&self, path: GString) -> bool {
        let Some(state) = self.state.as_ref() else {
            return false;
        };
        let path_string = path.to_string();
        if path_string.is_empty() {
            return false;
        }
        let snapshot = state.engine.snapshot();
        let Some(blob) = encode_ws2_blob(&snapshot) else {
            return false;
        };
        fs::write(path_string, blob).is_ok()
    }

    #[func]
    fn runtime_load_ws2(&mut self, path: GString) -> bool {
        let Some(state) = self.state.as_mut() else {
            return false;
        };
        let path_string = path.to_string();
        if path_string.is_empty() {
            return false;
        }
        let Ok(bytes) = fs::read(path_string) else {
            return false;
        };
        let Some(snapshot) = decode_ws2_blob(&bytes) else {
            return false;
        };
        state.engine.restore_from_snapshot(&snapshot);
        true
    }

    #[func]
    fn runtime_export_events_v2(&mut self) -> Array<VarDictionary> {
        let mut out: Array<VarDictionary> = Array::new();
        let Some(state) = self.state.as_mut() else {
            return out;
        };
        let mut drained: Vec<GameEvent> = Vec::new();
        if let Ok(mut events) = state.captured_events.lock() {
            drained.extend(events.drain(..));
        }
        for event in drained {
            let dict = game_event_to_v2_dict(&event);
            out.push(&dict);
        }
        out
    }

    #[func]
    fn drain_notifications(&mut self) -> Array<VarDictionary> {
        let mut out: Array<VarDictionary> = Array::new();
        let Some(state) = self.state.as_mut() else {
            return out;
        };
        let drained: Vec<_> = state
            .engine
            .resources_mut()
            .pending_notifications
            .drain(..)
            .collect();
        for notification in drained {
            let mut dict = VarDictionary::new();
            dict.set("tick", notification.tick as i64);
            dict.set("tier", notification.tier.as_i64());
            dict.set("kind", notification.kind.as_str());
            dict.set("importance", notification.importance);
            dict.set("primary_entity", notification.primary_entity as i64);
            dict.set(
                "secondary_entity",
                notification.secondary_entity.unwrap_or(0) as i64,
            );
            dict.set("message", notification.message_fallback.as_str());
            dict.set("position_x", notification.position_x);
            dict.set("position_y", notification.position_y);
            out.push(&dict);
        }
        out
    }

    #[func]
    fn runtime_get_registry_snapshot(&self) -> Array<VarDictionary> {
        let Some(state) = self.state.as_ref() else {
            return Array::new();
        };
        registry_snapshot(state)
    }

    #[func]
    fn runtime_get_chronicle_feed(&self, limit: i64, snapshot_revision: i64) -> VarDictionary {
        let Some(state) = self.state.as_ref() else {
            let mut out = VarDictionary::new();
            out.set("snapshot_revision", -1);
            out.set("revision_unavailable", true);
            out.set("items", Array::<VarDictionary>::new());
            return out;
        };
        let response = state.engine.resources().chronicle_timeline.feed_snapshot(
            limit.max(0) as usize,
            chronicle_snapshot_revision_from_arg(snapshot_revision),
        );
        chronicle_feed_response_to_dict(&response)
    }

    #[func]
    fn runtime_get_chronicle_entry_detail(
        &self,
        entry_id: i64,
        snapshot_revision: i64,
    ) -> VarDictionary {
        let Some(state) = self.state.as_ref() else {
            let mut out = VarDictionary::new();
            out.set("snapshot_revision", -1);
            out.set("revision_unavailable", true);
            out.set("available", false);
            return out;
        };
        let entry_id = match (entry_id > 0).then_some(entry_id as u64) {
            Some(raw_id) => ChronicleEntryId(raw_id),
            None => {
                let mut out = VarDictionary::new();
                out.set(
                    "snapshot_revision",
                    state
                        .engine
                        .resources()
                        .chronicle_timeline
                        .snapshot_revision()
                        .0 as i64,
                );
                out.set("revision_unavailable", false);
                out.set("available", false);
                return out;
            }
        };
        let response = state
            .engine
            .resources()
            .chronicle_timeline
            .entry_detail_snapshot(
                entry_id,
                chronicle_snapshot_revision_from_arg(snapshot_revision),
            );
        chronicle_entry_detail_snapshot_to_dict(&response)
    }

    #[func]
    fn runtime_get_story_threads(&self, limit: i64, snapshot_revision: i64) -> VarDictionary {
        let Some(state) = self.state.as_ref() else {
            let mut out = VarDictionary::new();
            out.set("snapshot_revision", -1);
            out.set("revision_unavailable", true);
            out.set("items", Array::<VarDictionary>::new());
            return out;
        };
        let response = state
            .engine
            .resources()
            .chronicle_timeline
            .story_threads_snapshot(
                limit.max(0) as usize,
                chronicle_snapshot_revision_from_arg(snapshot_revision),
            );
        chronicle_thread_list_response_to_dict(&response)
    }

    #[func]
    fn runtime_get_history_slice(
        &self,
        limit: i64,
        cursor_before_tick: i64,
        cursor_before_entry_id: i64,
        snapshot_revision: i64,
    ) -> VarDictionary {
        let Some(state) = self.state.as_ref() else {
            let mut out = VarDictionary::new();
            out.set("snapshot_revision", -1);
            out.set("revision_unavailable", true);
            out.set("items", Array::<VarDictionary>::new());
            out.set("next_cursor_before_tick", -1);
            out.set("next_cursor_before_entry_id", -1);
            return out;
        };
        let response = state
            .engine
            .resources()
            .chronicle_timeline
            .history_slice_snapshot(
                limit.max(0) as usize,
                (cursor_before_tick >= 0).then_some(cursor_before_tick as u64),
                (cursor_before_entry_id > 0)
                    .then_some(ChronicleEntryId(cursor_before_entry_id as u64)),
                chronicle_snapshot_revision_from_arg(snapshot_revision),
            );
        chronicle_history_slice_response_to_dict(&response)
    }

    #[func]
    fn runtime_get_recall_slice(&self, limit: i64, snapshot_revision: i64) -> VarDictionary {
        let Some(state) = self.state.as_ref() else {
            let mut out = VarDictionary::new();
            out.set("snapshot_revision", -1);
            out.set("revision_unavailable", true);
            out.set("items", Array::<VarDictionary>::new());
            return out;
        };
        let response = state
            .engine
            .resources()
            .chronicle_timeline
            .recall_slice_snapshot(
                limit.max(0) as usize,
                chronicle_snapshot_revision_from_arg(snapshot_revision),
            );
        chronicle_recall_slice_response_to_dict(&response)
    }

    #[func]
    fn runtime_get_chronicle_timeline(&self, limit: i64) -> Array<VarDictionary> {
        let Some(state) = self.state.as_ref() else {
            return Array::new();
        };
        let response = state
            .engine
            .resources()
            .chronicle_timeline
            .feed_snapshot(limit.max(0) as usize, None);
        chronicle_feed_response_to_legacy_array(&response)
    }

    #[func]
    fn runtime_register_default_systems(&mut self) -> i64 {
        let Some(state) = self.state.as_mut() else {
            return 0;
        };
        register_default_runtime_systems(state) as i64
    }

    #[func]
    fn runtime_get_compute_domain_modes(&self) -> VarDictionary {
        let mut out = VarDictionary::new();
        let Some(state) = self.state.as_ref() else {
            return out;
        };
        for (domain, mode) in &state.compute_domain_modes {
            out.set(domain.as_str(), mode.as_str());
        }
        out
    }

    #[func]
    fn runtime_apply_commands_v2(&mut self, commands: Array<VarDictionary>) {
        let Some(state) = self.state.as_mut() else {
            return;
        };
        apply_commands_v2(state, commands);
    }

    /// L2: Full entity detail snapshot — all 18 components, ~70 keys.
    /// Called when the player clicks an entity to open the detail panel.
    #[func]
    fn runtime_get_entity_detail(&self, entity_id: i64) -> VarDictionary {
        let mut dict = VarDictionary::new();
        let Some(state) = self.state.as_ref() else {
            return dict;
        };
        let entity = match resolve_runtime_entity(state.engine.world(), entity_id) {
            Some(e) => e,
            None => return dict,
        };
        let world = state.engine.world();
        let resources = state.engine.resources();
        let raw_lookup = runtime_queries::build_raw_entity_id_lookup(world);

        // Identity (required — return empty if missing)
        let Ok(id) = world.get::<&Identity>(entity) else {
            return dict;
        };
        let entity_raw_id = EntityId(entity.id() as u64);
        let selected_band_id = id.band_id;
        dict.set("entity_id", entity.to_bits().get() as i64);
        dict.set("name", id.name.clone());
        dict.set("sex", runtime_sex_to_str(id.sex));
        dict.set(
            "settlement_id",
            id.settlement_id.map(|s| s.0 as i64).unwrap_or(-1_i64),
        );
        dict.set("band_id", runtime_queries::runtime_band_id_raw(id.band_id));
        dict.set("band_name", "");
        dict.set("band_member_count", 0_i64);
        dict.set("band_is_promoted", false);
        dict.set("band_is_leader", false);
        dict.set("band_leader_name", "");
        let empty_band_members: Array<VarDictionary> = Array::new();
        dict.set("band_members", empty_band_members);
        if let Some(band_id) = selected_band_id {
            if let Some(band) = resources.band_store.get(band_id) {
                dict.set("band_name", band.name.as_str());
                dict.set("band_member_count", band.member_count() as i64);
                dict.set("band_is_promoted", band.is_promoted);
                dict.set("band_is_leader", band.leader == Some(entity_raw_id));
                dict.set(
                    "band_leader_name",
                    band.leader
                        .and_then(|leader_id| entity_name_from_raw_id(world, &raw_lookup, leader_id.0))
                        .unwrap_or_default(),
                );
                let mut member_arr: Array<VarDictionary> = Array::new();
                for &member_id in &band.members {
                    let Some(runtime_id) = runtime_bits_from_raw_id(&raw_lookup, member_id.0) else {
                        continue;
                    };
                    let mut member_dict = VarDictionary::new();
                    member_dict.set("entity_id", runtime_id);
                    member_dict.set(
                        "name",
                        entity_name_from_raw_id(world, &raw_lookup, member_id.0).unwrap_or_default(),
                    );
                    member_dict.set("is_leader", band.leader == Some(member_id));
                    member_arr.push(&member_dict);
                }
                dict.set("band_members", member_arr);
            }
        }
        dict.set("growth_stage", runtime_growth_stage_to_str(id.growth_stage));
        dict.set("zodiac", id.zodiac_sign.clone());
        dict.set("blood_type", id.blood_type.clone());
        dict.set("speech_tone", id.speech_tone.clone());
        dict.set("speech_verbosity", id.speech_verbosity.clone());
        dict.set("speech_humor", id.speech_humor.clone());
        drop(id);

        if let Ok(position) = world.get::<&Position>(entity) {
            dict.set("x", position.x);
            dict.set("y", position.y);
            dict.set("vel_x", position.vel_x);
            dict.set("vel_y", position.vel_y);
            dict.set("movement_dir", position.movement_dir as i64);
            let tile_x = position.tile_x();
            let tile_y = position.tile_y();
            if tile_x >= 0 && tile_y >= 0 && resources.map.in_bounds(tile_x, tile_y)
            {
                let warmth_influence = resources.influence_grid.sample(
                    tile_x as u32,
                    tile_y as u32,
                    ChannelId::Warmth,
                );
                dict.set("warmth_influence", warmth_influence as f32);

                // Room data — set defaults first so GDScript can always read these keys
                dict.set("room_id", -1_i64);
                dict.set("room_role", "ROOM_ROLE_UNKNOWN");
                dict.set("room_enclosed", false);
                let tile = resources.tile_grid.get(tile_x as u32, tile_y as u32);
                if let Some(room_id) = tile.room_id {
                    dict.set("room_id", room_id.0 as i64);
                    if let Some(room) = resources.rooms.iter().find(|r| r.id == room_id) {
                        dict.set("room_role", tile_info::room_role_locale_key(room.role));
                        dict.set("room_enclosed", room.enclosed);
                    }
                }
            }
        }

        // Age
        if let Ok(age) = world.get::<&Age>(entity) {
            dict.set("age_years", age.years as f32);
            dict.set("alive", age.alive);
        }

        // HEXACO personality axes (field is `axes`, not `hexaco`)
        if let Ok(pers) = world.get::<&Personality>(entity) {
            dict.set("hex_h", pers.axes[0] as f32);
            dict.set("hex_e", pers.axes[1] as f32);
            dict.set("hex_x", pers.axes[2] as f32);
            dict.set("hex_a", pers.axes[3] as f32);
            dict.set("hex_c", pers.axes[4] as f32);
            dict.set("hex_o", pers.axes[5] as f32);
            dict.set("archetype_key", archetype_label_key_from_axes(pers.axes));
        }
        if let Ok(temperament) = world.get::<&Temperament>(entity) {
            let td = temperament_detail::extract_temperament_detail(&temperament);
            dict.set("tci_ns", td.tci_ns);
            dict.set("tci_ha", td.tci_ha);
            dict.set("tci_rd", td.tci_rd);
            dict.set("tci_p", td.tci_p);
            dict.set("temperament_label_key", td.temperament_label_key);
            dict.set("temperament_awakened", td.temperament_awakened);
        }

        // Personal inventory (ItemStore-backed)
        if let Ok(inventory) = world.get::<&Inventory>(entity) {
            let mut inv_items: Array<VarDictionary> = Array::new();
            for &item_id in &inventory.items {
                if let Some(item) = state.engine.resources().item_store.get(item_id) {
                    let mut item_dict = VarDictionary::new();
                    item_dict.set("id", item.id.0 as i64);
                    item_dict.set("template_id", item.template_id.as_str());
                    item_dict.set("material_id", item.material_id.as_str());
                    item_dict.set("damage", item.derived_stats.damage as f32);
                    item_dict.set("speed", item.derived_stats.speed as f32);
                    item_dict.set("max_durability", item.derived_stats.max_durability as f32);
                    item_dict.set("current_durability", item.current_durability as f32);
                    item_dict.set("quality", item.quality as f32);
                    item_dict.set("stack_count", item.stack_count as i64);
                    item_dict.set("is_stackable", item.is_stackable());
                    item_dict.set(
                        "equipped_slot",
                        match &item.equipped_slot {
                            Some(EquipSlot::MainHand) => "main_hand",
                            Some(EquipSlot::OffHand) => "off_hand",
                            None => "",
                        },
                    );
                    inv_items.push(&item_dict);
                }
            }
            dict.set("inv_items", inv_items);
            dict.set("inv_item_count", inventory.count() as i64);
            dict.set("inv_max_tools", inventory.max_tool_slots as i64);
        }

        // Emotions (Plutchik 8)
        if let Ok(emo) = world.get::<&Emotion>(entity) {
            dict.set("emo_joy", emo.primary[0] as f32);
            dict.set("emo_trust", emo.primary[1] as f32);
            dict.set("emo_fear", emo.primary[2] as f32);
            dict.set("emo_surprise", emo.primary[3] as f32);
            dict.set("emo_sadness", emo.primary[4] as f32);
            dict.set("emo_disgust", emo.primary[5] as f32);
            dict.set("emo_anger", emo.primary[6] as f32);
            dict.set("emo_anticipation", emo.primary[7] as f32);
            let dom_names = [
                "joy",
                "trust",
                "fear",
                "surprise",
                "sadness",
                "disgust",
                "anger",
                "anticipation",
            ];
            let dom = emo
                .primary
                .iter()
                .enumerate()
                .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
                .map(|(i, _)| i)
                .unwrap_or(0);
            dict.set("dominant_emotion", dom_names[dom]);
        }

        // Needs (13 + energy)
        if let Ok(needs) = world.get::<&Needs>(entity) {
            let nv = &needs.values;
            dict.set("need_hunger", nv[0] as f32);
            dict.set("need_thirst", nv[1] as f32);
            dict.set("need_sleep", nv[2] as f32);
            dict.set("need_warmth", nv[3] as f32);
            dict.set("need_safety", nv[4] as f32);
            dict.set("need_belonging", nv[5] as f32);
            dict.set("need_intimacy", nv[6] as f32);
            dict.set("need_recognition", nv[7] as f32);
            dict.set("need_autonomy", nv[8] as f32);
            dict.set("need_competence", nv[9] as f32);
            dict.set("need_self_actualization", nv[10] as f32);
            dict.set("need_meaning", nv[11] as f32);
            dict.set("need_transcendence", nv[12] as f32);
            dict.set("energy", needs.energy as f32);
            let (top_need_key, top_need_value) = top_need_key_and_value(&needs);
            dict.set("top_need_key", top_need_key);
            dict.set("top_need_value", top_need_value as f32);
            let diagnostics = state
                .engine
                .resources()
                .agent_need_diagnostics
                .get(&EntityId(entity.id() as u64))
                .copied();
            dict.set(
                "need_hunger_delta",
                diagnostics.map(|entry| entry.hunger.delta).unwrap_or(0.0) as f32,
            );
            dict.set(
                "need_warmth_delta",
                diagnostics.map(|entry| entry.warmth.delta).unwrap_or(0.0) as f32,
            );
            dict.set(
                "need_safety_delta",
                diagnostics.map(|entry| entry.safety.delta).unwrap_or(0.0) as f32,
            );
            dict.set(
                "need_comfort",
                diagnostics
                    .map(|entry| entry.comfort.current)
                    .unwrap_or_else(|| diagnostic_comfort_score(nv[3], nv[4], nv[2]))
                    as f32,
            );
            dict.set(
                "need_comfort_delta",
                diagnostics.map(|entry| entry.comfort.delta).unwrap_or(0.0) as f32,
            );
            dict.set(
                "need_diagnostic_tick",
                diagnostics
                    .map(|entry| entry.last_tick as i64)
                    .unwrap_or(0_i64),
            );
        }

        // Stress
        if let Ok(stress) = world.get::<&Stress>(entity) {
            dict.set("stress_level", stress.level as f32);
            dict.set("stress_reserve", stress.reserve as f32);
            dict.set("allostatic_load", stress.allostatic_load as f32);
            dict.set("stress_state", format!("{:?}", stress.state));
            dict.set(
                "active_break",
                stress
                    .active_mental_break
                    .as_ref()
                    .map(|b| format!("{:?}", b))
                    .unwrap_or_default(),
            );
            dict.set("resilience", stress.resilience as f32);
        }

        // Body attributes
        dict.set("aggregate_hp", 1.0_f64);
        dict.set("damaged_groups", 0_i64);
        dict.set("active_conditions", 0_i64);
        dict.set("move_mult", 1.0_f64);
        dict.set("work_mult", 1.0_f64);
        dict.set("combat_mult", 1.0_f64);
        dict.set("pain", 0.0_f64);
        if let Ok(body) = world.get::<&Body>(entity) {
            dict.set("health", body.health);
            dict.set("aggregate_hp", f64::from(body.health));
            dict.set("body_str", body.str_realized);
            dict.set("body_agi", body.agi_realized);
            dict.set("body_end", body.end_realized);
            dict.set("body_tou", body.tou_realized);
            dict.set("body_rec", body.rec_realized);
            dict.set("body_dr", body.dr_realized);
            dict.set("attractiveness", body.attractiveness);
            dict.set("height", body.height);
        }
        if let Ok(body_health) = world.get::<&BodyHealth>(entity) {
            dict.set("aggregate_hp", body_health.aggregate_hp);
            dict.set("damaged_groups", body_health.damaged_groups as i64);
            dict.set("active_conditions", body_health.active_conditions as i64);
            dict.set("move_mult", body_health.move_mult());
            dict.set("work_mult", body_health.work_mult());
            dict.set("combat_mult", body_health.combat_mult());
            dict.set("pain", body_health.pain());
        }

        // Behavior / Job
        if let Ok(beh) = world.get::<&Behavior>(entity) {
            dict.set("job", beh.job.clone());
            dict.set("job_satisfaction", beh.job_satisfaction);
            dict.set("occupation", beh.occupation.clone());
            dict.set("current_action", format!("{:?}", beh.current_action));
            dict.set("action_progress", beh.action_progress as f32);
            dict.set("action_timer", beh.action_timer);
            dict.set("action_duration", beh.action_duration);
            dict.set("action_target_x", beh.action_target_x.unwrap_or(-1));
            dict.set("action_target_y", beh.action_target_y.unwrap_or(-1));
            dict.set("carry_total", beh.carry);
            dict.set("carry_capacity", sim_core::config::MAX_CARRY as f32);
            dict.set(
                "action_target_resource",
                action_target_resource_key(beh.current_action),
            );
        }

        // Active traits — PackedStringArray for string collections
        if let Ok(traits) = world.get::<&Traits>(entity) {
            let mut arr = PackedStringArray::new();
            for t in &traits.active {
                arr.push(&GString::from(t.as_str()));
            }
            dict.set("active_traits", arr);
        }

        let social_spouse = world.get::<&Social>(entity).ok().and_then(|social| social.spouse);
        let entity_raw_id = EntityId(entity.id() as u64);
        let mut summary_children: Vec<EntityId> =
            state.engine.resources().children_index.children_of(entity_raw_id).to_vec();
        // Social summary
        if let Ok(social) = world.get::<&Social>(entity) {
            summary_children.extend(social.children.iter().copied());
            summary_children.sort_unstable_by_key(|child| child.0);
            summary_children.dedup();
            dict.set(
                "spouse_id",
                social
                    .spouse
                    .and_then(|spouse| runtime_bits_from_raw_id(&raw_lookup, spouse.0))
                    .unwrap_or(-1_i64),
            );
            dict.set("relationship_count", social.edges.len() as i32);
        }
        dict.set("children_count", summary_children.len() as i32);
        dict.set("generation", 0_i64);
        dict.set("clan_id", -1_i64);
        dict.set("kinship_type", 0_i64);
        dict.set("has_father", false);
        dict.set("has_mother", false);
        dict.set("has_spouse", false);
        if let Ok(family) = world.get::<&FamilyComponent>(entity) {
            dict.set("generation", family.generation as i64);
            dict.set("clan_id", family.clan_id.map(i64::from).unwrap_or(-1_i64));
            dict.set("kinship_type", family.kinship_type as i64);
            dict.set("has_father", family.father.is_some());
            dict.set("has_mother", family.mother.is_some());
            dict.set("has_spouse", family.spouse.or(social_spouse).is_some());
        }
        dict.set("knowledge_count", 0_i64);
        dict.set("is_learning", false);
        dict.set("is_teaching", false);
        dict.set("innovation_potential", 0.0_f64);
        if let Ok(knowledge) = world.get::<&AgentKnowledge>(entity) {
            dict.set("knowledge_count", knowledge.known_count() as i64);
            dict.set("is_learning", knowledge.learning.is_some());
            dict.set("is_teaching", knowledge.teaching_target.is_some());
            dict.set("innovation_potential", knowledge.innovation_potential);
        }

        // Faith
        if let Ok(faith) = world.get::<&Faith>(entity) {
            dict.set("faith_tradition", faith.tradition.clone());
            dict.set("faith_strength", faith.strength as f32);
        }

        let recent_chronicle = state
            .engine
            .resources()
            .chronicle_log
            .query_by_entity(entity_raw_id, sim_core::config::DETAIL_PANEL_RECENT_EVENT_LIMIT);
        let mut chronicle_arr: Array<VarDictionary> = Array::new();
        for event in &recent_chronicle {
            chronicle_arr.push(&chronicle_event_to_dict(event));
        }
        dict.set("recent_chronicle_events", chronicle_arr);
        dict.set("recent_explains", chronicle_summary_keys(&recent_chronicle));
        if let Some(event) = recent_chronicle.first() {
            dict.set("recent_dominant_cause_key", event.summary_key.clone());
            dict.set("recent_influence_channel_id", event.cause.id());
        } else {
            dict.set("recent_dominant_cause_key", "");
            dict.set("recent_influence_channel_id", "");
        }
        let recent_entries = state
            .engine
            .resources()
            .chronicle_timeline
            .query_entries_by_entity(entity_raw_id, 3);
        let mut chronicle_summary_arr: Array<VarDictionary> = Array::new();
        for entry in recent_entries {
            chronicle_summary_arr.push(&chronicle_entry_lite_to_legacy_dict(entry));
        }
        dict.set("recent_chronicle_summaries", chronicle_summary_arr);

        // Causal log — recent events for this entity (newest first, max 8)
        let recent_causes = resources.causal_log.recent(entity_raw_id, 8);
        dict.set("recent_causes", causal_events_to_vardict(recent_causes));

        dict
    }

    /// L3: Tab-specific deep data. Tabs: mind / body / health / knowledge / family / skills / social / memory / misc.
    #[func]
    fn runtime_get_entity_tab(&self, entity_id: i64, tab: GString) -> VarDictionary {
        let mut dict = VarDictionary::new();
        let Some(state) = self.state.as_ref() else {
            return dict;
        };
        let entity = match resolve_runtime_entity(state.engine.world(), entity_id) {
            Some(e) => e,
            None => return dict,
        };
        let world = state.engine.world();
        let resources = state.engine.resources();
        let raw_lookup = runtime_queries::build_raw_entity_id_lookup(world);
        let entity_raw_id = EntityId(entity.id() as u64);
        let selected_band_id = world.get::<&Identity>(entity).ok().and_then(|id| id.band_id);

        match tab.to_string().as_str() {
            "mind" => {
                if let Ok(pers) = world.get::<&Personality>(entity) {
                    let mut facets = PackedFloat32Array::new();
                    for &f in &pers.facets {
                        facets.push(f as f32);
                    }
                    dict.set("facets", facets);
                }
                if let Ok(stress) = world.get::<&Stress>(entity) {
                    dict.set("gas_stage", stress.gas_stage);
                    dict.set("ace_score", stress.ace_score as f32);
                    let mut stressors: Array<VarDictionary> = Array::new();
                    for tr in &stress.stress_traces {
                        let mut sd = VarDictionary::new();
                        sd.set("source_id", tr.source_id.clone());
                        sd.set("per_tick", tr.per_tick);
                        sd.set("decay_rate", tr.decay_rate);
                        stressors.push(&sd);
                    }
                    dict.set("stressors", stressors);
                }
                if let Ok(mem) = world.get::<&Memory>(entity) {
                    let mut scars: Array<VarDictionary> = Array::new();
                    for s in &mem.trauma_scars {
                        let mut sd = VarDictionary::new();
                        sd.set("scar_id", s.scar_id.clone());
                        sd.set("severity", s.severity as f32);
                        sd.set("acquired_tick", s.acquired_tick as i64);
                        sd.set("reactivation_count", s.reactivation_count as i32);
                        scars.push(&sd);
                    }
                    dict.set("trauma_scars", scars);
                }
                if let Ok(vals) = world.get::<&Values>(entity) {
                    let mut values_arr = PackedFloat32Array::new();
                    for &v in &vals.values {
                        values_arr.push(v as f32);
                    }
                    dict.set("values_all", values_arr);
                }
                if let Ok(social) = world.get::<&Social>(entity) {
                    dict.set(
                        "attachment_type",
                        social
                            .attachment_type
                            .as_ref()
                            .map(|a| format!("{:?}", a))
                            .unwrap_or_else(|| "None".to_string()),
                    );
                }
                if let Ok(emo) = world.get::<&Emotion>(entity) {
                    let mut baselines = PackedFloat32Array::new();
                    for &b in &emo.baseline {
                        baselines.push(b as f32);
                    }
                    dict.set("emotion_baselines", baselines);
                }
            }
            "body" => {
                if let Ok(body) = world.get::<&Body>(entity) {
                    dict.set("str_pot", body.str_potential);
                    dict.set("str_train", body.str_trainability);
                    dict.set("str_real", body.str_realized);
                    dict.set("agi_pot", body.agi_potential);
                    dict.set("agi_train", body.agi_trainability);
                    dict.set("agi_real", body.agi_realized);
                    dict.set("end_pot", body.end_potential);
                    dict.set("end_train", body.end_trainability);
                    dict.set("end_real", body.end_realized);
                    dict.set("tou_pot", body.tou_potential);
                    dict.set("tou_train", body.tou_trainability);
                    dict.set("tou_real", body.tou_realized);
                    dict.set("rec_pot", body.rec_potential);
                    dict.set("rec_train", body.rec_trainability);
                    dict.set("rec_real", body.rec_realized);
                    dict.set("dr_pot", body.dr_potential);
                    dict.set("dr_train", body.dr_trainability);
                    dict.set("dr_real", body.dr_realized);
                    dict.set("attractiveness", body.attractiveness);
                    dict.set("height", body.height);
                    dict.set("health", body.health);
                    dict.set("innate_immunity", body.innate_immunity);
                    dict.set("blood_genotype", body.blood_genotype.clone());
                    dict.set(
                        "distinguishing_mark",
                        body.distinguishing_mark.clone().unwrap_or_default(),
                    );
                }
                if let Ok(intel) = world.get::<&Intelligence>(entity) {
                    let mut intel_arr = PackedFloat32Array::new();
                    for &v in &intel.values {
                        intel_arr.push(v as f32);
                    }
                    dict.set("intelligence", intel_arr);
                    dict.set("g_factor", intel.g_factor as f32);
                    dict.set("ace_penalty", intel.ace_penalty as f32);
                    dict.set("nutrition_penalty", intel.nutrition_penalty as f32);
                }
            }
            "health" => {
                dict.set("aggregate_hp", 1.0_f64);
                dict.set("damaged_groups", 0_i64);
                dict.set("lod_tier", 3_i64);
                dict.set("active_conditions", 0_i64);
                dict.set("move_mult", 1.0_f64);
                dict.set("work_mult", 1.0_f64);
                dict.set("combat_mult", 1.0_f64);
                dict.set("pain", 0.0_f64);
                dict.set("group_hp", vec_u8_to_packed(vec![100_u8; 10]));
                let empty_damaged_parts: Array<VarDictionary> = Array::new();
                dict.set("damaged_parts", empty_damaged_parts);
                if let Ok(body) = world.get::<&Body>(entity) {
                    dict.set("health", body.health);
                    dict.set("aggregate_hp", f64::from(body.health));
                }
                if let Ok(body_health) = world.get::<&BodyHealth>(entity) {
                    dict.set("aggregate_hp", body_health.aggregate_hp);
                    dict.set("damaged_groups", body_health.damaged_groups as i64);
                    dict.set("lod_tier", body_health.lod_tier as i64);
                    dict.set("active_conditions", body_health.active_conditions as i64);
                    dict.set("move_mult", body_health.move_mult());
                    dict.set("work_mult", body_health.work_mult());
                    dict.set("combat_mult", body_health.combat_mult());
                    dict.set("pain", body_health.pain());
                    dict.set("group_hp", vec_u8_to_packed(body_health.group_hp.to_vec()));

                    let mut damaged_parts: Array<VarDictionary> = Array::new();
                    for (index, part) in body_health.parts.iter().enumerate() {
                        if part.hp == 100 && !part.flags.any() {
                            continue;
                        }
                        let mut part_dict = VarDictionary::new();
                        part_dict.set("index", index as i64);
                        part_dict.set("name", PART_NAMES[index]);
                        part_dict.set("group", PART_TO_GROUP[index] as i64);
                        part_dict.set("hp", part.hp as i64);
                        part_dict.set("flags", part.flags.0 as i64);
                        part_dict.set("bleed_rate", part.bleed_rate as i64);
                        part_dict.set("infection_sev", part.infection_sev as i64);
                        part_dict.set("vital", PART_VITAL[index]);
                        damaged_parts.push(&part_dict);
                    }
                    dict.set("damaged_parts", damaged_parts);
                }
            }
            "knowledge" => {
                dict.set("innovation_potential", 0.0_f64);
                let empty_known: Array<VarDictionary> = Array::new();
                dict.set("known", empty_known);
                if let Ok(knowledge) = world.get::<&AgentKnowledge>(entity) {
                    dict.set("innovation_potential", knowledge.innovation_potential);
                    let mut known_arr: Array<VarDictionary> = Array::new();
                    for entry in &knowledge.known {
                        let mut kd = VarDictionary::new();
                        kd.set("id", entry.knowledge_id.as_str());
                        kd.set("proficiency", entry.proficiency);
                        kd.set("source", entry.source as i64);
                        kd.set("acquired_tick", entry.acquired_tick as i64);
                        kd.set("last_used_tick", entry.last_used_tick as i64);
                        kd.set("teacher_id", entry.teacher_id as i64);
                        known_arr.push(&kd);
                    }
                    dict.set("known", known_arr);

                    if let Some(learning) = &knowledge.learning {
                        let mut learning_dict = VarDictionary::new();
                        learning_dict.set("knowledge_id", learning.knowledge_id.as_str());
                        learning_dict.set("progress", learning.progress);
                        learning_dict.set("source", learning.source as i64);
                        learning_dict.set("teacher_id", learning.teacher_id as i64);
                        dict.set("learning", learning_dict);
                    }

                    if let Some((student_id, knowledge_id)) = &knowledge.teaching_target {
                        let mut teaching_dict = VarDictionary::new();
                        teaching_dict.set("student_id", *student_id as i64);
                        teaching_dict.set("knowledge_id", knowledge_id.as_str());
                        dict.set("teaching", teaching_dict);
                    }
                }
            }
            "family" => {
                dict.set("generation", 0_i64);
                dict.set("clan_id", -1_i64);
                dict.set("kinship_type", 0_i64);
                dict.set("birth_tick", 0_i64);
                dict.set("father", VarDictionary::new());
                dict.set("mother", VarDictionary::new());
                dict.set("spouse", VarDictionary::new());
                let empty_children: Array<VarDictionary> = Array::new();
                dict.set("children", empty_children);
                if let Ok(family) = world.get::<&FamilyComponent>(entity) {
                    dict.set("generation", family.generation as i64);
                    dict.set("clan_id", family.clan_id.map(i64::from).unwrap_or(-1_i64));
                    dict.set("kinship_type", family.kinship_type as i64);
                    dict.set("birth_tick", family.birth_tick as i64);

                    let mut father_dict = VarDictionary::new();
                    if let Some(father) = family.father {
                        if let Some(runtime_id) = runtime_bits_from_raw_id(&raw_lookup, father.0) {
                            father_dict.set("id", runtime_id);
                            father_dict.set(
                                "name",
                                entity_name_from_raw_id(world, &raw_lookup, father.0)
                                    .unwrap_or_default(),
                            );
                        }
                    }
                    dict.set("father", father_dict);

                    let mut mother_dict = VarDictionary::new();
                    if let Some(mother) = family.mother {
                        if let Some(runtime_id) = runtime_bits_from_raw_id(&raw_lookup, mother.0) {
                            mother_dict.set("id", runtime_id);
                            mother_dict.set(
                                "name",
                                entity_name_from_raw_id(world, &raw_lookup, mother.0)
                                    .unwrap_or_default(),
                            );
                        }
                    }
                    dict.set("mother", mother_dict);

                    let mut spouse_dict = VarDictionary::new();
                    let spouse = family
                        .spouse
                        .or_else(|| world.get::<&Social>(entity).ok().and_then(|social| social.spouse));
                    if let Some(spouse) = spouse {
                        if let Some(runtime_id) = runtime_bits_from_raw_id(&raw_lookup, spouse.0) {
                            spouse_dict.set("id", runtime_id);
                            spouse_dict.set(
                                "name",
                                entity_name_from_raw_id(world, &raw_lookup, spouse.0)
                                    .unwrap_or_default(),
                            );
                        }
                    }
                    dict.set("spouse", spouse_dict);

                    let mut children_arr: Array<VarDictionary> = Array::new();
                    let mut child_ids: Vec<EntityId> =
                        resources.children_index.children_of(entity_raw_id).to_vec();
                    if let Ok(social) = world.get::<&Social>(entity) {
                        child_ids.extend(social.children.iter().copied());
                    }
                    child_ids.sort_unstable_by_key(|child| child.0);
                    child_ids.dedup();
                    for child_raw in child_ids {
                        let Some(runtime_id) = runtime_bits_from_raw_id(&raw_lookup, child_raw.0)
                        else {
                            continue;
                        };
                        let Some(child_entity) = resolve_runtime_entity(world, runtime_id) else {
                            continue;
                        };
                        let mut child_dict = VarDictionary::new();
                        child_dict.set("id", runtime_id);
                        if let Ok(identity) = world.get::<&Identity>(child_entity) {
                            child_dict.set("name", identity.name.clone());
                        }
                        if let Ok(age) = world.get::<&Age>(child_entity) {
                            child_dict.set("age", age.years.round() as i64);
                        }
                        children_arr.push(&child_dict);
                    }
                    dict.set("children", children_arr);
                }
            }
            "skills" => {
                if let Ok(skills) = world.get::<&Skills>(entity) {
                    let mut arr: Array<VarDictionary> = Array::new();
                    let mut sorted: Vec<_> = skills.entries.iter().collect();
                    sorted.sort_by_key(|(k, _)| k.as_str());
                    for (id, entry) in sorted {
                        let mut sd = VarDictionary::new();
                        sd.set("id", id.clone());
                        sd.set("level", entry.level as i32);
                        sd.set("xp", entry.xp as f32);
                        arr.push(&sd);
                    }
                    dict.set("skills", arr);
                }
            }
            "social" => {
                if let Ok(social) = world.get::<&Social>(entity) {
                    let selected_band_members: Option<Vec<EntityId>> = selected_band_id
                        .and_then(|band_id| resources.band_store.get(band_id))
                        .map(|band| band.members.clone());
                    let mut spouse_dict = VarDictionary::new();
                    if let Some(s) = social.spouse {
                        if let Some(runtime_id) = runtime_bits_from_raw_id(&raw_lookup, s.0) {
                            spouse_dict.set("id", runtime_id);
                        }
                    }
                    dict.set("spouse", spouse_dict);
                    let mut parents_arr: Array<VarDictionary> = Array::new();
                    for p in &social.parents {
                        let Some(runtime_id) = runtime_bits_from_raw_id(&raw_lookup, p.0) else {
                            continue;
                        };
                        let mut pd = VarDictionary::new();
                        pd.set("id", runtime_id);
                        parents_arr.push(&pd);
                    }
                    dict.set("parents", parents_arr);
                    let mut children_arr: Array<VarDictionary> = Array::new();
                    for c in &social.children {
                        let Some(runtime_id) = runtime_bits_from_raw_id(&raw_lookup, c.0) else {
                            continue;
                        };
                        let mut cd = VarDictionary::new();
                        cd.set("id", runtime_id);
                        children_arr.push(&cd);
                    }
                    dict.set("children", children_arr);
                    let mut rel_arr: Array<VarDictionary> = Array::new();
                    for edge in social.edges.iter().take(15) {
                        let Some(runtime_id) = runtime_bits_from_raw_id(&raw_lookup, edge.target.0)
                        else {
                            continue;
                        };
                        let mut ed = VarDictionary::new();
                        ed.set("target_id", runtime_id);
                        ed.set("affinity", edge.affinity as f32);
                        ed.set("trust", edge.trust as f32);
                        ed.set("familiarity", edge.familiarity as f32);
                        ed.set("relation_type", format!("{:?}", edge.relation_type));
                        ed.set(
                            "is_band_mate",
                            selected_band_members
                                .as_ref()
                                .map(|members| members.contains(&edge.target))
                                .unwrap_or(false),
                        );
                        ed.set("last_interaction_tick", edge.last_interaction_tick as i64);
                        rel_arr.push(&ed);
                    }
                    dict.set("relationships", rel_arr);
                    dict.set("reputation_tags", PackedStringArray::new());
                    dict.set("social_class", format!("{:?}", social.social_class));
                }
                if let Ok(econ) = world.get::<&Economic>(entity) {
                    dict.set("saving_tendency", econ.saving_tendency as f32);
                    dict.set("risk_appetite", econ.risk_appetite as f32);
                    dict.set("generosity", econ.generosity as f32);
                    dict.set("materialism", econ.materialism as f32);
                    dict.set("wealth", econ.wealth as f32);
                }
            }
            "memory" => {
                if let Ok(mem) = world.get::<&Memory>(entity) {
                    let mut recent_arr: Array<VarDictionary> = Array::new();
                    for entry in mem.short_term.iter().rev().take(20) {
                        let mut ed = VarDictionary::new();
                        ed.set("event_type", entry.event_type.clone());
                        ed.set("tick", entry.tick as i64);
                        ed.set("intensity", entry.current_intensity as f32);
                        ed.set("is_permanent", entry.is_permanent);
                        recent_arr.push(&ed);
                    }
                    dict.set("recent_memories", recent_arr);
                    let mut perm_arr: Array<VarDictionary> = Array::new();
                    for entry in mem.permanent.iter().rev().take(20) {
                        let mut ed = VarDictionary::new();
                        ed.set("event_type", entry.event_type.clone());
                        ed.set("tick", entry.tick as i64);
                        ed.set("intensity", entry.intensity as f32);
                        ed.set("is_permanent", true);
                        perm_arr.push(&ed);
                    }
                    dict.set("permanent_memories", perm_arr);
                    let mut scars: Array<VarDictionary> = Array::new();
                    for s in &mem.trauma_scars {
                        let mut sd = VarDictionary::new();
                        sd.set("scar_id", s.scar_id.clone());
                        sd.set("severity", s.severity as f32);
                        sd.set("acquired_tick", s.acquired_tick as i64);
                        sd.set("reactivation_count", s.reactivation_count as i32);
                        scars.push(&sd);
                    }
                    dict.set("trauma_scars", scars);
                }
                dict.set(
                    "story_events",
                    recent_story_events_for_entity(
                        &state.engine.resources().event_store,
                        entity.id(),
                        world,
                        &raw_lookup,
                    ),
                );
                if let Ok(stress) = world.get::<&Stress>(entity) {
                    dict.set("ace_score", stress.ace_score as f32);
                }
                if let Ok(social) = world.get::<&Social>(entity) {
                    dict.set(
                        "attachment_type",
                        social
                            .attachment_type
                            .as_ref()
                            .map(|a| format!("{:?}", a))
                            .unwrap_or_else(|| "None".to_string()),
                    );
                }
            }
            "misc" => {
                if let Ok(id) = world.get::<&Identity>(entity) {
                    dict.set("zodiac", id.zodiac_sign.clone());
                    dict.set("blood_type", id.blood_type.clone());
                    dict.set("speech_tone", id.speech_tone.clone());
                    dict.set("speech_verbosity", id.speech_verbosity.clone());
                    dict.set("speech_humor", id.speech_humor.clone());
                    dict.set("pref_food", id.pref_food.clone());
                    dict.set("pref_color", id.pref_color.clone());
                    dict.set("pref_season", id.pref_season.clone());
                    let mut dislikes_arr = PackedStringArray::new();
                    for d in &id.dislikes {
                        dislikes_arr.push(&GString::from(d.as_str()));
                    }
                    dict.set("dislikes", dislikes_arr);
                }
                if let Ok(faith) = world.get::<&Faith>(entity) {
                    dict.set("faith_tradition", faith.tradition.clone());
                    dict.set("faith_strength", faith.strength as f32);
                    dict.set("is_priest", faith.is_priest);
                    dict.set("ritual_count", faith.ritual_count as i32);
                }
                if let Ok(coping) = world.get::<&Coping>(entity) {
                    dict.set(
                        "active_coping",
                        coping
                            .active_strategy
                            .as_ref()
                            .map(|s| format!("{:?}", s))
                            .unwrap_or_default(),
                    );
                    dict.set("dependency_score", coping.dependency_score);
                    dict.set("helplessness_score", coping.helplessness_score);
                    dict.set("break_count", coping.break_count as i32);
                }
            }
            _ => {} // unknown tab — return empty dict
        }

        dict
    }

    #[func]
    fn runtime_get_settlement_detail(&self, settlement_id: i64) -> VarDictionary {
        let Some(state) = self.state.as_ref() else {
            return VarDictionary::new();
        };
        bridge_settlement_detail(state, settlement_id)
    }

    #[func]
    fn runtime_get_building_detail(&self, building_id: i64) -> VarDictionary {
        let Some(state) = self.state.as_ref() else {
            return VarDictionary::new();
        };
        bridge_building_detail(state, building_id)
    }

    #[func]
    fn runtime_get_world_summary(&self) -> VarDictionary {
        let Some(state) = self.state.as_ref() else {
            return VarDictionary::new();
        };
        bridge_world_summary(state)
    }

    #[func]
    fn runtime_get_band_list(&self) -> Array<VarDictionary> {
        let Some(state) = self.state.as_ref() else {
            return Array::new();
        };
        let world = state.engine.world();
        let resources = state.engine.resources();
        let raw_lookup = runtime_queries::build_raw_entity_id_lookup(world);
        let mut result: Array<VarDictionary> = Array::new();

        for band in resources.band_store.all() {
            let mut dict = VarDictionary::new();
            dict.set("id", band.id.0 as i64);
            dict.set("name", band.name.as_str());
            dict.set("member_count", band.member_count() as i64);
            dict.set("is_promoted", band.is_promoted);

            let mut leader_runtime_id: i64 = -1;
            let mut leader_name = String::new();
            if let Some(leader_id) = band.leader {
                if let Some(runtime_id) = runtime_bits_from_raw_id(&raw_lookup, leader_id.0) {
                    // Snapshot stores entity.id() (lower 32 bits); mask to match.
                    leader_runtime_id = runtime_id & 0xFFFF_FFFF;
                }
                leader_name =
                    entity_name_from_raw_id(world, &raw_lookup, leader_id.0).unwrap_or_default();
            }

            dict.set("leader_id", leader_runtime_id);
            dict.set("leader_name", leader_name);

            // Member entity IDs in 32-bit snapshot format for overlay rendering.
            let mut member_ids = Array::<i64>::new();
            for member_id in &band.members {
                if let Some(runtime_id) = runtime_bits_from_raw_id(&raw_lookup, member_id.0) {
                    member_ids.push(runtime_id & 0xFFFF_FFFF);
                }
            }
            dict.set("member_ids", member_ids);

            result.push(&dict);
        }

        result
    }

    /// `runtime_get_band_detail` returns one band snapshot for the sidebar inspector.
    #[func]
    fn runtime_get_band_detail(&self, band_id: i64) -> VarDictionary {
        let mut dict = VarDictionary::new();
        let Some(state) = self.state.as_ref() else {
            return dict;
        };

        let world = state.engine.world();
        let resources = state.engine.resources();
        let raw_lookup = runtime_queries::build_raw_entity_id_lookup(world);
        let Some(band) = resources.band_store.get(BandId(band_id.max(0) as u64)) else {
            return dict;
        };

        dict.set("id", band.id.0 as i64);
        dict.set("name", band.name.as_str());
        dict.set("member_count", band.member_count() as i64);
        dict.set("is_promoted", band.is_promoted);
        dict.set("provisional_since", band.provisional_since as i64);
        dict.set(
            "promoted_tick",
            band.promoted_tick.map(|tick| tick as i64).unwrap_or(-1_i64),
        );

        let mut leader_dict = VarDictionary::new();
        if let Some(leader_id) = band.leader {
            if let Some(runtime_id) = runtime_bits_from_raw_id(&raw_lookup, leader_id.0) {
                leader_dict.set("id", runtime_id);
                leader_dict.set(
                    "name",
                    entity_name_from_raw_id(world, &raw_lookup, leader_id.0).unwrap_or_default(),
                );
            }
        }
        dict.set("leader", leader_dict);

        dict.set("settlement_id", band.settlement_id.map(|s| s.0 as i64).unwrap_or(-1i64));
        let settlement_name = band
            .settlement_id
            .and_then(|sid| resources.settlements.get(&sid))
            .map(|s| s.name.clone())
            .unwrap_or_default();
        dict.set("settlement_name", settlement_name);

        let mut members = Array::<VarDictionary>::new();
        for member_id in &band.members {
            let Some(runtime_id) = runtime_bits_from_raw_id(&raw_lookup, member_id.0) else {
                continue;
            };
            let Some(member_entity) = resolve_runtime_entity(world, runtime_id) else {
                continue;
            };
            let mut member_dict = VarDictionary::new();
            member_dict.set("id", runtime_id);
            member_dict.set(
                "is_leader",
                band.leader.map(|leader_id| leader_id == *member_id).unwrap_or(false),
            );

            if let Ok(identity) = world.get::<&Identity>(member_entity) {
                member_dict.set("name", identity.name.clone());
                member_dict.set("sex", format!("{:?}", identity.sex).to_lowercase());
            }
            if let Ok(age) = world.get::<&Age>(member_entity) {
                member_dict.set("age_years", age.years as f32);
            }
            if let Ok(behavior) = world.get::<&Behavior>(member_entity) {
                member_dict.set("current_action", behavior.current_action.to_string());
                member_dict.set("job", behavior.job.clone());
            }
            members.push(&member_dict);
        }
        dict.set("members", members);

        // Use dedicated band_events buffer — never evicted by movement events.
        // No member filter: the buffer only contains BandLifecycle events (max 200),
        // and filtering by current members drops events from agents who have since left.
        let mut aggregated_events: Vec<&ChronicleEvent> = resources
            .chronicle_log
            .recent_band_events(200);
        aggregated_events.sort_by_key(|event| std::cmp::Reverse(event.tick));
        aggregated_events.dedup_by(|left, right| {
            left.tick == right.tick
                && left.entity_id == right.entity_id
                && left.summary_key == right.summary_key
                && left.effect_key == right.effect_key
        });

        let mut event_arr = Array::<VarDictionary>::new();
        for event in aggregated_events.into_iter().take(20) {
            let mut event_dict = VarDictionary::new();
            event_dict.set("tick", event.tick as i64);
            event_dict.set("text", event.summary_key.clone());
            event_dict.set("text_key", event.summary_key.clone());
            event_dict.set("type", format!("{:?}", event.event_type));
            let mut params_dict = VarDictionary::new();
            for (key, value) in &event.summary_params {
                params_dict.set(key.as_str(), value.as_str());
            }
            event_dict.set("params", params_dict);
            event_arr.push(&event_dict);
        }
        dict.set("events", event_arr);

        dict
    }

    /// Returns dominant-faction territory texture for shader rendering.
    #[func]
    fn runtime_get_territory_texture(&self) -> VarDictionary {
        let Some(state) = self.state.as_ref() else {
            return VarDictionary::new();
        };
        let (faction_map, density_map, factions, grid_width, grid_height) = {
            let grid = &state.engine.resources().territory_grid;
            let (fm, dm) = grid.export_dominant();
            let fa = grid.active_factions();
            let w = grid.width;
            let h = grid.height;
            (fm, dm, fa, w, h)
        };

        let palette: [(f32, f32, f32); 8] = [
            (0.85, 0.45, 0.20),
            (0.30, 0.65, 0.85),
            (0.75, 0.35, 0.65),
            (0.40, 0.78, 0.40),
            (0.90, 0.72, 0.25),
            (0.55, 0.40, 0.80),
            (0.80, 0.30, 0.30),
            (0.30, 0.75, 0.70),
        ];
        let mut colors: Array<Vector3> = Array::new();
        for (i, _) in factions.iter().enumerate() {
            let (r, g, b) = palette[i % palette.len()];
            colors.push(Vector3::new(r, g, b));
        }

        // Per-faction border_hardness values — parallel array to colors.
        let mut hardness_arr: Array<f32> = Array::new();
        for fid in &factions {
            let h = state
                .engine
                .resources()
                .territory_hardness
                .get(fid)
                .copied()
                .unwrap_or(0.2);
            hardness_arr.push(h);
        }

        let mut dict = VarDictionary::new();
        dict.set("faction_ids", PackedByteArray::from(faction_map));
        dict.set("density", PackedByteArray::from(density_map));
        dict.set("colors", colors);
        dict.set("hardness", hardness_arr);
        dict.set("faction_count", factions.len() as i64);
        dict.set("width", grid_width as i64);
        dict.set("height", grid_height as i64);
        dict
    }

    /// Exports one influence-grid channel as normalized `u8` bytes for GPU heatmap rendering.
    #[func]
    fn runtime_get_influence_texture(&self, channel_name: GString) -> PackedByteArray {
        let Some(state) = self.state.as_ref() else {
            return PackedByteArray::new();
        };
        let channel_name = channel_name.to_string();
        let Some(channel) = ChannelId::from_key(channel_name.as_str()) else {
            return PackedByteArray::new();
        };
        let grid = &state.engine.resources().influence_grid;
        let (width, height) = grid.dimensions();
        let cell_count = (width * height) as usize;
        if cell_count == 0 {
            return PackedByteArray::new();
        }

        let data = grid.get_channel_data(channel);
        let mut min_val = f64::INFINITY;
        let mut max_val = f64::NEG_INFINITY;
        for &value in data.iter().take(cell_count) {
            min_val = min_val.min(value);
            max_val = max_val.max(value);
        }

        let mut bytes = Vec::with_capacity(cell_count);
        let range = max_val - min_val;
        if !min_val.is_finite() || !max_val.is_finite() || range.abs() <= f64::EPSILON {
            bytes.resize(cell_count, 0_u8);
        } else {
            for &value in data.iter().take(cell_count) {
                let normalized = ((value - min_val) / range).clamp(0.0, 1.0);
                bytes.push((normalized * 255.0).round() as u8);
            }
        }

        PackedByteArray::from(bytes)
    }

    #[func]
    fn runtime_get_influence_grid_size(&self) -> Vector2i {
        let Some(state) = self.state.as_ref() else {
            return Vector2i::new(0, 0);
        };
        let (width, height) = state.engine.resources().influence_grid.dimensions();
        Vector2i::new(width as i32, height as i32)
    }

    #[func]
    fn runtime_get_minimap_snapshot(&self) -> VarDictionary {
        let Some(state) = self.state.as_ref() else {
            return VarDictionary::new();
        };
        bridge_minimap_snapshot(state)
    }

    /// Tile-grid wall/floor/door/furniture data for the structural renderer.
    /// Returns packed arrays for efficient GDScript drawing.
    #[func]
    fn get_tile_grid_walls(&self) -> VarDictionary {
        let Some(state) = self.state.as_ref() else {
            return VarDictionary::new();
        };
        runtime_queries::tile_grid_walls(state)
    }

    /// Number of pending wall plans in the simulation.
    /// Used by building_renderer debug logging.
    #[func]
    fn get_wall_plans_count(&self) -> i64 {
        let Some(state) = self.state.as_ref() else { return -1; };
        state.engine.resources().wall_plans.len() as i64
    }

    /// Returns tile-grid structural data for a single tile coordinate.
    /// Used by the tile-click info panel to display wall/floor/furniture/room
    /// details for the clicked tile. Returns an empty dictionary if the
    /// coordinate is out of bounds or the simulation is not running.
    /// Delegates to [`tile_info::extract_tile_info`] for the core extraction logic.
    #[func]
    fn get_tile_info(&self, tile_x: i64, tile_y: i64) -> VarDictionary {
        let Some(state) = self.state.as_ref() else {
            return VarDictionary::new();
        };
        let resources = state.engine.resources();
        let Some(info) = tile_info::extract_tile_info(
            &resources.tile_grid,
            &resources.rooms,
            tile_x as i32,
            tile_y as i32,
        ) else {
            return VarDictionary::new();
        };

        // Return empty dict for completely empty tiles (no structural data)
        if !info.has_structural_data() {
            return VarDictionary::new();
        }

        let mut out = VarDictionary::new();

        // Wall info
        out.set("has_wall", info.has_wall);
        if let Some(ref mat) = info.wall_material {
            out.set("wall_material", GString::from(mat.as_str()));
        }
        out.set("wall_hp", info.wall_hp);
        out.set("is_door", info.is_door);

        // Floor info
        out.set("has_floor", info.has_floor);
        if let Some(ref mat) = info.floor_material {
            out.set("floor_material", GString::from(mat.as_str()));
        }

        // Furniture info
        out.set("has_furniture", info.has_furniture);
        if let Some(ref fid) = info.furniture_id {
            out.set("furniture_id", GString::from(fid.as_str()));
        }

        // Room info — room_role is a locale-safe key (lowercase)
        if let Some(room_id) = info.room_id {
            out.set("room_id", room_id as i64);
            if let Some(ref role_key) = info.room_role_key {
                out.set("room_role", GString::from(role_key.as_str()));
            }
            if let Some(enclosed) = info.room_enclosed {
                out.set("room_enclosed", enclosed);
            }
            if let Some(tile_count) = info.room_tile_count {
                out.set("room_tile_count", tile_count as i64);
            }
        }

        out.set("tile_x", tile_x);
        out.set("tile_y", tile_y);
        out
    }

    /// Entity list snapshot — lightweight summary of every agent.
    /// Used by the population list panel.
    #[func]
    fn runtime_get_entity_list(&self) -> Array<VarDictionary> {
        let Some(state) = self.state.as_ref() else {
            return Array::new();
        };
        bridge_entity_list(state.engine.world(), &state.engine.resources().band_store)
    }

    #[func]
    fn get_archetype_label(&self, entity_id: i64) -> GString {
        let Some(state) = self.state.as_ref() else {
            return GString::new();
        };
        let Some(entity) = resolve_runtime_entity(state.engine.world(), entity_id) else {
            return GString::new();
        };
        let world = state.engine.world();
        let Ok(personality) = world.get::<&Personality>(entity) else {
            return GString::new();
        };
        GString::from(archetype_label_key_from_axes(personality.axes))
    }

    #[func]
    fn get_thought_text(&self, entity_id: i64) -> GString {
        let Some(state) = self.state.as_ref() else {
            return GString::new();
        };
        let Some(entity) = resolve_runtime_entity(state.engine.world(), entity_id) else {
            return GString::new();
        };
        let world = state.engine.world();
        let raw_lookup = runtime_queries::build_raw_entity_id_lookup(world);
        let raw_entity_id = entity.id();

        let name = world
            .get::<&Identity>(entity)
            .ok()
            .map(|identity| identity.name.clone())
            .unwrap_or_else(|| "Someone".to_string());
        let emotion_adjective = world
            .get::<&Emotion>(entity)
            .ok()
            .map(|emotion| dominant_emotion_adjective(&emotion));
        let need_sentence = world
            .get::<&Needs>(entity)
            .ok()
            .and_then(|needs| need_motivation_sentence(&needs));
        let stress_is_high = world
            .get::<&Stress>(entity)
            .ok()
            .map(|stress| {
                matches!(
                    stress.state,
                    sim_core::enums::StressState::Resistance
                        | sim_core::enums::StressState::Exhaustion
                        | sim_core::enums::StressState::Collapse
                ) || stress.active_mental_break.is_some()
            })
            .unwrap_or(false);
        let social_sentence = recent_social_observation(
            &state.engine.resources().event_store,
            state.engine.current_tick(),
            raw_entity_id,
            world,
            &raw_lookup,
        );
        let action_text = world.get::<&Behavior>(entity).ok().map(|behavior| {
            humanize_status_text(format!("{:?}", behavior.current_action).as_str())
        });

        let thought_text = build_thought_text(
            name.as_str(),
            emotion_adjective,
            need_sentence.as_deref(),
            social_sentence.as_deref(),
            action_text.as_deref(),
            stress_is_high,
        );
        GString::from(thought_text.as_str())
    }

    #[func]
    fn get_narrative_display(&mut self, entity_id: i64) -> VarDictionary {
        let Some(state) = self.state.as_mut() else {
            return narrative_display_to_dict(&NarrativeDisplayData::default());
        };
        let tick = state.engine.current_tick();
        let (world, resources) = state.engine.world_and_resources_mut();
        drain_and_apply_llm_responses(world, resources, tick);
        let Some(entity) = resolve_runtime_entity(state.engine.world(), entity_id) else {
            return narrative_display_to_dict(&NarrativeDisplayData::default());
        };
        let display = build_narrative_display(
            state.engine.world(),
            state.engine.resources(),
            entity,
            entity.to_bits().get(),
        );
        narrative_display_to_dict(&display)
    }

    #[func]
    fn on_entity_narrative_click(&mut self, entity_id: i64) -> u8 {
        let Some(state) = self.state.as_mut() else {
            return 0;
        };
        let tick = state.engine.current_tick();
        let (world, resources) = state.engine.world_and_resources_mut();
        drain_and_apply_llm_responses(world, resources, tick);
        state.engine.resources().llm_runtime.push_debug_log(format!(
            "[LLM-DEBUG] on_entity_narrative_click called for entity {}",
            entity_id
        ));
        let llm_quality = state.engine.resources().get_llm_quality();
        let llm_running = state.engine.resources().llm_runtime.is_running();
        state.engine.resources().llm_runtime.push_debug_log(format!(
            "[LLM-DEBUG] on_entity_narrative_click state quality={} running={}",
            llm_quality, llm_running
        ));
        if llm_quality == 0 || !llm_running {
            state
                .engine
                .resources()
                .llm_runtime
                .push_debug_log("[LLM-DEBUG] on_entity_narrative_click returning 3".to_string());
            return 3;
        }

        let current_tick = state.engine.current_tick();
        let Some(entity) = resolve_runtime_entity(state.engine.world(), entity_id) else {
            state.engine.resources().llm_runtime.push_debug_log(
                "[LLM-DEBUG] on_entity_narrative_click returning 0 (entity not found)".to_string(),
            );
            return 0;
        };
        let pending_info = state
            .engine
            .world()
            .get::<&LlmPending>(entity)
            .ok()
            .map(|pending| {
                (
                    pending.request_id,
                    pending.request_type,
                    current_tick.saturating_sub(pending.submitted_tick),
                    pending_request_should_be_preempted(&pending, current_tick),
                )
            });
        if let Some((pending_request_id, pending_request_type, pending_age, should_preempt)) =
            pending_info
        {
            state
                .engine
                .resources()
                .llm_runtime
                .push_debug_log(format!(
                    "[LLM-DEBUG] entity {} already has pending request id={} type={:?} age_ticks={} preempt={}",
                    entity_id,
                    pending_request_id,
                    pending_request_type,
                    pending_age,
                    should_preempt
                ));
            if !should_preempt {
                state.engine.resources().llm_runtime.push_debug_log(
                    "[LLM-DEBUG] on_entity_narrative_click returning 2".to_string(),
                );
                return 2;
            }
            let _ = state
                .engine
                .resources_mut()
                .take_llm_request_meta(pending_request_id);
            let _ = state.engine.world_mut().remove_one::<LlmPending>(entity);
        }

        if let Ok(cache) = state.engine.world().get::<&NarrativeCache>(entity) {
            if narrative_cache_is_complete_and_fresh(&cache, current_tick) {
                state.engine.resources().llm_runtime.push_debug_log(
                    "[LLM-DEBUG] on_entity_narrative_click returning 0 (fresh cache)".to_string(),
                );
                return 0;
            }
        }

        let request = {
            let world = state.engine.world();
            let resources = state.engine.resources();
            build_click_narrative_request(world, resources, entity, current_tick)
        };
        let Some(request) = request else {
            state.engine.resources().llm_runtime.push_debug_log(
                "[LLM-DEBUG] on_entity_narrative_click returning 0 (no request plan)".to_string(),
            );
            return 0;
        };

        let request_id = match state
            .engine
            .resources_mut()
            .submit_priority_llm_request(request.clone())
        {
            Ok(value) => value,
            Err(error) => {
                state.engine.resources().llm_runtime.push_debug_log(format!(
                    "[LLM-DEBUG] on_entity_narrative_click submit failed: {}",
                    error
                ));
                return 0;
            }
        };

        {
            let world = state.engine.world_mut();
            let pending = LlmPending {
                request_id,
                request_type: request.request_type,
                submitted_tick: current_tick,
                timeout_ticks: sim_core::config::LLM_TIMEOUT_TICKS,
            };
            let _ = world.insert_one(entity, pending);
            if let Ok(mut capable) = world.get::<&mut LlmCapable>(entity) {
                capable.last_request_tick = current_tick;
            }
        }
        state.engine.resources().llm_runtime.push_debug_log(format!(
            "[LLM-DEBUG] on_entity_narrative_click returning 1 (queued request_id={})",
            request_id
        ));
        1
    }

    // ── Debug API ──────────────────────────────────────────────────────────────

    /// Enables or disables debug mode (controls PerfTracker overhead).
    #[func]
    fn enable_debug_mode(&mut self, enabled: bool) {
        if let Some(state) = self.state.as_mut() {
            debug_api::enable_debug(state, enabled);
        }
    }

    /// Returns a summary of the current simulation state.
    #[func]
    fn get_debug_summary(&self) -> VarDictionary {
        let Some(state) = self.state.as_ref() else {
            return VarDictionary::new();
        };
        debug_api::get_debug_summary(state)
    }

    /// Returns per-system timing data (populated when debug_mode is true).
    #[func]
    fn get_system_perf(&self) -> VarDictionary {
        let Some(state) = self.state.as_ref() else {
            return VarDictionary::new();
        };
        debug_api::get_system_perf(state)
    }

    /// Returns last 300 tick durations in milliseconds.
    #[func]
    fn get_tick_history(&self) -> PackedFloat32Array {
        let Some(state) = self.state.as_ref() else {
            return PackedFloat32Array::new();
        };
        debug_api::get_tick_history(state)
    }

    /// Returns all SimConfig key-value pairs.
    #[func]
    fn get_config_values(&self) -> VarDictionary {
        let Some(state) = self.state.as_ref() else {
            return VarDictionary::new();
        };
        debug_api::get_config_values(state)
    }

    /// Sets a SimConfig value by key. Returns false if the key doesn't exist.
    #[func]
    fn set_config_value(&mut self, key: GString, value: f64) -> bool {
        let Some(state) = self.state.as_mut() else {
            return false;
        };
        debug_api::set_config_value(state, &key.to_string(), value)
    }

    /// Returns guardrail status for all 9 guardrails.
    #[func]
    fn get_guardrail_status(&self) -> Array<VarDictionary> {
        let Some(state) = self.state.as_ref() else {
            return Array::new();
        };
        debug_api::get_guardrail_status(state)
    }

    /// Queries entities by condition and returns matching entity IDs.
    ///
    /// Supported conditions: `"stress_gte"`, `"health_lte"`, `"hunger_lte"`.
    #[func]
    fn query_entities_by_condition(&self, condition: GString, threshold: f64) -> PackedInt32Array {
        let Some(state) = self.state.as_ref() else {
            return PackedInt32Array::new();
        };
        debug_api::query_entities_by_condition(state, &condition.to_string(), threshold)
    }

    /// Returns a width×height u8 array representing dispute intensity per tile.
    ///
    /// Each byte encodes the second-strongest faction's influence on that tile (0 = no contest,
    /// 255 = max). Tiles with fewer than two factions above threshold are zero.
    /// Used by the renderer to visualize contested border zones.
    #[func]
    fn runtime_get_territory_dispute_texture(&self) -> PackedByteArray {
        let Some(state) = self.state.as_ref() else {
            return PackedByteArray::new();
        };
        let dispute_map = state
            .engine
            .resources()
            .territory_grid
            .export_dispute_map(sim_core::config::TERRITORY_DISPUTE_MIN_STRENGTH);
        PackedByteArray::from(dispute_map.as_slice())
    }

    /// Returns a flat PackedFloat32Array with 16 floats per alive agent for MultiMesh rendering.
    /// Layout per instance: Transform2D (8, column-major + 2 padding) + Color (4) + CustomData (4).
    /// GDScript: count = buffer.size() / 16.
    #[func]
    fn runtime_get_agent_multimesh_buffer(&self) -> PackedFloat32Array {
        let Some(state) = self.state.as_ref() else {
            return PackedFloat32Array::new();
        };
        let (buffer, _count) = build_agent_multimesh_buffer(state.engine.world());
        PackedFloat32Array::from(buffer.as_slice())
    }

    /// Returns a per-tile hardness texture (map_width × map_height bytes, FORMAT_L8).
    /// Each byte encodes the hardness of the dominant faction at that tile (0 = no faction/soft).
    /// Used by the territory shader for per-tile visual interpolation.
    #[func]
    fn runtime_get_territory_hardness_texture(&self) -> PackedByteArray {
        let Some(state) = self.state.as_ref() else {
            return PackedByteArray::new();
        };
        let (faction_map, _density_map) = state.engine.resources().territory_grid.export_dominant();
        let hardness_map = state.engine.resources().territory_hardness.clone();

        let bytes: Vec<u8> = faction_map
            .iter()
            .map(|&encoded| {
                // encoded = faction_id + 1 (0 means no faction)
                if encoded == 0 {
                    return 0u8;
                }
                let fid = encoded as u16;
                let h = hardness_map.get(&fid).copied().unwrap_or(0.0);
                (h.clamp(0.0, 1.0) * 255.0) as u8
            })
            .collect();

        PackedByteArray::from(bytes.as_slice())
    }

    /// Returns current border friction scores as a Dictionary.
    ///
    /// Keys are `"<settlement_a_id>_<settlement_b_id>"` strings (canonical min-first ordering).
    /// Values are f32 friction scores in the range 0.0..=TERRITORY_FRICTION_MAX.
    #[func]
    fn runtime_get_border_friction(&self) -> VarDictionary {
        let mut dict = VarDictionary::new();
        let Some(state) = self.state.as_ref() else {
            return dict;
        };
        for ((a, b), friction) in &state.engine.resources().border_friction {
            let key = format!("{}_{}", a.0, b.0);
            dict.set(key, *friction as f32);
        }
        dict
    }

    /// A-5 debug API: returns the Hot/Warm/Cold tier distribution across all
    /// default runtime systems. Auto-derived from each spec's tick_interval.
    /// Keys: "hot", "warm", "cold", "total".
    #[func]
    fn runtime_tier_distribution(&self) -> VarDictionary {
        let (hot, warm, cold) = crate::runtime_system::tier_distribution();
        let mut dict = VarDictionary::new();
        dict.set("hot", hot as i64);
        dict.set("warm", warm as i64);
        dict.set("cold", cold as i64);
        dict.set("total", (hot + warm + cold) as i64);
        dict
    }

    /// A-5 debug API: returns the registry names of all default runtime
    /// systems whose tier matches `tier_str` ("hot", "warm", or "cold").
    /// Returns an empty array if `tier_str` is not a valid tier name.
    #[func]
    fn runtime_systems_by_tier(&self, tier_str: GString) -> Array<GString> {
        let tier = match tier_str.to_string().as_str() {
            "hot" => sim_core::TickTier::Hot,
            "warm" => sim_core::TickTier::Warm,
            "cold" => sim_core::TickTier::Cold,
            _ => return Array::new(),
        };
        let mut arr: Array<GString> = Array::new();
        for id in crate::runtime_system::systems_by_tier(tier) {
            arr.push(&GString::from(id));
        }
        arr
    }
}

#[derive(GodotClass)]
#[class(base=Object, singleton)]
pub struct WorldSimBridge {
    base: Base<Object>,
}

#[godot_api]
impl IObject for WorldSimBridge {
    fn init(base: Base<Object>) -> Self {
        Self { base }
    }
}

#[godot_api]
impl WorldSimBridge {
    #[func]
    fn set_pathfinding_backend(&self, mode: GString) -> bool {
        let mode_string = mode.to_string();
        set_pathfind_backend_mode(&mode_string)
    }

    #[func]
    fn get_pathfinding_backend(&self) -> GString {
        get_pathfind_backend_mode().into()
    }

    #[func]
    fn resolve_pathfinding_backend(&self) -> GString {
        resolve_pathfind_backend_mode().into()
    }

    #[func]
    fn has_gpu_pathfinding(&self) -> bool {
        has_gpu_backend()
    }

    #[func]
    fn get_pathfinding_backend_stats(&self) -> VarDictionary {
        let configured_mode = get_backend_mode();
        let resolved_mode = resolve_backend_mode_code(configured_mode);
        let (cpu_dispatches, gpu_dispatches) = read_dispatch_counts();

        let mut dict = VarDictionary::new();
        dict.set("configured", backend_mode_to_str(configured_mode));
        dict.set("resolved", resolve_backend_mode(resolved_mode));
        dict.set("cpu_dispatches", cpu_dispatches as i64);
        dict.set("gpu_dispatches", gpu_dispatches as i64);
        dict.set("total_dispatches", (cpu_dispatches + gpu_dispatches) as i64);
        dict
    }

    #[func]
    fn reset_pathfinding_backend_stats(&self) {
        reset_dispatch_counts();
    }

    #[func]
    fn locale_load_fluent(&self, locale: GString, source: GString) -> bool {
        store_fluent_source(&locale.to_string(), &source.to_string())
    }

    #[func]
    fn locale_clear_fluent(&self, locale: GString) {
        clear_fluent_source(&locale.to_string());
    }

    #[func]
    fn locale_format_fluent(
        &self,
        locale: GString,
        key: GString,
        params: VarDictionary,
    ) -> GString {
        let key_string = key.to_string();
        if key_string.is_empty() {
            return GString::new();
        }
        let Some(resolved) = format_fluent_message(&locale.to_string(), &key_string, &params)
        else {
            return GString::from(key_string.as_str());
        };
        GString::from(resolved.as_str())
    }

    #[func]
    fn body_compute_age_curve(&self, axis: GString, age_years: f32) -> f32 {
        let axis_string = axis.to_string();
        body::compute_age_curve(axis_string.as_str(), age_years)
    }

    #[func]
    fn body_compute_age_curves(&self, age_years: f32) -> PackedFloat32Array {
        let curves = body::compute_age_curves(age_years);
        vec_f32_to_packed(curves.to_vec())
    }

    #[func]
    fn body_calc_training_gain(
        &self,
        potential: i32,
        trainability: i32,
        xp: f32,
        training_ceiling: f32,
        xp_for_full_progress: f32,
    ) -> i32 {
        body::calc_training_gain(
            potential,
            trainability,
            xp,
            training_ceiling,
            xp_for_full_progress,
        )
    }

    #[func]
    fn body_calc_training_gains(
        &self,
        potentials: PackedInt32Array,
        trainabilities: PackedInt32Array,
        xps: PackedFloat32Array,
        training_ceilings: PackedFloat32Array,
        xp_for_full_progress: f32,
    ) -> PackedInt32Array {
        let gains = body::calc_training_gains(
            potentials.as_slice(),
            trainabilities.as_slice(),
            xps.as_slice(),
            training_ceilings.as_slice(),
            xp_for_full_progress,
        );
        vec_i32_to_packed(gains)
    }

    #[func]
    fn body_calc_realized_values(
        &self,
        potentials: PackedInt32Array,
        trainabilities: PackedInt32Array,
        xps: PackedFloat32Array,
        training_ceilings: PackedFloat32Array,
        age_years: f32,
        xp_for_full_progress: f32,
    ) -> PackedInt32Array {
        let realized = body::calc_realized_values(
            potentials.as_slice(),
            trainabilities.as_slice(),
            xps.as_slice(),
            training_ceilings.as_slice(),
            age_years,
            xp_for_full_progress,
        );
        vec_i32_to_packed(realized)
    }

    #[func]
    fn body_age_trainability_modifier(&self, axis: GString, age_years: f32) -> f32 {
        let axis_string = axis.to_string();
        body::age_trainability_modifier(axis_string.as_str(), age_years)
    }

    #[func]
    fn body_age_trainability_modifier_rec(&self, age_years: f32) -> f32 {
        body::age_trainability_modifier("rec", age_years)
    }

    #[func]
    fn body_age_trainability_modifiers(&self, age_years: f32) -> PackedFloat32Array {
        let modifiers = body::age_trainability_modifiers(age_years);
        vec_f32_to_packed(modifiers.to_vec())
    }

    #[func]
    fn body_action_energy_cost(
        &self,
        base_cost: f32,
        end_norm: f32,
        end_cost_reduction: f32,
    ) -> f32 {
        body::action_energy_cost(base_cost, end_norm, end_cost_reduction)
    }

    #[func]
    fn body_rest_energy_recovery(
        &self,
        base_recovery: f32,
        rec_norm: f32,
        rec_recovery_bonus: f32,
    ) -> f32 {
        body::rest_energy_recovery(base_recovery, rec_norm, rec_recovery_bonus)
    }

    #[func]
    fn body_thirst_decay(&self, base_decay: f32, tile_temp: f32, temp_neutral: f32) -> f32 {
        body::thirst_decay(base_decay, tile_temp, temp_neutral)
    }

    #[func]
    fn body_warmth_decay(
        &self,
        base_decay: f32,
        tile_temp: f32,
        has_tile_temp: bool,
        temp_neutral: f32,
        temp_freezing: f32,
        temp_cold: f32,
    ) -> f32 {
        body::warmth_decay(
            base_decay,
            tile_temp,
            has_tile_temp,
            temp_neutral,
            temp_freezing,
            temp_cold,
        )
    }

    #[func]
    fn body_needs_base_decay_step(
        &self,
        scalar_inputs: PackedFloat32Array,
        flag_inputs: PackedByteArray,
    ) -> PackedFloat32Array {
        let scalars = scalar_inputs.as_slice();
        let hunger_value = *scalars.first().unwrap_or(&0.0);
        let hunger_decay_rate = *scalars.get(1).unwrap_or(&0.0);
        let hunger_stage_mult = *scalars.get(2).unwrap_or(&1.0);
        let hunger_metabolic_min = *scalars.get(3).unwrap_or(&0.0);
        let hunger_metabolic_range = *scalars.get(4).unwrap_or(&0.0);
        let energy_decay_rate = *scalars.get(5).unwrap_or(&0.0);
        let social_decay_rate = *scalars.get(6).unwrap_or(&0.0);
        let safety_decay_rate = *scalars.get(7).unwrap_or(&0.0);
        let thirst_base_decay = *scalars.get(8).unwrap_or(&0.0);
        let warmth_base_decay = *scalars.get(9).unwrap_or(&0.0);
        let tile_temp = *scalars.get(10).unwrap_or(&0.0);
        let temp_neutral = *scalars.get(11).unwrap_or(&0.5);
        let temp_freezing = *scalars.get(12).unwrap_or(&0.0);
        let temp_cold = *scalars.get(13).unwrap_or(&0.25);
        let flags = flag_inputs.as_slice();
        let has_tile_temp = flags.first().copied().unwrap_or(0) != 0;
        let needs_expansion_enabled = flags.get(1).copied().unwrap_or(0) != 0;

        let out = body::needs_base_decay_step(
            hunger_value,
            hunger_decay_rate,
            hunger_stage_mult,
            hunger_metabolic_min,
            hunger_metabolic_range,
            energy_decay_rate,
            social_decay_rate,
            safety_decay_rate,
            thirst_base_decay,
            warmth_base_decay,
            tile_temp,
            has_tile_temp,
            temp_neutral,
            temp_freezing,
            temp_cold,
            needs_expansion_enabled,
        );
        vec_f32_to_packed(out.to_vec())
    }

    #[func]
    fn body_needs_critical_severity_step_packed(
        &self,
        scalar_inputs: PackedFloat32Array,
    ) -> PackedFloat32Array {
        let scalars = scalar_inputs.as_slice();
        let thirst = *scalars.first().unwrap_or(&0.0);
        let warmth = *scalars.get(1).unwrap_or(&0.0);
        let safety = *scalars.get(2).unwrap_or(&0.0);
        let thirst_critical = *scalars.get(3).unwrap_or(&0.0);
        let warmth_critical = *scalars.get(4).unwrap_or(&0.0);
        let safety_critical = *scalars.get(5).unwrap_or(&0.0);
        let out = body::needs_critical_severity_step(
            thirst,
            warmth,
            safety,
            thirst_critical,
            warmth_critical,
            safety_critical,
        );
        vec_f32_to_packed(out.to_vec())
    }

    #[func]
    fn body_needs_critical_severity_step(
        &self,
        thirst: f32,
        warmth: f32,
        safety: f32,
        thirst_critical: f32,
        warmth_critical: f32,
        safety_critical: f32,
    ) -> PackedFloat32Array {
        let scalar_inputs = vec_f32_to_packed(vec![
            thirst,
            warmth,
            safety,
            thirst_critical,
            warmth_critical,
            safety_critical,
        ]);
        self.body_needs_critical_severity_step_packed(scalar_inputs)
    }

    #[func]
    fn body_erg_frustration_step_packed(
        &self,
        scalar_inputs: PackedFloat32Array,
        flag_inputs: PackedByteArray,
    ) -> PackedInt32Array {
        let scalars = scalar_inputs.as_slice();
        let competence = *scalars.first().unwrap_or(&0.0);
        let autonomy = *scalars.get(1).unwrap_or(&0.0);
        let self_actualization = *scalars.get(2).unwrap_or(&0.0);
        let belonging = *scalars.get(3).unwrap_or(&0.0);
        let intimacy = *scalars.get(4).unwrap_or(&0.0);
        let growth_threshold = *scalars.get(5).unwrap_or(&0.0);
        let relatedness_threshold = *scalars.get(6).unwrap_or(&0.0);
        let frustration_window = scalars.get(7).copied().unwrap_or(0.0).round() as i32;
        let growth_ticks = scalars.get(8).copied().unwrap_or(0.0).round() as i32;
        let relatedness_ticks = scalars.get(9).copied().unwrap_or(0.0).round() as i32;
        let flags = flag_inputs.as_slice();
        let was_regressing_growth = flags.first().copied().unwrap_or(0) != 0;
        let was_regressing_relatedness = flags.get(1).copied().unwrap_or(0) != 0;

        let out = body::erg_frustration_step(
            competence,
            autonomy,
            self_actualization,
            belonging,
            intimacy,
            growth_threshold,
            relatedness_threshold,
            frustration_window,
            growth_ticks,
            relatedness_ticks,
            was_regressing_growth,
            was_regressing_relatedness,
        );
        vec_i32_to_packed(out.to_vec())
    }

    #[func]
    fn body_anxious_attachment_stress_delta(
        &self,
        social: f32,
        social_threshold: f32,
        stress_rate: f32,
    ) -> f32 {
        body::anxious_attachment_stress_delta(social, social_threshold, stress_rate)
    }

    #[func]
    fn body_upper_needs_best_skill_normalized(
        &self,
        skill_levels: PackedInt32Array,
        max_level: i32,
    ) -> f32 {
        body::upper_needs_best_skill_normalized(skill_levels.as_slice(), max_level)
    }

    #[func]
    fn body_occupation_best_skill_index(&self, skill_levels: PackedInt32Array) -> i32 {
        body::occupation_best_skill_index(skill_levels.as_slice())
    }

    #[func]
    fn body_occupation_should_switch(
        &self,
        best_skill_level: i32,
        current_skill_level: i32,
        change_hysteresis: f32,
    ) -> bool {
        body::occupation_should_switch(best_skill_level, current_skill_level, change_hysteresis)
    }

    #[func]
    fn body_job_assignment_best_job_code(
        &self,
        ratios: PackedFloat32Array,
        counts: PackedInt32Array,
        alive_count: i32,
    ) -> i32 {
        body::job_assignment_best_job_code(ratios.as_slice(), counts.as_slice(), alive_count)
    }

    #[func]
    fn body_job_assignment_rebalance_codes(
        &self,
        ratios: PackedFloat32Array,
        counts: PackedInt32Array,
        alive_count: i32,
        threshold: f32,
    ) -> PackedInt32Array {
        let out = body::job_assignment_rebalance_codes(
            ratios.as_slice(),
            counts.as_slice(),
            alive_count,
            threshold,
        );
        vec_i32_to_packed(out.to_vec())
    }

    #[func]
    fn body_stat_threshold_is_active(
        &self,
        value: i32,
        threshold: i32,
        direction_code: i32,
        hysteresis: i32,
        currently_active: bool,
    ) -> bool {
        body::stat_threshold_is_active(
            value,
            threshold,
            direction_code,
            hysteresis,
            currently_active,
        )
    }

    #[func]
    fn body_stats_resource_deltas_per_100(
        &self,
        latest_food: f32,
        latest_wood: f32,
        latest_stone: f32,
        older_food: f32,
        older_wood: f32,
        older_stone: f32,
        tick_diff: f32,
    ) -> PackedFloat32Array {
        let out = body::stats_resource_deltas_per_100(
            latest_food,
            latest_wood,
            latest_stone,
            older_food,
            older_wood,
            older_stone,
            tick_diff,
        );
        vec_f32_to_packed(out.to_vec())
    }

    #[func]
    fn body_personality_linear_target(
        &self,
        age: i32,
        max_shift: f32,
        start_age: i32,
        end_age: i32,
    ) -> f32 {
        body::personality_linear_target(age, max_shift, start_age, end_age)
    }

    #[func]
    fn body_intelligence_effective_value(
        &self,
        potential: f32,
        base_mod: f32,
        age_years: f32,
        is_fluid: bool,
        activity_mod: f32,
        ace_fluid_mult: f32,
        env_penalty: f32,
        min_val: f32,
        max_val: f32,
    ) -> f32 {
        body::intelligence_effective_value(
            potential,
            base_mod,
            age_years,
            is_fluid,
            activity_mod,
            ace_fluid_mult,
            env_penalty,
            min_val,
            max_val,
        )
    }

    #[func]
    fn body_intelligence_g_value(
        &self,
        has_parents: bool,
        parent_a_g: f32,
        parent_b_g: f32,
        heritability_g: f32,
        g_mean: f32,
        openness_mean: f32,
        openness_weight: f32,
        noise: f32,
    ) -> f32 {
        body::intelligence_g_value(
            has_parents,
            parent_a_g,
            parent_b_g,
            heritability_g,
            g_mean,
            openness_mean,
            openness_weight,
            noise,
        )
    }

    #[func]
    fn body_personality_child_axis_z(
        &self,
        has_parents: bool,
        parent_a_axis: f32,
        parent_b_axis: f32,
        heritability: f32,
        random_axis_z: f32,
        is_female: bool,
        sex_diff_d: f32,
        culture_shift: f32,
    ) -> f32 {
        body::personality_child_axis_z(
            has_parents,
            parent_a_axis,
            parent_b_axis,
            heritability,
            random_axis_z,
            is_female,
            sex_diff_d,
            culture_shift,
        )
    }

    #[func]
    fn body_morale_behavior_weight_multiplier(
        &self,
        morale: f32,
        flourishing_threshold: f32,
        flourishing_min: f32,
        flourishing_max: f32,
        normal_min: f32,
        normal_max: f32,
        dissatisfied_min: f32,
        dissatisfied_max: f32,
        languishing_min: f32,
        languishing_max: f32,
    ) -> f32 {
        body::morale_behavior_weight_multiplier(
            morale,
            flourishing_threshold,
            flourishing_min,
            flourishing_max,
            normal_min,
            normal_max,
            dissatisfied_min,
            dissatisfied_max,
            languishing_min,
            languishing_max,
        )
    }

    #[func]
    fn body_morale_migration_probability(
        &self,
        morale_s: f32,
        k: f32,
        threshold_morale: f32,
        patience: f32,
        patience_resistance: f32,
        max_probability: f32,
    ) -> f32 {
        body::morale_migration_probability(
            morale_s,
            k,
            threshold_morale,
            patience,
            patience_resistance,
            max_probability,
        )
    }

    #[func]
    fn body_stat_sync_derived_scores(&self, inputs: PackedFloat32Array) -> PackedFloat32Array {
        let v = packed_f32_to_vec(&inputs);
        let out = body::stat_sync_derived_scores(&v);
        vec_f32_to_packed(out.to_vec())
    }

    #[func]
    fn body_contagion_aoe_total_susceptibility(
        &self,
        donor_count: i32,
        crowd_dilute_divisor: f32,
        refractory_active: bool,
        refractory_susceptibility: f32,
        x_axis: f32,
        e_axis: f32,
    ) -> f32 {
        body::contagion_aoe_total_susceptibility(
            donor_count,
            crowd_dilute_divisor,
            refractory_active,
            refractory_susceptibility,
            x_axis,
            e_axis,
        )
    }

    #[func]
    fn body_contagion_stress_delta(
        &self,
        stress_gap: f32,
        stress_gap_threshold: f32,
        transfer_rate: f32,
        total_susceptibility: f32,
        max_delta: f32,
    ) -> f32 {
        body::contagion_stress_delta(
            stress_gap,
            stress_gap_threshold,
            transfer_rate,
            total_susceptibility,
            max_delta,
        )
    }

    #[func]
    fn body_contagion_network_delta(
        &self,
        donor_count: i32,
        crowd_dilute_divisor: f32,
        refractory_active: bool,
        refractory_susceptibility: f32,
        network_decay: f32,
        a_axis: f32,
        valence_gap: f32,
        delta_scale: f32,
        max_abs_delta: f32,
    ) -> f32 {
        body::contagion_network_delta(
            donor_count,
            crowd_dilute_divisor,
            refractory_active,
            refractory_susceptibility,
            network_decay,
            a_axis,
            valence_gap,
            delta_scale,
            max_abs_delta,
        )
    }

    #[func]
    fn body_contagion_spiral_increment(
        &self,
        stress: f32,
        valence: f32,
        stress_threshold: f32,
        valence_threshold: f32,
        stress_divisor: f32,
        valence_divisor: f32,
        intensity_scale: f32,
        max_increment: f32,
    ) -> f32 {
        body::contagion_spiral_increment(
            stress,
            valence,
            stress_threshold,
            valence_threshold,
            stress_divisor,
            valence_divisor,
            intensity_scale,
            max_increment,
        )
    }

    #[func]
    fn body_mental_break_threshold(
        &self,
        base_break_threshold: f32,
        resilience: f32,
        c_axis: f32,
        e_axis: f32,
        allostatic: f32,
        energy_norm: f32,
        hunger_norm: f32,
        ace_break_threshold_mult: f32,
        trait_break_threshold_add: f32,
        threshold_min: f32,
        threshold_max: f32,
        reserve: f32,
        scar_threshold_reduction: f32,
    ) -> f32 {
        body::mental_break_threshold(
            base_break_threshold,
            resilience,
            c_axis,
            e_axis,
            allostatic,
            energy_norm,
            hunger_norm,
            ace_break_threshold_mult,
            trait_break_threshold_add,
            threshold_min,
            threshold_max,
            reserve,
            scar_threshold_reduction,
        )
    }

    #[func]
    fn body_mental_break_chance(
        &self,
        stress: f32,
        threshold: f32,
        reserve: f32,
        allostatic: f32,
        break_scale: f32,
        break_cap_per_tick: f32,
    ) -> f32 {
        body::mental_break_chance(
            stress,
            threshold,
            reserve,
            allostatic,
            break_scale,
            break_cap_per_tick,
        )
    }

    #[func]
    fn body_trait_violation_context_modifier(
        &self,
        is_habit: bool,
        forced_by_authority: bool,
        survival_necessity: bool,
        no_witness: bool,
        repeated_habit_modifier: f32,
        forced_modifier: f32,
        survival_modifier: f32,
        no_witness_modifier: f32,
    ) -> f32 {
        body::trait_violation_context_modifier(
            is_habit,
            forced_by_authority,
            survival_necessity,
            no_witness,
            repeated_habit_modifier,
            forced_modifier,
            survival_modifier,
            no_witness_modifier,
        )
    }

    #[func]
    fn body_trait_violation_facet_scale(&self, facet_value: f32, threshold: f32) -> f32 {
        body::trait_violation_facet_scale(facet_value, threshold)
    }

    #[func]
    fn body_trait_violation_intrusive_chance(
        &self,
        base_chance: f32,
        ptsd_mult: f32,
        ticks_since: i32,
        history_decay_ticks: i32,
        has_trauma_scars: bool,
    ) -> f32 {
        body::trait_violation_intrusive_chance(
            base_chance,
            ptsd_mult,
            ticks_since,
            history_decay_ticks,
            has_trauma_scars,
        )
    }

    #[func]
    fn body_trauma_scar_acquire_chance(
        &self,
        base_chance: f32,
        chance_scale: f32,
        existing_stacks: i32,
        kindling_factor: f32,
    ) -> f32 {
        body::trauma_scar_acquire_chance(
            base_chance,
            chance_scale,
            existing_stacks,
            kindling_factor,
        )
    }

    #[func]
    fn body_trauma_scar_sensitivity_factor(&self, base_mult: f32, stacks: i32) -> f32 {
        body::trauma_scar_sensitivity_factor(base_mult, stacks)
    }

    #[func]
    fn body_memory_decay_batch(
        &self,
        intensities: PackedFloat32Array,
        rates: PackedFloat32Array,
        dt_years: f32,
    ) -> PackedFloat32Array {
        let intensity_vec = packed_f32_to_vec(&intensities);
        let rate_vec = packed_f32_to_vec(&rates);
        let out = body::memory_decay_batch(&intensity_vec, &rate_vec, dt_years);
        vec_f32_to_packed(out)
    }

    #[func]
    fn body_memory_summary_intensity(&self, max_intensity: f32, summary_scale: f32) -> f32 {
        body::memory_summary_intensity(max_intensity, summary_scale)
    }

    #[func]
    fn body_attachment_type_code(
        &self,
        sensitivity: f32,
        consistency: f32,
        ace_score: f32,
        abuser_is_caregiver: bool,
        sensitivity_threshold_secure: f32,
        consistency_threshold_secure: f32,
        sensitivity_threshold_anxious: f32,
        consistency_threshold_disorganized: f32,
        abuser_is_caregiver_ace_min: f32,
        avoidant_sensitivity_max: f32,
        avoidant_consistency_min: f32,
    ) -> i32 {
        body::attachment_type_code(
            sensitivity,
            consistency,
            ace_score,
            abuser_is_caregiver,
            sensitivity_threshold_secure,
            consistency_threshold_secure,
            sensitivity_threshold_anxious,
            consistency_threshold_disorganized,
            abuser_is_caregiver_ace_min,
            avoidant_sensitivity_max,
            avoidant_consistency_min,
        )
    }

    #[func]
    fn body_attachment_raw_parenting_quality(
        &self,
        has_personality: bool,
        a_axis: f32,
        e_axis: f32,
        has_emotion_data: bool,
        stress: f32,
        allostatic: f32,
        has_active_break: bool,
        ace_score: f32,
    ) -> f32 {
        body::attachment_raw_parenting_quality(
            has_personality,
            a_axis,
            e_axis,
            has_emotion_data,
            stress,
            allostatic,
            has_active_break,
            ace_score,
        )
    }

    #[func]
    fn body_attachment_coping_quality_step(
        &self,
        base_quality: f32,
        dependency: f32,
        neglect_chance: f32,
        consistency_penalty: f32,
    ) -> PackedFloat32Array {
        let out = body::attachment_coping_quality_step(
            base_quality,
            dependency,
            neglect_chance,
            consistency_penalty,
        );
        vec_f32_to_packed(out.to_vec())
    }

    #[func]
    fn body_attachment_protective_factor(
        &self,
        is_secure: bool,
        eh: f32,
        secure_weight: f32,
        eh_weight: f32,
        max_pf: f32,
    ) -> f32 {
        body::attachment_protective_factor(is_secure, eh, secure_weight, eh_weight, max_pf)
    }

    #[func]
    fn body_intergen_scar_index(&self, scar_count: i32, norm_divisor: f32) -> f32 {
        body::intergen_scar_index(scar_count, norm_divisor)
    }

    #[func]
    fn body_intergen_child_epigenetic_step(
        &self,
        inputs: PackedFloat32Array,
    ) -> PackedFloat32Array {
        let v = packed_f32_to_vec(&inputs);
        let out = body::intergen_child_epigenetic_step(&v);
        vec_f32_to_packed(out.to_vec())
    }

    #[func]
    fn body_intergen_hpa_sensitivity(&self, epigenetic_load: f32, hpa_load_weight: f32) -> f32 {
        body::intergen_hpa_sensitivity(epigenetic_load, hpa_load_weight)
    }

    #[func]
    fn body_intergen_meaney_repair_load(
        &self,
        current_load: f32,
        parenting_quality: f32,
        threshold: f32,
        repair_rate: f32,
        min_load: f32,
    ) -> f32 {
        body::intergen_meaney_repair_load(
            current_load,
            parenting_quality,
            threshold,
            repair_rate,
            min_load,
        )
    }

    #[func]
    fn body_parenting_hpa_adjusted_stress_gain(
        &self,
        current_stress_mult: f32,
        epigenetic_load: f32,
        hpa_load_weight: f32,
    ) -> f32 {
        body::parenting_hpa_adjusted_stress_gain(
            current_stress_mult,
            epigenetic_load,
            hpa_load_weight,
        )
    }

    #[func]
    fn body_parenting_bandura_base_rate(
        &self,
        base_coeff: f32,
        coping_mult: f32,
        observation_strength: f32,
        is_maladaptive: bool,
        maladaptive_multiplier: f32,
    ) -> f32 {
        body::parenting_bandura_base_rate(
            base_coeff,
            coping_mult,
            observation_strength,
            is_maladaptive,
            maladaptive_multiplier,
        )
    }

    #[func]
    fn body_ace_partial_score_next(
        &self,
        current_partial: f32,
        severity: f32,
        ace_weight: f32,
    ) -> f32 {
        body::ace_partial_score_next(current_partial, severity, ace_weight)
    }

    #[func]
    fn body_ace_score_total_from_partials(&self, partials: PackedFloat32Array) -> f32 {
        body::ace_score_total_from_partials(&packed_f32_to_vec(&partials))
    }

    #[func]
    fn body_ace_threat_deprivation_totals(
        &self,
        partials: PackedFloat32Array,
        type_codes: PackedInt32Array,
    ) -> PackedFloat32Array {
        let partial_vec = packed_f32_to_vec(&partials);
        let code_vec = packed_i32_to_vec(&type_codes);
        let out = body::ace_threat_deprivation_totals(&partial_vec, &code_vec);
        vec_f32_to_packed(out.to_vec())
    }

    #[func]
    fn body_ace_adult_modifiers_adjusted(
        &self,
        base_stress_gain_mult: f32,
        base_break_threshold_mult: f32,
        base_allostatic_base: f32,
        break_floor: f32,
        protective_factor: f32,
    ) -> PackedFloat32Array {
        let out = body::ace_adult_modifiers_adjusted(
            base_stress_gain_mult,
            base_break_threshold_mult,
            base_allostatic_base,
            break_floor,
            protective_factor,
        );
        vec_f32_to_packed(out.to_vec())
    }

    #[func]
    fn body_ace_backfill_score(
        &self,
        allostatic: f32,
        trauma_count: i32,
        attachment_code: i32,
    ) -> f32 {
        body::ace_backfill_score(allostatic, trauma_count, attachment_code)
    }

    #[func]
    fn body_leader_age_respect(&self, age_years: f32) -> f32 {
        body::leader_age_respect(age_years)
    }

    #[func]
    fn body_leader_score(
        &self,
        charisma: f32,
        wisdom: f32,
        trustworthiness: f32,
        intimidation: f32,
        social_capital: f32,
        age_respect: f32,
        w_charisma: f32,
        w_wisdom: f32,
        w_trustworthiness: f32,
        w_intimidation: f32,
        w_social_capital: f32,
        w_age_respect: f32,
        rep_overall: f32,
    ) -> f32 {
        body::leader_score(
            charisma,
            wisdom,
            trustworthiness,
            intimidation,
            social_capital,
            age_respect,
            w_charisma,
            w_wisdom,
            w_trustworthiness,
            w_intimidation,
            w_social_capital,
            w_age_respect,
            rep_overall,
        )
    }

    #[func]
    fn body_network_social_capital_norm(
        &self,
        strong_count: f32,
        weak_count: f32,
        bridge_count: f32,
        rep_score: f32,
        strong_weight: f32,
        weak_weight: f32,
        bridge_weight: f32,
        rep_weight: f32,
        norm_div: f32,
    ) -> f32 {
        body::network_social_capital_norm(
            strong_count,
            weak_count,
            bridge_count,
            rep_score,
            strong_weight,
            weak_weight,
            bridge_weight,
            rep_weight,
            norm_div,
        )
    }

    #[func]
    fn body_revolution_risk_score(
        &self,
        unhappiness: f32,
        frustration: f32,
        inequality: f32,
        leader_unpopularity: f32,
        independence_ratio: f32,
    ) -> f32 {
        body::revolution_risk_score(
            unhappiness,
            frustration,
            inequality,
            leader_unpopularity,
            independence_ratio,
        )
    }

    #[func]
    fn body_reputation_event_delta(
        &self,
        valence: f32,
        magnitude: f32,
        delta_scale: f32,
        neg_bias: f32,
    ) -> f32 {
        body::reputation_event_delta(valence, magnitude, delta_scale, neg_bias)
    }

    #[func]
    fn body_reputation_decay_value(&self, value: f32, pos_decay: f32, neg_decay: f32) -> f32 {
        body::reputation_decay_value(value, pos_decay, neg_decay)
    }

    #[func]
    fn body_economic_tendencies_step(
        &self,
        scalar_inputs: PackedFloat32Array,
        is_male: bool,
        wealth_generosity_penalty: f32,
    ) -> PackedFloat32Array {
        let scalars = scalar_inputs.as_slice();
        let out = body::economic_tendencies_step(
            *scalars.first().unwrap_or(&0.5),
            *scalars.get(1).unwrap_or(&0.5),
            *scalars.get(2).unwrap_or(&0.5),
            *scalars.get(3).unwrap_or(&0.5),
            *scalars.get(4).unwrap_or(&0.5),
            *scalars.get(5).unwrap_or(&0.5),
            *scalars.get(6).unwrap_or(&0.0),
            *scalars.get(7).unwrap_or(&0.0),
            *scalars.get(8).unwrap_or(&0.0),
            *scalars.get(9).unwrap_or(&0.0),
            *scalars.get(10).unwrap_or(&0.0),
            *scalars.get(11).unwrap_or(&0.0),
            *scalars.get(12).unwrap_or(&0.0),
            *scalars.get(13).unwrap_or(&0.0),
            *scalars.get(14).unwrap_or(&0.0),
            *scalars.get(15).unwrap_or(&0.0),
            *scalars.get(16).unwrap_or(&0.0),
            *scalars.get(17).unwrap_or(&0.0),
            *scalars.get(18).unwrap_or(&0.0),
            *scalars.get(19).unwrap_or(&0.0),
            *scalars.get(20).unwrap_or(&0.0),
            is_male,
            wealth_generosity_penalty,
        );
        vec_f32_to_packed(out.to_vec())
    }

    #[func]
    fn body_stratification_gini(&self, values: PackedFloat32Array) -> f32 {
        body::stratification_gini(values.as_slice())
    }

    #[func]
    fn body_stratification_status_score(&self, scalar_inputs: PackedFloat32Array) -> f32 {
        let scalars = scalar_inputs.as_slice();
        body::stratification_status_score(
            *scalars.first().unwrap_or(&0.0),
            *scalars.get(1).unwrap_or(&0.0),
            *scalars.get(2).unwrap_or(&0.0),
            *scalars.get(3).unwrap_or(&0.0),
            *scalars.get(4).unwrap_or(&0.0),
            *scalars.get(5).unwrap_or(&0.0),
            *scalars.get(6).unwrap_or(&0.0),
            *scalars.get(7).unwrap_or(&0.0),
            *scalars.get(8).unwrap_or(&0.0),
            *scalars.get(9).unwrap_or(&0.0),
        )
    }

    #[func]
    fn body_stratification_wealth_score(
        &self,
        food_days: f32,
        wood_norm: f32,
        stone_norm: f32,
        w_food: f32,
        w_wood: f32,
        w_stone: f32,
    ) -> f32 {
        body::stratification_wealth_score(food_days, wood_norm, stone_norm, w_food, w_wood, w_stone)
    }

    #[func]
    fn body_value_plasticity(&self, age_years: f32) -> f32 {
        body::value_plasticity(age_years)
    }

    #[func]
    fn body_family_newborn_health(
        &self,
        gestation_weeks: i32,
        mother_nutrition: f32,
        mother_age: f32,
        genetics_z: f32,
        tech: f32,
    ) -> f32 {
        body::family_newborn_health(
            gestation_weeks,
            mother_nutrition,
            mother_age,
            genetics_z,
            tech,
        )
    }

    #[func]
    fn body_title_is_elder(&self, age_years: f32, elder_min_age_years: f32) -> bool {
        body::title_is_elder(age_years, elder_min_age_years)
    }

    #[func]
    fn body_title_skill_tier(&self, level: i32, expert_level: i32, master_level: i32) -> i32 {
        body::title_skill_tier(level, expert_level, master_level)
    }

    #[func]
    fn body_social_attachment_affinity_multiplier(&self, a_mult: f32, b_mult: f32) -> f32 {
        body::social_attachment_affinity_multiplier(a_mult, b_mult)
    }

    #[func]
    fn body_social_proposal_accept_prob(&self, romantic_interest: f32, compatibility: f32) -> f32 {
        body::social_proposal_accept_prob(romantic_interest, compatibility)
    }

    #[func]
    fn body_tension_scarcity_pressure(
        &self,
        s1_deficit: bool,
        s2_deficit: bool,
        per_shared_resource: f32,
    ) -> f32 {
        body::tension_scarcity_pressure(s1_deficit, s2_deficit, per_shared_resource)
    }

    #[func]
    fn body_tension_next_value(
        &self,
        current: f32,
        scarcity_pressure: f32,
        decay_per_year: f32,
        dt_years: f32,
    ) -> f32 {
        body::tension_next_value(current, scarcity_pressure, decay_per_year, dt_years)
    }

    #[func]
    fn body_resource_regen_next(&self, current: f32, cap: f32, rate: f32) -> f32 {
        body::resource_regen_next(current, cap, rate)
    }

    #[func]
    fn body_age_body_speed(&self, agi_realized: i32, speed_scale: f32, speed_base: f32) -> f32 {
        body::age_body_speed(agi_realized, speed_scale, speed_base)
    }

    #[func]
    fn body_age_body_strength(&self, str_realized: i32) -> f32 {
        body::age_body_strength(str_realized)
    }

    #[func]
    #[allow(clippy::too_many_arguments)]
    fn body_tech_discovery_prob(
        &self,
        base: f32,
        pop_bonus: f32,
        knowledge_bonus: f32,
        openness_bonus: f32,
        logical_bonus: f32,
        naturalistic_bonus: f32,
        soft_bonus: f32,
        rediscovery_bonus: f32,
        max_bonus: f32,
        checks_per_year: f32,
    ) -> f32 {
        body::tech_discovery_prob(
            base,
            pop_bonus,
            knowledge_bonus,
            openness_bonus,
            logical_bonus,
            naturalistic_bonus,
            soft_bonus,
            rediscovery_bonus,
            max_bonus,
            checks_per_year,
        )
    }

    #[func]
    fn body_migration_food_scarce(
        &self,
        nearby_food: f32,
        population: i32,
        per_capita_threshold: f32,
    ) -> bool {
        body::migration_food_scarce(nearby_food, population, per_capita_threshold)
    }

    #[func]
    fn body_migration_should_attempt(
        &self,
        overcrowded: bool,
        food_scarce: bool,
        chance_roll: f32,
        migration_chance: f32,
    ) -> bool {
        body::migration_should_attempt(overcrowded, food_scarce, chance_roll, migration_chance)
    }

    #[func]
    fn body_population_housing_cap(
        &self,
        total_shelters: i32,
        free_population_cap: i32,
        shelter_capacity_per_building: i32,
    ) -> i32 {
        body::population_housing_cap(
            total_shelters,
            free_population_cap,
            shelter_capacity_per_building,
        )
    }

    #[func]
    #[allow(clippy::too_many_arguments)]
    fn body_population_birth_block_code(
        &self,
        alive_count: i32,
        max_entities: i32,
        total_shelters: i32,
        total_food: f32,
        min_population: i32,
        free_population_cap: i32,
        shelter_capacity_per_building: i32,
        food_per_alive: f32,
    ) -> i32 {
        body::population_birth_block_code(
            alive_count,
            max_entities,
            total_shelters,
            total_food,
            min_population,
            free_population_cap,
            shelter_capacity_per_building,
            food_per_alive,
        )
    }

    #[func]
    fn body_chronicle_should_prune(
        &self,
        current_year: i32,
        last_prune_year: i32,
        prune_interval_years: i32,
    ) -> bool {
        body::chronicle_should_prune(current_year, last_prune_year, prune_interval_years)
    }

    #[func]
    fn body_chronicle_cutoff_tick(
        &self,
        current_year: i32,
        max_age_years: i32,
        ticks_per_year: i32,
    ) -> i32 {
        body::chronicle_cutoff_tick(current_year, max_age_years, ticks_per_year)
    }

    #[func]
    fn body_chronicle_keep_world_event(
        &self,
        event_tick: i32,
        importance: i32,
        low_cutoff_tick: i32,
        med_cutoff_tick: i32,
    ) -> bool {
        body::chronicle_keep_world_event(event_tick, importance, low_cutoff_tick, med_cutoff_tick)
    }

    #[func]
    fn body_chronicle_keep_personal_event(
        &self,
        has_valid_world_tick: bool,
        importance: i32,
    ) -> bool {
        body::chronicle_keep_personal_event(has_valid_world_tick, importance)
    }

    #[func]
    fn body_psychology_break_type_code(&self, break_type: GString) -> i32 {
        body::psychology_break_type_code(&break_type.to_string())
    }

    #[func]
    fn body_psychology_break_type_label(&self, code: i32) -> GString {
        GString::from(body::psychology_break_type_label(code))
    }

    #[func]
    fn body_coping_learn_probability(
        &self,
        stress: f32,
        allostatic: f32,
        is_recovery: bool,
        break_count: i32,
        owned_count: i32,
        coping_count_max: f32,
    ) -> f32 {
        body::coping_learn_probability(
            stress,
            allostatic,
            is_recovery,
            break_count,
            owned_count,
            coping_count_max,
        )
    }

    #[func]
    fn body_coping_softmax_index(&self, scores: PackedFloat32Array, roll01: f32) -> i32 {
        body::coping_softmax_index(&packed_f32_to_vec(&scores), roll01)
    }

    #[func]
    fn body_emotion_break_threshold(&self, z_c: f32, base_threshold: f32, z_scale: f32) -> f32 {
        body::emotion_break_threshold(z_c, base_threshold, z_scale)
    }

    #[func]
    fn body_emotion_break_trigger_probability(
        &self,
        stress: f32,
        threshold: f32,
        beta: f32,
        tick_prob: f32,
    ) -> f32 {
        body::emotion_break_trigger_probability(stress, threshold, beta, tick_prob)
    }

    #[func]
    fn body_emotion_break_type_code(
        &self,
        outrage: f32,
        fear: f32,
        anger: f32,
        sadness: f32,
        disgust: f32,
        outrage_threshold: f32,
    ) -> i32 {
        body::emotion_break_type_code(outrage, fear, anger, sadness, disgust, outrage_threshold)
    }

    #[func]
    fn body_emotion_adjusted_half_life(&self, base_half_life: f32, coeff: f32, z: f32) -> f32 {
        body::emotion_adjusted_half_life(base_half_life, coeff, z)
    }

    #[func]
    fn body_emotion_baseline_value(
        &self,
        base_value: f32,
        scale_value: f32,
        z: f32,
        min_value: f32,
        max_value: f32,
    ) -> f32 {
        body::emotion_baseline_value(base_value, scale_value, z, min_value, max_value)
    }

    #[func]
    fn body_emotion_habituation_factor(&self, eta: f32, repeat_count: i32) -> f32 {
        body::emotion_habituation_factor(eta, repeat_count)
    }

    #[func]
    fn body_emotion_contagion_susceptibility(&self, z_e: f32, z_a: f32) -> f32 {
        body::emotion_contagion_susceptibility(z_e, z_a)
    }

    #[func]
    fn body_emotion_contagion_distance_factor(&self, distance: f32, distance_scale: f32) -> f32 {
        body::emotion_contagion_distance_factor(distance, distance_scale)
    }

    #[func]
    fn body_emotion_event_impulse_from_appraisal(
        &self,
        inputs: PackedFloat32Array,
    ) -> PackedFloat32Array {
        let out = body::emotion_event_impulse_from_appraisal(&packed_f32_to_vec(&inputs));
        vec_f32_to_packed(out.to_vec())
    }

    #[func]
    fn body_emotion_event_impulse_batch(
        &self,
        flat_inputs: PackedFloat32Array,
    ) -> PackedFloat32Array {
        let out = body::emotion_event_impulse_batch(&packed_f32_to_vec(&flat_inputs));
        vec_f32_to_packed(out)
    }

    #[func]
    fn body_tech_cultural_memory_decay(
        &self,
        current_memory: f32,
        base_decay: f32,
        forgotten_long_multiplier: f32,
        memory_floor: f32,
        forgotten_recent: bool,
    ) -> f32 {
        body::tech_cultural_memory_decay(
            current_memory,
            base_decay,
            forgotten_long_multiplier,
            memory_floor,
            forgotten_recent,
        )
    }

    #[func]
    fn body_tech_modifier_stack_clamp(
        &self,
        multiplier_product: f32,
        additive_sum: f32,
        multiplier_cap: f32,
        additive_cap: f32,
    ) -> PackedFloat32Array {
        let out = body::tech_modifier_stack_clamp(
            multiplier_product,
            additive_sum,
            multiplier_cap,
            additive_cap,
        );
        vec_f32_to_packed(out.to_vec())
    }

    #[func]
    fn body_movement_should_skip_tick(&self, skip_mod: i32, tick: i32, entity_id: i32) -> bool {
        body::movement_should_skip_tick(skip_mod, tick, entity_id)
    }

    #[func]
    fn body_building_campfire_social_boost(
        &self,
        is_night: bool,
        day_boost: f32,
        night_boost: f32,
    ) -> f32 {
        body::building_campfire_social_boost(is_night, day_boost, night_boost)
    }

    #[func]
    fn body_building_add_capped(&self, current: f32, delta: f32, cap: f32) -> f32 {
        body::building_add_capped(current, delta, cap)
    }

    #[func]
    fn body_childcare_take_food(&self, available: f32, remaining: f32) -> f32 {
        body::childcare_take_food(available, remaining)
    }

    #[func]
    fn body_childcare_hunger_after(
        &self,
        current_hunger: f32,
        withdrawn: f32,
        food_hunger_restore: f32,
    ) -> f32 {
        body::childcare_hunger_after(current_hunger, withdrawn, food_hunger_restore)
    }

    #[func]
    fn body_tech_propagation_culture_modifier(
        &self,
        knowledge_avg: f32,
        tradition_avg: f32,
        knowledge_weight: f32,
        tradition_weight: f32,
        min_mod: f32,
        max_mod: f32,
    ) -> f32 {
        body::tech_propagation_culture_modifier(
            knowledge_avg,
            tradition_avg,
            knowledge_weight,
            tradition_weight,
            min_mod,
            max_mod,
        )
    }

    #[func]
    fn body_tech_propagation_carrier_bonus(
        &self,
        max_skill: i32,
        skill_divisor: f32,
        weight: f32,
    ) -> f32 {
        body::tech_propagation_carrier_bonus(max_skill, skill_divisor, weight)
    }

    #[func]
    fn body_tech_propagation_final_prob(
        &self,
        base_prob: f32,
        lang_penalty: f32,
        culture_mod: f32,
        carrier_bonus: f32,
        stability_bonus: f32,
        max_prob: f32,
    ) -> f32 {
        body::tech_propagation_final_prob(
            base_prob,
            lang_penalty,
            culture_mod,
            carrier_bonus,
            stability_bonus,
            max_prob,
        )
    }

    #[func]
    fn body_mortality_hazards_and_prob(
        &self,
        model_inputs: PackedFloat32Array,
        env_inputs: PackedFloat32Array,
        is_monthly: bool,
    ) -> PackedFloat32Array {
        let m = packed_f32_to_vec(&model_inputs);
        let e = packed_f32_to_vec(&env_inputs);
        if m.len() < 10 || e.len() < 8 {
            return PackedFloat32Array::new();
        }
        let out = body::mortality_hazards_and_prob(
            m[0], m[1], m[2], m[3], m[4], m[5], m[6], m[7], m[8], m[9], e[0], e[1], e[2], e[3],
            e[4], e[5], e[6], e[7], is_monthly,
        );
        vec_f32_to_packed(out.to_vec())
    }

    #[func]
    fn body_cognition_activity_modifier(
        &self,
        active_skill_count: i32,
        activity_buffer: f32,
        inactivity_accel: f32,
    ) -> f32 {
        body::cognition_activity_modifier(active_skill_count, activity_buffer, inactivity_accel)
    }

    #[func]
    fn body_cognition_ace_fluid_decline_mult(
        &self,
        ace_penalty: f32,
        ace_penalty_minor: f32,
        ace_fluid_decline_mult: f32,
    ) -> f32 {
        body::cognition_ace_fluid_decline_mult(
            ace_penalty,
            ace_penalty_minor,
            ace_fluid_decline_mult,
        )
    }

    #[func]
    fn body_upper_needs_job_alignment(
        &self,
        job_code: i32,
        craftsmanship: f32,
        skill: f32,
        hard_work: f32,
        nature: f32,
        independence: f32,
    ) -> f32 {
        body::upper_needs_job_alignment(
            job_code,
            craftsmanship,
            skill,
            hard_work,
            nature,
            independence,
        )
    }

    #[func]
    fn body_job_satisfaction_score(
        &self,
        personality_actual: PackedFloat32Array,
        personality_ideal: PackedFloat32Array,
        value_actual: PackedFloat32Array,
        value_weights: PackedFloat32Array,
        skill_fit: f32,
        autonomy: f32,
        competence: f32,
        meaning: f32,
        autonomy_level: f32,
        prestige: f32,
        w_skill_fit: f32,
        w_value_fit: f32,
        w_personality_fit: f32,
        w_need_fit: f32,
    ) -> f32 {
        body::job_satisfaction_score(
            personality_actual.as_slice(),
            personality_ideal.as_slice(),
            value_actual.as_slice(),
            value_weights.as_slice(),
            skill_fit,
            autonomy,
            competence,
            meaning,
            autonomy_level,
            prestige,
            w_skill_fit,
            w_value_fit,
            w_personality_fit,
            w_need_fit,
        )
    }

    #[func]
    fn body_job_satisfaction_score_batch(
        &self,
        personality_actual: PackedFloat32Array,
        personality_ideals_flat: PackedFloat32Array,
        value_actual: PackedFloat32Array,
        value_weights_flat: PackedFloat32Array,
        skill_fits: PackedFloat32Array,
        autonomy: f32,
        competence: f32,
        meaning: f32,
        autonomy_levels: PackedFloat32Array,
        prestiges: PackedFloat32Array,
        w_skill_fit: f32,
        w_value_fit: f32,
        w_personality_fit: f32,
        w_need_fit: f32,
    ) -> PackedFloat32Array {
        let out = body::job_satisfaction_score_batch(
            personality_actual.as_slice(),
            personality_ideals_flat.as_slice(),
            value_actual.as_slice(),
            value_weights_flat.as_slice(),
            skill_fits.as_slice(),
            autonomy,
            competence,
            meaning,
            autonomy_levels.as_slice(),
            prestiges.as_slice(),
            w_skill_fit,
            w_value_fit,
            w_personality_fit,
            w_need_fit,
        );
        vec_f32_to_packed(out)
    }

    #[func]
    fn body_upper_needs_step_packed(
        &self,
        scalar_inputs: PackedFloat32Array,
        flag_inputs: PackedByteArray,
    ) -> PackedFloat32Array {
        let scalars = scalar_inputs.as_slice();
        let current_values = [
            *scalars.first().unwrap_or(&0.0),
            *scalars.get(1).unwrap_or(&0.0),
            *scalars.get(2).unwrap_or(&0.0),
            *scalars.get(3).unwrap_or(&0.0),
            *scalars.get(4).unwrap_or(&0.0),
            *scalars.get(5).unwrap_or(&0.0),
            *scalars.get(6).unwrap_or(&0.0),
            *scalars.get(7).unwrap_or(&0.0),
        ];
        let decay_values = [
            *scalars.get(8).unwrap_or(&0.0),
            *scalars.get(9).unwrap_or(&0.0),
            *scalars.get(10).unwrap_or(&0.0),
            *scalars.get(11).unwrap_or(&0.0),
            *scalars.get(12).unwrap_or(&0.0),
            *scalars.get(13).unwrap_or(&0.0),
            *scalars.get(14).unwrap_or(&0.0),
            *scalars.get(15).unwrap_or(&0.0),
        ];
        let competence_job_gain = *scalars.get(16).unwrap_or(&0.0);
        let autonomy_job_gain = *scalars.get(17).unwrap_or(&0.0);
        let belonging_settlement_gain = *scalars.get(18).unwrap_or(&0.0);
        let intimacy_partner_gain = *scalars.get(19).unwrap_or(&0.0);
        let recognition_skill_coeff = *scalars.get(20).unwrap_or(&0.0);
        let self_act_skill_coeff = *scalars.get(21).unwrap_or(&0.0);
        let meaning_base_gain = *scalars.get(22).unwrap_or(&0.0);
        let meaning_aligned_gain = *scalars.get(23).unwrap_or(&0.0);
        let transcendence_settlement_gain = *scalars.get(24).unwrap_or(&0.0);
        let transcendence_sacrifice_coeff = *scalars.get(25).unwrap_or(&0.0);
        let best_skill_norm = *scalars.get(26).unwrap_or(&0.0);
        let alignment = *scalars.get(27).unwrap_or(&0.0);
        let sacrifice_value = *scalars.get(28).unwrap_or(&0.0);
        let flags = flag_inputs.as_slice();
        let has_job = flags.first().copied().unwrap_or(0) != 0;
        let has_settlement = flags.get(1).copied().unwrap_or(0) != 0;
        let has_partner = flags.get(2).copied().unwrap_or(0) != 0;

        let out = body::upper_needs_step(
            &current_values,
            &decay_values,
            competence_job_gain,
            autonomy_job_gain,
            belonging_settlement_gain,
            intimacy_partner_gain,
            recognition_skill_coeff,
            self_act_skill_coeff,
            meaning_base_gain,
            meaning_aligned_gain,
            transcendence_settlement_gain,
            transcendence_sacrifice_coeff,
            best_skill_norm,
            alignment,
            sacrifice_value,
            has_job,
            has_settlement,
            has_partner,
        );
        vec_f32_to_packed(out.to_vec())
    }

    #[func]
    fn body_child_parent_stress_transfer(
        &self,
        parent_stress: f32,
        parent_dependency: f32,
        attachment_code: i32,
        caregiver_support_active: bool,
        buffer_power: f32,
        contagion_input: f32,
    ) -> f32 {
        body::child_parent_stress_transfer(
            parent_stress,
            parent_dependency,
            attachment_code,
            caregiver_support_active,
            buffer_power,
            contagion_input,
        )
    }

    #[func]
    fn body_child_simultaneous_ace_step(
        &self,
        prev_residual: f32,
        severities: PackedFloat32Array,
    ) -> PackedFloat32Array {
        let out = body::child_simultaneous_ace_step(severities.as_slice(), prev_residual);
        vec_f32_to_packed(out.to_vec())
    }

    #[func]
    fn body_child_social_buffered_intensity(
        &self,
        intensity: f32,
        attachment_quality: f32,
        caregiver_present: bool,
        buffer_power: f32,
    ) -> f32 {
        body::child_social_buffered_intensity(
            intensity,
            attachment_quality,
            caregiver_present,
            buffer_power,
        )
    }

    #[func]
    fn body_child_shrp_step(
        &self,
        intensity: f32,
        shrp_active: bool,
        shrp_override_threshold: f32,
        vulnerability_mult: f32,
    ) -> PackedFloat32Array {
        let out = body::child_shrp_step(
            intensity,
            shrp_active,
            shrp_override_threshold,
            vulnerability_mult,
        );
        vec_f32_to_packed(out.to_vec())
    }

    #[func]
    fn body_child_stress_type_code(
        &self,
        intensity: f32,
        attachment_present: bool,
        attachment_quality: f32,
    ) -> i32 {
        body::child_stress_type_code(intensity, attachment_present, attachment_quality)
    }

    #[func]
    fn body_child_stress_apply_step(
        &self,
        resilience: f32,
        reserve: f32,
        stress: f32,
        allostatic: f32,
        intensity: f32,
        spike_mult: f32,
        vulnerability_mult: f32,
        break_threshold_mult: f32,
        stress_type_code: i32,
    ) -> PackedFloat32Array {
        let out = body::child_stress_apply_step(
            resilience,
            reserve,
            stress,
            allostatic,
            intensity,
            spike_mult,
            vulnerability_mult,
            break_threshold_mult,
            stress_type_code,
        );
        vec_f32_to_packed(out.to_vec())
    }

    #[func]
    fn body_child_parent_transfer_apply_step(
        &self,
        current_stress: f32,
        transferred: f32,
        transfer_threshold: f32,
        transfer_scale: f32,
        stress_clamp_max: f32,
    ) -> f32 {
        body::child_parent_transfer_apply_step(
            current_stress,
            transferred,
            transfer_threshold,
            transfer_scale,
            stress_clamp_max,
        )
    }

    #[func]
    fn body_child_deprivation_damage_step(&self, current_damage: f32, damage_rate: f32) -> f32 {
        body::child_deprivation_damage_step(current_damage, damage_rate)
    }

    #[func]
    fn body_child_stage_code_from_age_ticks(
        &self,
        age_ticks: i32,
        infant_max_years: f32,
        toddler_max_years: f32,
        child_max_years: f32,
        teen_max_years: f32,
    ) -> i32 {
        body::child_stage_code_from_age_ticks(
            age_ticks,
            infant_max_years,
            toddler_max_years,
            child_max_years,
            teen_max_years,
        )
    }

    #[func]
    fn body_stress_rebound_apply_step(
        &self,
        stress: f32,
        hidden_threat_accumulator: f32,
        total_rebound: f32,
        stress_clamp_max: f32,
    ) -> PackedFloat32Array {
        let out = body::stress_rebound_apply_step(
            stress,
            hidden_threat_accumulator,
            total_rebound,
            stress_clamp_max,
        );
        vec_f32_to_packed(out.to_vec())
    }

    #[func]
    fn body_stress_injection_apply_step(
        &self,
        stress: f32,
        final_instant: f32,
        final_per_tick: f32,
        trace_threshold: f32,
        stress_clamp_max: f32,
    ) -> PackedFloat32Array {
        let out = body::stress_injection_apply_step(
            stress,
            final_instant,
            final_per_tick,
            trace_threshold,
            stress_clamp_max,
        );
        vec_f32_to_packed(out.to_vec())
    }

    #[func]
    fn body_stress_shaken_countdown_step(&self, shaken_remaining: i32) -> PackedFloat32Array {
        let out = body::stress_shaken_countdown_step(shaken_remaining);
        vec_f32_to_packed(out.to_vec())
    }

    #[func]
    fn body_stress_support_score(&self, strengths: PackedFloat32Array) -> f32 {
        body::stress_support_score(strengths.as_slice())
    }

    #[func]
    fn pathfind_grid(
        &self,
        width: i32,
        height: i32,
        walkable: PackedByteArray,
        move_cost: PackedFloat32Array,
        from_x: i32,
        from_y: i32,
        to_x: i32,
        to_y: i32,
        max_steps: i32,
    ) -> PackedVector2Array {
        let steps = normalize_max_steps(max_steps);
        let backend_mode = get_backend_mode();

        let path = match dispatch_pathfind_grid_bytes(
            backend_mode,
            width,
            height,
            walkable.as_slice(),
            move_cost.as_slice(),
            from_x,
            from_y,
            to_x,
            to_y,
            steps,
        ) {
            Ok(path) => path,
            Err(_) => return PackedVector2Array::new(),
        };

        encode_path_vec2(path)
    }

    #[func]
    fn pathfind_grid_xy(
        &self,
        width: i32,
        height: i32,
        walkable: PackedByteArray,
        move_cost: PackedFloat32Array,
        from_x: i32,
        from_y: i32,
        to_x: i32,
        to_y: i32,
        max_steps: i32,
    ) -> PackedInt32Array {
        let steps = normalize_max_steps(max_steps);
        let backend_mode = get_backend_mode();

        let path = match dispatch_pathfind_grid_bytes(
            backend_mode,
            width,
            height,
            walkable.as_slice(),
            move_cost.as_slice(),
            from_x,
            from_y,
            to_x,
            to_y,
            steps,
        ) {
            Ok(path) => path,
            Err(_) => return PackedInt32Array::new(),
        };

        encode_path_xy(path)
    }

    #[func]
    fn pathfind_grid_gpu(
        &self,
        width: i32,
        height: i32,
        walkable: PackedByteArray,
        move_cost: PackedFloat32Array,
        from_x: i32,
        from_y: i32,
        to_x: i32,
        to_y: i32,
        max_steps: i32,
    ) -> PackedVector2Array {
        let steps = normalize_max_steps(max_steps);

        let path = match dispatch_pathfind_grid_bytes(
            PATHFIND_BACKEND_GPU,
            width,
            height,
            walkable.as_slice(),
            move_cost.as_slice(),
            from_x,
            from_y,
            to_x,
            to_y,
            steps,
        ) {
            Ok(path) => path,
            Err(_) => return PackedVector2Array::new(),
        };

        encode_path_vec2(path)
    }

    #[func]
    fn pathfind_grid_gpu_xy(
        &self,
        width: i32,
        height: i32,
        walkable: PackedByteArray,
        move_cost: PackedFloat32Array,
        from_x: i32,
        from_y: i32,
        to_x: i32,
        to_y: i32,
        max_steps: i32,
    ) -> PackedInt32Array {
        let steps = normalize_max_steps(max_steps);

        let path = match dispatch_pathfind_grid_bytes(
            PATHFIND_BACKEND_GPU,
            width,
            height,
            walkable.as_slice(),
            move_cost.as_slice(),
            from_x,
            from_y,
            to_x,
            to_y,
            steps,
        ) {
            Ok(path) => path,
            Err(_) => return PackedInt32Array::new(),
        };

        encode_path_xy(path)
    }

    #[func]
    fn pathfind_grid_batch(
        &self,
        width: i32,
        height: i32,
        walkable: PackedByteArray,
        move_cost: PackedFloat32Array,
        from_points: PackedVector2Array,
        to_points: PackedVector2Array,
        max_steps: i32,
    ) -> Array<PackedVector2Array> {
        let steps = normalize_max_steps(max_steps);
        let backend_mode = get_backend_mode();

        let path_groups = match dispatch_pathfind_grid_batch_vec2_bytes(
            backend_mode,
            width,
            height,
            walkable.as_slice(),
            move_cost.as_slice(),
            from_points.as_slice(),
            to_points.as_slice(),
            steps,
        ) {
            Ok(groups) => groups,
            Err(_) => return Array::new(),
        };

        encode_path_groups_vec2(path_groups)
    }

    #[func]
    fn pathfind_grid_gpu_batch(
        &self,
        width: i32,
        height: i32,
        walkable: PackedByteArray,
        move_cost: PackedFloat32Array,
        from_points: PackedVector2Array,
        to_points: PackedVector2Array,
        max_steps: i32,
    ) -> Array<PackedVector2Array> {
        let steps = normalize_max_steps(max_steps);

        let path_groups = match dispatch_pathfind_grid_batch_vec2_bytes(
            PATHFIND_BACKEND_GPU,
            width,
            height,
            walkable.as_slice(),
            move_cost.as_slice(),
            from_points.as_slice(),
            to_points.as_slice(),
            steps,
        ) {
            Ok(groups) => groups,
            Err(_) => return Array::new(),
        };

        encode_path_groups_vec2(path_groups)
    }

    #[func]
    fn pathfind_grid_batch_xy(
        &self,
        width: i32,
        height: i32,
        walkable: PackedByteArray,
        move_cost: PackedFloat32Array,
        from_xy: PackedInt32Array,
        to_xy: PackedInt32Array,
        max_steps: i32,
    ) -> Array<PackedInt32Array> {
        let steps = normalize_max_steps(max_steps);
        let backend_mode = get_backend_mode();

        let path_groups = match dispatch_pathfind_grid_batch_xy_bytes(
            backend_mode,
            width,
            height,
            walkable.as_slice(),
            move_cost.as_slice(),
            from_xy.as_slice(),
            to_xy.as_slice(),
            steps,
        ) {
            Ok(groups) => groups,
            Err(_) => return Array::new(),
        };

        encode_path_groups_xy(path_groups)
    }

    #[func]
    fn pathfind_grid_gpu_batch_xy(
        &self,
        width: i32,
        height: i32,
        walkable: PackedByteArray,
        move_cost: PackedFloat32Array,
        from_xy: PackedInt32Array,
        to_xy: PackedInt32Array,
        max_steps: i32,
    ) -> Array<PackedInt32Array> {
        let steps = normalize_max_steps(max_steps);

        let path_groups = match dispatch_pathfind_grid_batch_xy_bytes(
            PATHFIND_BACKEND_GPU,
            width,
            height,
            walkable.as_slice(),
            move_cost.as_slice(),
            from_xy.as_slice(),
            to_xy.as_slice(),
            steps,
        ) {
            Ok(groups) => groups,
            Err(_) => return Array::new(),
        };

        encode_path_groups_xy(path_groups)
    }

    #[func]
    fn stat_log_xp_required(
        &self,
        level: i32,
        base_xp: f32,
        exponent: f32,
        level_breakpoints: PackedInt32Array,
        breakpoint_multipliers: PackedFloat32Array,
    ) -> f32 {
        let breakpoints = packed_i32_to_vec(&level_breakpoints);
        let multipliers = packed_f32_to_vec(&breakpoint_multipliers);
        stat_curve::log_xp_required(level, base_xp, exponent, &breakpoints, &multipliers)
    }

    #[func]
    fn stat_xp_to_level(
        &self,
        xp: f32,
        base_xp: f32,
        exponent: f32,
        level_breakpoints: PackedInt32Array,
        breakpoint_multipliers: PackedFloat32Array,
        max_level: i32,
    ) -> i32 {
        let breakpoints = packed_i32_to_vec(&level_breakpoints);
        let multipliers = packed_f32_to_vec(&breakpoint_multipliers);
        stat_curve::xp_to_level(xp, base_xp, exponent, &breakpoints, &multipliers, max_level)
    }

    #[func]
    fn stat_skill_xp_progress(
        &self,
        level: i32,
        xp: f32,
        base_xp: f32,
        exponent: f32,
        level_breakpoints: PackedInt32Array,
        breakpoint_multipliers: PackedFloat32Array,
        max_level: i32,
    ) -> VarDictionary {
        let breakpoints = packed_i32_to_vec(&level_breakpoints);
        let multipliers = packed_f32_to_vec(&breakpoint_multipliers);
        let clamped_max = max_level.max(0);
        let clamped_level = level.clamp(0, clamped_max);

        let mut xp_at_level = 0.0_f32;
        for lv in 1..=clamped_level {
            xp_at_level +=
                stat_curve::log_xp_required(lv, base_xp, exponent, &breakpoints, &multipliers);
        }

        let xp_to_next = if clamped_level < clamped_max {
            stat_curve::log_xp_required(
                clamped_level + 1,
                base_xp,
                exponent,
                &breakpoints,
                &multipliers,
            )
        } else {
            0.0_f32
        };

        let progress = xp - xp_at_level;

        let mut dict = VarDictionary::new();
        dict.set("level", clamped_level);
        dict.set("max_level", clamped_max);
        dict.set("xp_at_level", xp_at_level as f64);
        dict.set("xp_to_next", xp_to_next as f64);
        dict.set("progress_in_level", progress as f64);
        dict
    }

    #[func]
    fn stat_scurve_speed(
        &self,
        current_value: i32,
        phase_breakpoints: PackedInt32Array,
        phase_speeds: PackedFloat32Array,
    ) -> f32 {
        let breakpoints = packed_i32_to_vec(&phase_breakpoints);
        let speeds = packed_f32_to_vec(&phase_speeds);
        stat_curve::scurve_speed(current_value, &breakpoints, &speeds)
    }

    #[func]
    fn stat_need_decay(
        &self,
        current: i32,
        decay_per_year: i32,
        ticks_elapsed: i32,
        metabolic_mult: f32,
        ticks_per_year: i32,
    ) -> i32 {
        stat_curve::need_decay(
            current,
            decay_per_year,
            ticks_elapsed,
            metabolic_mult,
            ticks_per_year,
        )
    }

    #[func]
    fn stat_stress_continuous_inputs(
        &self,
        hunger: f32,
        energy: f32,
        social: f32,
        appraisal_scale: f32,
    ) -> VarDictionary {
        let out = stat_curve::stress_continuous_inputs(hunger, energy, social, appraisal_scale);

        let mut dict = VarDictionary::new();
        dict.set("hunger", out.hunger as f64);
        dict.set("energy_deficit", out.energy_deficit as f64);
        dict.set("social_isolation", out.social_isolation as f64);
        dict.set("total", out.total as f64);
        dict
    }

    #[func]
    fn stat_stress_appraisal_scale(
        &self,
        hunger: f32,
        energy: f32,
        social: f32,
        threat: f32,
        conflict: f32,
        support_score: f32,
        extroversion: f32,
        fear_value: f32,
        trust_value: f32,
        conscientiousness: f32,
        openness: f32,
        reserve_ratio: f32,
    ) -> f32 {
        stat_curve::stress_appraisal_scale(
            hunger,
            energy,
            social,
            threat,
            conflict,
            support_score,
            extroversion,
            fear_value,
            trust_value,
            conscientiousness,
            openness,
            reserve_ratio,
        )
    }

    #[func]
    fn stat_stress_primary_step(
        &self,
        hunger: f32,
        energy: f32,
        social: f32,
        threat: f32,
        conflict: f32,
        support_score: f32,
        extroversion: f32,
        fear_value: f32,
        trust_value: f32,
        conscientiousness: f32,
        openness: f32,
        reserve_ratio: f32,
    ) -> VarDictionary {
        let out = stat_curve::stress_primary_step(
            hunger,
            energy,
            social,
            threat,
            conflict,
            support_score,
            extroversion,
            fear_value,
            trust_value,
            conscientiousness,
            openness,
            reserve_ratio,
        );

        let mut dict = VarDictionary::new();
        dict.set("appraisal_scale", out.appraisal_scale as f64);
        dict.set("hunger", out.hunger as f64);
        dict.set("energy_deficit", out.energy_deficit as f64);
        dict.set("social_isolation", out.social_isolation as f64);
        dict.set("total", out.total as f64);
        dict
    }

    #[func]
    fn stat_stress_emotion_contribution(
        &self,
        fear: f32,
        anger: f32,
        sadness: f32,
        disgust: f32,
        surprise: f32,
        joy: f32,
        trust: f32,
        anticipation: f32,
        valence: f32,
        arousal: f32,
    ) -> VarDictionary {
        let out = stat_curve::stress_emotion_contribution(
            fear,
            anger,
            sadness,
            disgust,
            surprise,
            joy,
            trust,
            anticipation,
            valence,
            arousal,
        );

        let mut dict = VarDictionary::new();
        dict.set("fear", out.fear as f64);
        dict.set("anger", out.anger as f64);
        dict.set("sadness", out.sadness as f64);
        dict.set("disgust", out.disgust as f64);
        dict.set("surprise", out.surprise as f64);
        dict.set("joy", out.joy as f64);
        dict.set("trust", out.trust as f64);
        dict.set("anticipation", out.anticipation as f64);
        dict.set("va_composite", out.va_composite as f64);
        dict.set("total", out.total as f64);
        dict
    }

    #[func]
    fn stat_stress_recovery_value(
        &self,
        stress: f32,
        support_score: f32,
        resilience: f32,
        reserve: f32,
        is_sleeping: bool,
        is_safe: bool,
    ) -> f32 {
        stat_curve::stress_recovery_value(
            stress,
            support_score,
            resilience,
            reserve,
            is_sleeping,
            is_safe,
        )
    }

    #[func]
    fn stat_stress_emotion_recovery_delta_step(
        &self,
        emotion_inputs: PackedFloat32Array,
        scalar_inputs: PackedFloat32Array,
        flags: PackedByteArray,
    ) -> VarDictionary {
        let e = emotion_inputs.as_slice();
        let s = scalar_inputs.as_slice();
        let f = flags.as_slice();
        let ef = |idx: usize, fallback: f32| -> f32 { e.get(idx).copied().unwrap_or(fallback) };
        let sf = |idx: usize, fallback: f32| -> f32 { s.get(idx).copied().unwrap_or(fallback) };
        let bf = |idx: usize| -> bool { f.get(idx).copied().unwrap_or(0_u8) != 0_u8 };

        let out = stat_curve::stress_emotion_recovery_delta_step(
            ef(0, 0.0),
            ef(1, 0.0),
            ef(2, 0.0),
            ef(3, 0.0),
            ef(4, 0.0),
            ef(5, 0.0),
            ef(6, 0.0),
            ef(7, 0.0),
            ef(8, 0.0),
            ef(9, 0.0),
            sf(0, 0.0),
            sf(1, 0.3),
            sf(2, 0.5),
            sf(3, 50.0),
            bf(0),
            bf(1),
            sf(4, 0.0),
            sf(5, 0.0),
            sf(6, 1.0),
            sf(7, 1.0),
            sf(8, 0.05),
            bf(2),
            sf(9, 0.6),
            sf(10, 0.0),
            sf(11, 800.0),
        );

        let mut dict = VarDictionary::new();
        dict.set("fear", out.fear as f64);
        dict.set("anger", out.anger as f64);
        dict.set("sadness", out.sadness as f64);
        dict.set("disgust", out.disgust as f64);
        dict.set("surprise", out.surprise as f64);
        dict.set("joy", out.joy as f64);
        dict.set("trust", out.trust as f64);
        dict.set("anticipation", out.anticipation as f64);
        dict.set("va_composite", out.va_composite as f64);
        dict.set("emotion_total", out.emotion_total as f64);
        dict.set("recovery", out.recovery as f64);
        dict.set("delta", out.delta as f64);
        dict.set(
            "hidden_threat_accumulator",
            out.hidden_threat_accumulator as f64,
        );
        dict
    }

    #[func]
    fn stat_stress_trace_emotion_recovery_delta_step(
        &self,
        per_tick: PackedFloat32Array,
        decay_rate: PackedFloat32Array,
        min_keep: f32,
        emotion_inputs: PackedFloat32Array,
        scalar_inputs: PackedFloat32Array,
        flags: PackedByteArray,
    ) -> VarDictionary {
        let e = emotion_inputs.as_slice();
        let s = scalar_inputs.as_slice();
        let f = flags.as_slice();
        let ef = |idx: usize, fallback: f32| -> f32 { e.get(idx).copied().unwrap_or(fallback) };
        let sf = |idx: usize, fallback: f32| -> f32 { s.get(idx).copied().unwrap_or(fallback) };
        let bf = |idx: usize| -> bool { f.get(idx).copied().unwrap_or(0_u8) != 0_u8 };

        let out = stat_curve::stress_trace_emotion_recovery_delta_step(
            per_tick.as_slice(),
            decay_rate.as_slice(),
            min_keep,
            ef(0, 0.0),
            ef(1, 0.0),
            ef(2, 0.0),
            ef(3, 0.0),
            ef(4, 0.0),
            ef(5, 0.0),
            ef(6, 0.0),
            ef(7, 0.0),
            ef(8, 0.0),
            ef(9, 0.0),
            sf(0, 0.0),
            sf(1, 0.3),
            sf(2, 0.5),
            sf(3, 50.0),
            bf(0),
            bf(1),
            sf(4, 0.0),
            sf(5, 1.0),
            sf(6, 1.0),
            sf(7, 0.05),
            bf(2),
            sf(8, 0.6),
            sf(9, 0.0),
            sf(10, 800.0),
        );

        let mut dict = VarDictionary::new();
        dict.set(
            "total_trace_contribution",
            out.total_trace_contribution as f64,
        );
        dict.set("updated_per_tick", vec_f32_to_packed(out.updated_per_tick));
        dict.set("active_mask", vec_u8_to_packed(out.active_mask));
        dict.set("fear", out.fear as f64);
        dict.set("anger", out.anger as f64);
        dict.set("sadness", out.sadness as f64);
        dict.set("disgust", out.disgust as f64);
        dict.set("surprise", out.surprise as f64);
        dict.set("joy", out.joy as f64);
        dict.set("trust", out.trust as f64);
        dict.set("anticipation", out.anticipation as f64);
        dict.set("va_composite", out.va_composite as f64);
        dict.set("emotion_total", out.emotion_total as f64);
        dict.set("recovery", out.recovery as f64);
        dict.set("delta", out.delta as f64);
        dict.set(
            "hidden_threat_accumulator",
            out.hidden_threat_accumulator as f64,
        );
        dict
    }

    #[func]
    fn stat_stress_tick_step(
        &self,
        per_tick: PackedFloat32Array,
        decay_rate: PackedFloat32Array,
        min_keep: f32,
        scalar_inputs: PackedFloat32Array,
        flags: PackedByteArray,
    ) -> VarDictionary {
        let s = scalar_inputs.as_slice();
        let f = flags.as_slice();
        let sf = |idx: usize, fallback: f32| -> f32 { s.get(idx).copied().unwrap_or(fallback) };
        let bf = |idx: usize| -> bool { f.get(idx).copied().unwrap_or(0_u8) != 0_u8 };

        let out = stat_curve::stress_tick_step(
            per_tick.as_slice(),
            decay_rate.as_slice(),
            min_keep,
            sf(0, 0.5),         // hunger
            sf(1, 0.5),         // energy
            sf(2, 0.5),         // social
            sf(3, 0.0),         // threat
            sf(4, 0.0),         // conflict
            sf(5, 0.3),         // support_score
            sf(6, 0.5),         // extroversion
            sf(7, 0.0),         // fear
            sf(8, 0.0),         // trust
            sf(9, 0.5),         // conscientiousness
            sf(10, 0.5),        // openness
            sf(11, 0.5),        // reserve_ratio
            sf(12, 0.0),        // anger
            sf(13, 0.0),        // sadness
            sf(14, 0.0),        // disgust
            sf(15, 0.0),        // surprise
            sf(16, 0.0),        // joy
            sf(17, 0.0),        // anticipation
            sf(18, 0.0),        // valence
            sf(19, 0.0),        // arousal
            sf(20, 0.0),        // stress
            sf(21, 0.5),        // resilience
            sf(22, 50.0),       // reserve
            sf(23, 0.0),        // stress_delta_last
            sf(24, 0.0) as i32, // gas_stage
            bf(0),              // is_sleeping
            bf(1),              // is_safe
            sf(25, 0.0),        // allostatic
            sf(26, 1.0),        // ace_stress_mult
            sf(27, 1.0),        // trait_accum_mult
            sf(28, 0.05),       // epsilon
            bf(2),              // denial_active
            sf(29, 0.6),        // denial_redirect_fraction
            sf(30, 0.0),        // hidden_threat_accumulator
            sf(31, 800.0),      // denial_max_accumulator
            sf(32, 1.0),        // avoidant_allostatic_mult
            sf(33, 0.5),        // e_axis
            sf(34, 0.5),        // c_axis
            sf(35, 0.5),        // x_axis
            sf(36, 0.5),        // o_axis
            sf(37, 0.5),        // a_axis
            sf(38, 0.5),        // h_axis
            sf(39, 0.0),        // scar_resilience_mod
        );

        let mut dict = VarDictionary::new();
        dict.set("appraisal_scale", out.appraisal_scale as f64);
        dict.set("hunger", out.hunger as f64);
        dict.set("energy_deficit", out.energy_deficit as f64);
        dict.set("social_isolation", out.social_isolation as f64);
        dict.set("continuous_total", out.continuous_total as f64);
        dict.set(
            "total_trace_contribution",
            out.total_trace_contribution as f64,
        );
        dict.set("updated_per_tick", vec_f32_to_packed(out.updated_per_tick));
        dict.set("active_mask", vec_u8_to_packed(out.active_mask));
        dict.set("fear", out.fear as f64);
        dict.set("anger", out.anger as f64);
        dict.set("sadness", out.sadness as f64);
        dict.set("disgust", out.disgust as f64);
        dict.set("surprise", out.surprise as f64);
        dict.set("joy", out.joy as f64);
        dict.set("trust", out.trust as f64);
        dict.set("anticipation", out.anticipation as f64);
        dict.set("va_composite", out.va_composite as f64);
        dict.set("emotion_total", out.emotion_total as f64);
        dict.set("recovery", out.recovery as f64);
        dict.set("delta", out.delta as f64);
        dict.set(
            "hidden_threat_accumulator",
            out.hidden_threat_accumulator as f64,
        );
        dict.set("stress", out.stress as f64);
        dict.set("reserve", out.reserve as f64);
        dict.set("gas_stage", out.gas_stage);
        dict.set("allostatic", out.allostatic as f64);
        dict.set("stress_state", out.stress_state);
        dict.set("stress_mu_sadness", out.stress_mu_sadness as f64);
        dict.set("stress_mu_anger", out.stress_mu_anger as f64);
        dict.set("stress_mu_fear", out.stress_mu_fear as f64);
        dict.set("stress_mu_joy", out.stress_mu_joy as f64);
        dict.set("stress_mu_trust", out.stress_mu_trust as f64);
        dict.set("stress_neg_gain_mult", out.stress_neg_gain_mult as f64);
        dict.set("stress_pos_gain_mult", out.stress_pos_gain_mult as f64);
        dict.set("stress_blunt_mult", out.stress_blunt_mult as f64);
        dict.set("resilience", out.resilience as f64);
        dict
    }

    #[func]
    fn stat_stress_tick_step_packed(
        &self,
        per_tick: PackedFloat32Array,
        decay_rate: PackedFloat32Array,
        min_keep: f32,
        scalar_inputs: PackedFloat32Array,
        flags: PackedByteArray,
    ) -> VarDictionary {
        let s = scalar_inputs.as_slice();
        let f = flags.as_slice();
        let sf = |idx: usize, fallback: f32| -> f32 { s.get(idx).copied().unwrap_or(fallback) };
        let bf = |idx: usize| -> bool { f.get(idx).copied().unwrap_or(0_u8) != 0_u8 };

        let out = stat_curve::stress_tick_step(
            per_tick.as_slice(),
            decay_rate.as_slice(),
            min_keep,
            sf(0, 0.5),
            sf(1, 0.5),
            sf(2, 0.5),
            sf(3, 0.0),
            sf(4, 0.0),
            sf(5, 0.3),
            sf(6, 0.5),
            sf(7, 0.0),
            sf(8, 0.0),
            sf(9, 0.5),
            sf(10, 0.5),
            sf(11, 0.5),
            sf(12, 0.0),
            sf(13, 0.0),
            sf(14, 0.0),
            sf(15, 0.0),
            sf(16, 0.0),
            sf(17, 0.0),
            sf(18, 0.0),
            sf(19, 0.0),
            sf(20, 0.0),
            sf(21, 0.5),
            sf(22, 50.0),
            sf(23, 0.0),
            sf(24, 0.0) as i32,
            bf(0),
            bf(1),
            sf(25, 0.0),
            sf(26, 1.0),
            sf(27, 1.0),
            sf(28, 0.05),
            bf(2),
            sf(29, 0.6),
            sf(30, 0.0),
            sf(31, 800.0),
            sf(32, 1.0),
            sf(33, 0.5),
            sf(34, 0.5),
            sf(35, 0.5),
            sf(36, 0.5),
            sf(37, 0.5),
            sf(38, 0.5),
            sf(39, 0.0),
        );

        let scalars: Vec<f32> = vec![
            out.appraisal_scale,
            out.hunger,
            out.energy_deficit,
            out.social_isolation,
            out.total_trace_contribution,
            out.fear,
            out.anger,
            out.sadness,
            out.disgust,
            out.surprise,
            out.joy,
            out.trust,
            out.anticipation,
            out.va_composite,
            out.recovery,
            out.delta,
            out.hidden_threat_accumulator,
            out.stress,
            out.reserve,
            out.allostatic,
            out.resilience,
            out.stress_mu_sadness,
            out.stress_mu_anger,
            out.stress_mu_fear,
            out.stress_mu_joy,
            out.stress_mu_trust,
            out.stress_neg_gain_mult,
            out.stress_pos_gain_mult,
            out.stress_blunt_mult,
            out.continuous_total,
            out.emotion_total,
        ];
        let ints: Vec<i32> = vec![out.gas_stage, out.stress_state];

        let mut dict = VarDictionary::new();
        dict.set("scalars", vec_f32_to_packed(scalars));
        dict.set("ints", vec_i32_to_packed(ints));
        dict.set("updated_per_tick", vec_f32_to_packed(out.updated_per_tick));
        dict.set("active_mask", vec_u8_to_packed(out.active_mask));
        dict
    }

    #[func]
    fn stat_stress_delta_step(
        &self,
        continuous_input: f32,
        trace_input: f32,
        emotion_input: f32,
        ace_stress_mult: f32,
        trait_accum_mult: f32,
        recovery: f32,
        epsilon: f32,
        denial_active: bool,
        denial_redirect_fraction: f32,
        hidden_threat_accumulator: f32,
        denial_max_accumulator: f32,
    ) -> VarDictionary {
        let out = stat_curve::stress_delta_step(
            continuous_input,
            trace_input,
            emotion_input,
            ace_stress_mult,
            trait_accum_mult,
            recovery,
            epsilon,
            denial_active,
            denial_redirect_fraction,
            hidden_threat_accumulator,
            denial_max_accumulator,
        );

        let mut dict = VarDictionary::new();
        dict.set("delta", out.delta as f64);
        dict.set(
            "hidden_threat_accumulator",
            out.hidden_threat_accumulator as f64,
        );
        dict
    }

    #[func]
    fn stat_stress_post_update_step(
        &self,
        reserve: f32,
        stress: f32,
        resilience: f32,
        stress_delta_last: f32,
        gas_stage: i32,
        is_sleeping: bool,
        allostatic: f32,
        avoidant_allostatic_mult: f32,
    ) -> VarDictionary {
        let out = stat_curve::stress_post_update_step(
            reserve,
            stress,
            resilience,
            stress_delta_last,
            gas_stage,
            is_sleeping,
            allostatic,
            avoidant_allostatic_mult,
        );

        let mut dict = VarDictionary::new();
        dict.set("reserve", out.reserve as f64);
        dict.set("gas_stage", out.gas_stage);
        dict.set("allostatic", out.allostatic as f64);
        dict.set("stress_state", out.stress_state);
        dict.set("stress_mu_sadness", out.stress_mu_sadness as f64);
        dict.set("stress_mu_anger", out.stress_mu_anger as f64);
        dict.set("stress_mu_fear", out.stress_mu_fear as f64);
        dict.set("stress_mu_joy", out.stress_mu_joy as f64);
        dict.set("stress_mu_trust", out.stress_mu_trust as f64);
        dict.set("stress_neg_gain_mult", out.stress_neg_gain_mult as f64);
        dict.set("stress_pos_gain_mult", out.stress_pos_gain_mult as f64);
        dict.set("stress_blunt_mult", out.stress_blunt_mult as f64);
        dict
    }

    #[func]
    fn stat_stress_post_update_resilience_step(
        &self,
        scalar_inputs: PackedFloat32Array,
        flags: PackedByteArray,
    ) -> VarDictionary {
        let s = scalar_inputs.as_slice();
        let f = flags.as_slice();
        let sf = |idx: usize, fallback: f32| -> f32 { s.get(idx).copied().unwrap_or(fallback) };
        let bf = |idx: usize| -> bool { f.get(idx).copied().unwrap_or(0_u8) != 0_u8 };

        let out = stat_curve::stress_post_update_resilience_step(
            sf(0, 0.0),        // reserve
            sf(1, 0.0),        // stress
            sf(2, 0.5),        // resilience
            sf(3, 0.0),        // stress_delta_last
            sf(4, 0.0) as i32, // gas_stage
            bf(0),             // is_sleeping
            sf(5, 0.0),        // allostatic
            sf(6, 1.0),        // avoidant_allostatic_mult
            sf(7, 0.5),        // e_axis
            sf(8, 0.5),        // c_axis
            sf(9, 0.5),        // x_axis
            sf(10, 0.5),       // o_axis
            sf(11, 0.5),       // a_axis
            sf(12, 0.5),       // h_axis
            sf(13, 0.3),       // support_score
            sf(14, 0.5),       // hunger
            sf(15, 0.5),       // energy
            sf(16, 0.0),       // scar_resilience_mod
        );

        let mut dict = VarDictionary::new();
        dict.set("reserve", out.reserve as f64);
        dict.set("gas_stage", out.gas_stage);
        dict.set("allostatic", out.allostatic as f64);
        dict.set("stress_state", out.stress_state);
        dict.set("stress_mu_sadness", out.stress_mu_sadness as f64);
        dict.set("stress_mu_anger", out.stress_mu_anger as f64);
        dict.set("stress_mu_fear", out.stress_mu_fear as f64);
        dict.set("stress_mu_joy", out.stress_mu_joy as f64);
        dict.set("stress_mu_trust", out.stress_mu_trust as f64);
        dict.set("stress_neg_gain_mult", out.stress_neg_gain_mult as f64);
        dict.set("stress_pos_gain_mult", out.stress_pos_gain_mult as f64);
        dict.set("stress_blunt_mult", out.stress_blunt_mult as f64);
        dict.set("resilience", out.resilience as f64);
        dict
    }

    #[func]
    fn stat_stress_reserve_step(
        &self,
        reserve: f32,
        stress: f32,
        resilience: f32,
        stress_delta_last: f32,
        gas_stage: i32,
        is_sleeping: bool,
    ) -> VarDictionary {
        let out = stat_curve::stress_reserve_step(
            reserve,
            stress,
            resilience,
            stress_delta_last,
            gas_stage,
            is_sleeping,
        );

        let mut dict = VarDictionary::new();
        dict.set("reserve", out.reserve as f64);
        dict.set("gas_stage", out.gas_stage);
        dict
    }

    #[func]
    fn stat_stress_allostatic_step(
        &self,
        allostatic: f32,
        stress: f32,
        avoidant_allostatic_mult: f32,
    ) -> f32 {
        stat_curve::stress_allostatic_step(allostatic, stress, avoidant_allostatic_mult)
    }

    #[func]
    fn stat_stress_state_snapshot(&self, stress: f32, allostatic: f32) -> VarDictionary {
        let out = stat_curve::stress_state_snapshot(stress, allostatic);
        let mut dict = VarDictionary::new();
        dict.set("stress_state", out.stress_state);
        dict.set("stress_mu_sadness", out.stress_mu_sadness as f64);
        dict.set("stress_mu_anger", out.stress_mu_anger as f64);
        dict.set("stress_mu_fear", out.stress_mu_fear as f64);
        dict.set("stress_mu_joy", out.stress_mu_joy as f64);
        dict.set("stress_mu_trust", out.stress_mu_trust as f64);
        dict.set("stress_neg_gain_mult", out.stress_neg_gain_mult as f64);
        dict.set("stress_pos_gain_mult", out.stress_pos_gain_mult as f64);
        dict.set("stress_blunt_mult", out.stress_blunt_mult as f64);
        dict
    }

    #[func]
    fn stat_stress_trace_batch_step(
        &self,
        per_tick: PackedFloat32Array,
        decay_rate: PackedFloat32Array,
        min_keep: f32,
    ) -> VarDictionary {
        let out = stat_curve::stress_trace_batch_step(
            per_tick.as_slice(),
            decay_rate.as_slice(),
            min_keep,
        );
        let mut dict = VarDictionary::new();
        dict.set("total_contribution", out.total_contribution as f64);
        dict.set("updated_per_tick", vec_f32_to_packed(out.updated_per_tick));
        dict.set("active_mask", vec_u8_to_packed(out.active_mask));
        dict
    }

    #[func]
    fn stat_stress_resilience_value(
        &self,
        e_axis: f32,
        c_axis: f32,
        x_axis: f32,
        o_axis: f32,
        a_axis: f32,
        h_axis: f32,
        support_score: f32,
        allostatic: f32,
        hunger: f32,
        energy: f32,
        scar_resilience_mod: f32,
    ) -> f32 {
        stat_curve::stress_resilience_value(
            e_axis,
            c_axis,
            x_axis,
            o_axis,
            a_axis,
            h_axis,
            support_score,
            allostatic,
            hunger,
            energy,
            scar_resilience_mod,
        )
    }

    #[func]
    fn stat_stress_work_efficiency(&self, stress: f32, shaken_penalty: f32) -> f32 {
        stat_curve::stress_work_efficiency(stress, shaken_penalty)
    }

    #[func]
    fn stat_stress_personality_scale(
        &self,
        values: PackedFloat32Array,
        weights: PackedFloat32Array,
        high_amplifies: PackedByteArray,
        trait_multipliers: PackedFloat32Array,
    ) -> f32 {
        stat_curve::stress_personality_scale(
            &packed_f32_to_vec(&values),
            &packed_f32_to_vec(&weights),
            &packed_u8_to_vec(&high_amplifies),
            &packed_f32_to_vec(&trait_multipliers),
        )
    }

    #[func]
    fn stat_stress_relationship_scale(
        &self,
        method: GString,
        bond_strength: f32,
        min_mult: f32,
        max_mult: f32,
    ) -> f32 {
        let method_string = method.to_string();
        stat_curve::stress_relationship_scale(&method_string, bond_strength, min_mult, max_mult)
    }

    #[func]
    fn stat_stress_context_scale(&self, active_multipliers: PackedFloat32Array) -> f32 {
        stat_curve::stress_context_scale(&packed_f32_to_vec(&active_multipliers))
    }

    #[func]
    fn stat_stress_emotion_inject_step(
        &self,
        fast_current: PackedFloat32Array,
        slow_current: PackedFloat32Array,
        fast_inject: PackedFloat32Array,
        slow_inject: PackedFloat32Array,
        scale: f32,
    ) -> VarDictionary {
        let out = stat_curve::stress_emotion_inject_step(
            &packed_f32_to_vec(&fast_current),
            &packed_f32_to_vec(&slow_current),
            &packed_f32_to_vec(&fast_inject),
            &packed_f32_to_vec(&slow_inject),
            scale,
        );
        let mut dict = VarDictionary::new();
        dict.set("fast", vec_f32_to_packed(out.fast));
        dict.set("slow", vec_f32_to_packed(out.slow));
        dict
    }

    #[func]
    fn stat_stress_rebound_queue_step(
        &self,
        amounts: PackedFloat32Array,
        delays: PackedInt32Array,
        decay_per_tick: f32,
    ) -> VarDictionary {
        let out = stat_curve::stress_rebound_queue_step(
            &packed_f32_to_vec(&amounts),
            &packed_i32_to_vec(&delays),
            decay_per_tick,
        );
        let mut dict = VarDictionary::new();
        dict.set("total_rebound", out.total_rebound as f64);
        dict.set(
            "remaining_amounts",
            vec_f32_to_packed(out.remaining_amounts),
        );
        dict.set("remaining_delays", vec_i32_to_packed(out.remaining_delays));
        dict
    }

    #[func]
    fn stat_stress_event_scale_step(
        &self,
        base_instant: f32,
        base_per_tick: f32,
        is_loss: bool,
        personality_scale: f32,
        appraisal_scale: f32,
        relationship_method: GString,
        bond_strength: f32,
        relationship_min_mult: f32,
        relationship_max_mult: f32,
        context_active_multipliers: PackedFloat32Array,
    ) -> VarDictionary {
        let out = stat_curve::stress_event_scale_step(
            base_instant,
            base_per_tick,
            is_loss,
            personality_scale,
            appraisal_scale,
            &relationship_method.to_string(),
            bond_strength,
            relationship_min_mult,
            relationship_max_mult,
            &packed_f32_to_vec(&context_active_multipliers),
        );
        let mut dict = VarDictionary::new();
        dict.set("relationship_scale", out.relationship_scale as f64);
        dict.set("context_scale", out.context_scale as f64);
        dict.set("total_scale", out.total_scale as f64);
        dict.set("loss_mult", out.loss_mult as f64);
        dict.set("final_instant", out.final_instant as f64);
        dict.set("final_per_tick", out.final_per_tick as f64);
        dict
    }

    #[func]
    fn stat_stress_event_scale_step_code(
        &self,
        base_instant: f32,
        base_per_tick: f32,
        is_loss: bool,
        personality_scale: f32,
        appraisal_scale: f32,
        relationship_method_code: i32,
        bond_strength: f32,
        relationship_min_mult: f32,
        relationship_max_mult: f32,
        context_active_multipliers: PackedFloat32Array,
    ) -> VarDictionary {
        let out = stat_curve::stress_event_scale_step_code(
            base_instant,
            base_per_tick,
            is_loss,
            personality_scale,
            appraisal_scale,
            relationship_method_code,
            bond_strength,
            relationship_min_mult,
            relationship_max_mult,
            &packed_f32_to_vec(&context_active_multipliers),
        );
        let mut dict = VarDictionary::new();
        dict.set("relationship_scale", out.relationship_scale as f64);
        dict.set("context_scale", out.context_scale as f64);
        dict.set("total_scale", out.total_scale as f64);
        dict.set("loss_mult", out.loss_mult as f64);
        dict.set("final_instant", out.final_instant as f64);
        dict.set("final_per_tick", out.final_per_tick as f64);
        dict
    }

    #[func]
    fn stat_stress_event_inject_step(
        &self,
        base_instant: f32,
        base_per_tick: f32,
        is_loss: bool,
        personality_scale: f32,
        appraisal_scale: f32,
        relationship_method: GString,
        bond_strength: f32,
        relationship_min_mult: f32,
        relationship_max_mult: f32,
        context_active_multipliers: PackedFloat32Array,
        fast_current: PackedFloat32Array,
        slow_current: PackedFloat32Array,
        fast_inject: PackedFloat32Array,
        slow_inject: PackedFloat32Array,
    ) -> VarDictionary {
        let out = stat_curve::stress_event_inject_step(
            base_instant,
            base_per_tick,
            is_loss,
            personality_scale,
            appraisal_scale,
            &relationship_method.to_string(),
            bond_strength,
            relationship_min_mult,
            relationship_max_mult,
            &packed_f32_to_vec(&context_active_multipliers),
            &packed_f32_to_vec(&fast_current),
            &packed_f32_to_vec(&slow_current),
            &packed_f32_to_vec(&fast_inject),
            &packed_f32_to_vec(&slow_inject),
        );
        let mut dict = VarDictionary::new();
        dict.set("relationship_scale", out.relationship_scale as f64);
        dict.set("context_scale", out.context_scale as f64);
        dict.set("total_scale", out.total_scale as f64);
        dict.set("loss_mult", out.loss_mult as f64);
        dict.set("final_instant", out.final_instant as f64);
        dict.set("final_per_tick", out.final_per_tick as f64);
        dict.set("fast", vec_f32_to_packed(out.fast));
        dict.set("slow", vec_f32_to_packed(out.slow));
        dict
    }

    #[func]
    fn stat_stress_event_inject_step_code(
        &self,
        base_instant: f32,
        base_per_tick: f32,
        is_loss: bool,
        personality_scale: f32,
        appraisal_scale: f32,
        relationship_method_code: i32,
        bond_strength: f32,
        relationship_min_mult: f32,
        relationship_max_mult: f32,
        context_active_multipliers: PackedFloat32Array,
        fast_current: PackedFloat32Array,
        slow_current: PackedFloat32Array,
        fast_inject: PackedFloat32Array,
        slow_inject: PackedFloat32Array,
    ) -> VarDictionary {
        let out = stat_curve::stress_event_inject_step_code(
            base_instant,
            base_per_tick,
            is_loss,
            personality_scale,
            appraisal_scale,
            relationship_method_code,
            bond_strength,
            relationship_min_mult,
            relationship_max_mult,
            &packed_f32_to_vec(&context_active_multipliers),
            &packed_f32_to_vec(&fast_current),
            &packed_f32_to_vec(&slow_current),
            &packed_f32_to_vec(&fast_inject),
            &packed_f32_to_vec(&slow_inject),
        );
        let mut dict = VarDictionary::new();
        dict.set("relationship_scale", out.relationship_scale as f64);
        dict.set("context_scale", out.context_scale as f64);
        dict.set("total_scale", out.total_scale as f64);
        dict.set("loss_mult", out.loss_mult as f64);
        dict.set("final_instant", out.final_instant as f64);
        dict.set("final_per_tick", out.final_per_tick as f64);
        dict.set("fast", vec_f32_to_packed(out.fast));
        dict.set("slow", vec_f32_to_packed(out.slow));
        dict
    }

    #[func]
    fn stat_stress_event_scaled(
        &self,
        base_instant: f32,
        base_per_tick: f32,
        is_loss: bool,
        personality_scale: f32,
        relationship_scale: f32,
        context_scale: f32,
        appraisal_scale: f32,
    ) -> VarDictionary {
        let out = stat_curve::stress_event_scaled(
            base_instant,
            base_per_tick,
            is_loss,
            personality_scale,
            relationship_scale,
            context_scale,
            appraisal_scale,
        );
        let mut dict = VarDictionary::new();
        dict.set("total_scale", out.total_scale as f64);
        dict.set("loss_mult", out.loss_mult as f64);
        dict.set("final_instant", out.final_instant as f64);
        dict.set("final_per_tick", out.final_per_tick as f64);
        dict
    }

    #[func]
    fn stat_sigmoid_extreme(
        &self,
        value: i32,
        flat_zone_lo: i32,
        flat_zone_hi: i32,
        pole_multiplier: f32,
    ) -> f32 {
        stat_curve::sigmoid_extreme(value, flat_zone_lo, flat_zone_hi, pole_multiplier)
    }

    #[func]
    fn stat_power_influence(&self, value: i32, exponent: f32) -> f32 {
        stat_curve::power_influence(value, exponent)
    }

    #[func]
    fn stat_threshold_power(
        &self,
        value: i32,
        threshold: i32,
        exponent: f32,
        max_output: f32,
    ) -> f32 {
        stat_curve::threshold_power(value, threshold, exponent, max_output)
    }

    #[func]
    fn stat_linear_influence(&self, value: i32) -> f32 {
        stat_curve::linear_influence(value)
    }

    #[func]
    fn stat_step_influence(
        &self,
        value: i32,
        threshold: i32,
        above_value: f32,
        below_value: f32,
    ) -> f32 {
        stat_curve::step_influence(value, threshold, above_value, below_value)
    }

    #[func]
    fn stat_step_linear(
        &self,
        value: i32,
        below_thresholds: PackedInt32Array,
        multipliers: PackedFloat32Array,
    ) -> f32 {
        let step_pairs = build_step_pairs(&below_thresholds, &multipliers);
        stat_curve::step_linear(value, &step_pairs)
    }
}

struct SimBridgeExtension;

#[gdextension(entry_symbol = worldsim_rust_init)]
unsafe impl ExtensionLibrary for SimBridgeExtension {
    fn on_level_init(_level: godot::init::InitLevel) {
        if _level == godot::init::InitLevel::Scene {
            std::panic::set_hook(Box::new(|info| {
                let payload = if let Some(s) = info.payload().downcast_ref::<&str>() {
                    s.to_string()
                } else if let Some(s) = info.payload().downcast_ref::<String>() {
                    s.clone()
                } else {
                    "unknown panic".to_string()
                };
                let location = info
                    .location()
                    .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
                    .unwrap_or_else(|| "unknown location".to_string());
                eprintln!("[RUST PANIC] {} at {}", payload, location);
                eprintln!("[RUST PANIC] backtrace:\n{}", std::backtrace::Backtrace::force_capture());
            }));
        }
    }
}

// ── FrameSnapshot helpers ─────────────────────────────────────────────────────

fn runtime_growth_stage_to_str(gs: GrowthStage) -> &'static str {
    match gs {
        GrowthStage::Infant => "infant",
        GrowthStage::Toddler => "toddler",
        GrowthStage::Child => "child",
        GrowthStage::Teen => "teen",
        GrowthStage::Adult => "adult",
        GrowthStage::Elder => "elder",
    }
}

fn runtime_sex_to_str(sex: Sex) -> &'static str {
    match sex {
        Sex::Male => "male",
        Sex::Female => "female",
    }
}

#[cfg(test)]
mod tests {
    use super::pathfinding_core::{
        read_dispatch_counts, reset_dispatch_counts, PATHFIND_BACKEND_AUTO, PATHFIND_BACKEND_CPU,
        PATHFIND_BACKEND_GPU,
    };
    use super::{
        archetype_label_key_from_axes, build_thought_text, collect_entity_list_rows,
        decode_ws2_blob,
        dispatch_pathfind_grid_batch_vec2_bytes, dispatch_pathfind_grid_batch_xy_bytes,
        dispatch_pathfind_grid_bytes, encode_ws2_blob, format_fluent_from_source_args,
        format_story_event_locale, get_pathfind_backend_mode, has_gpu_pathfind_backend,
        is_significant_story_event, parse_pathfind_backend,
        pathfind_backend_dispatch_counts, pathfind_from_flat, pathfind_grid_batch_bytes,
        pathfind_grid_batch_dispatch_bytes, pathfind_grid_batch_vec2_bytes,
        pathfind_grid_batch_xy_bytes, pathfind_grid_batch_xy_dispatch_bytes, pathfind_grid_bytes,
        reset_pathfind_backend_dispatch_counts, resolve_backend_mode,
        resolve_pathfind_backend_mode, set_pathfind_backend_mode, NarrativeDisplayData,
        EntityListRowSnapshot, PathfindError, PathfindInput,
    };
    use crate::locale_bindings::{clear_fluent_source, locale_test_lock, store_fluent_source};
    use fluent_bundle::types::FluentNumber;
    use fluent_bundle::{FluentArgs, FluentValue};
    use godot::prelude::Vector2;
    use sim_core::band::{Band, BandStore};
    use sim_core::components::{
        Age, Behavior, Identity, LlmPending, LlmRequestType, NarrativeCache, Needs,
    };
    use sim_core::enums::{ActionType, NeedType};
    use sim_core::{BandId, EntityId, SettlementId};
    use sim_engine::{
        ChronicleCapsule, ChronicleDossierStub, ChronicleEntityRefState, ChronicleEntryId,
        ChronicleEntryLite, ChronicleEntryStatus, ChronicleEventCause, ChronicleEventType,
        ChronicleHeadline, ChronicleLocationRefLite, ChronicleQueueBucket,
        ChronicleSignificanceCategory, ChronicleSignificanceMeta, ChronicleSubjectRefLite,
        EngineSnapshot, GameEvent, LlmPromptVariant, SimEvent, SimEventType,
    };
    use sim_systems::pathfinding::GridPos;
    use std::collections::BTreeMap;

    fn base_input() -> PathfindInput {
        PathfindInput {
            width: 4,
            height: 4,
            walkable: vec![true; 16],
            move_cost: vec![1.0; 16],
            from: GridPos::new(0, 0),
            to: GridPos::new(3, 3),
            max_steps: 200,
        }
    }

    #[test]
    fn fluent_format_replaces_named_params() {
        let source = "ui-greeting = Hello, { $name }!";
        let mut args = FluentArgs::new();
        args.set("name", FluentValue::String("Aria".into()));
        let value = format_fluent_from_source_args(source, "en-US", "ui-greeting", Some(args))
            .expect("message should be formatted");
        assert_eq!(value, "Hello, Aria!");
    }

    #[test]
    fn fluent_format_supports_plural_rules() {
        let source =
            "ui-item-count = { $count ->\n    [one] One item\n   *[other] { $count } items\n}";
        let mut args = FluentArgs::new();
        args.set("count", FluentValue::Number(FluentNumber::from(3_i64)));
        let value = format_fluent_from_source_args(source, "en-US", "ui-item-count", Some(args))
            .expect("plural message should be formatted");
        assert_eq!(value, "3 items");
    }

    #[test]
    fn archetype_label_key_prefers_largest_hexaco_deviation() {
        assert_eq!(
            archetype_label_key_from_axes([0.92, 0.50, 0.51, 0.49, 0.48, 0.47]),
            "ARCHETYPE_PRINCIPLED_GUARDIAN"
        );
        assert_eq!(
            archetype_label_key_from_axes([0.50, 0.51, 0.49, 0.48, 0.52, 0.10]),
            "ARCHETYPE_PRACTICAL_REALIST"
        );
    }

    #[test]
    fn build_thought_text_bolds_first_sentence_and_mentions_need() {
        let text = build_thought_text(
            "Kaya",
            Some("joyful"),
            Some("Hunger is starting to bite."),
            Some("A recent kindness still stands out."),
            Some("Building"),
            true,
        );
        assert!(text.starts_with("[b]Kaya feels joyful.[/b]"));
        assert!(text.contains("Hunger is starting to bite."));
        assert!(text.contains("building"));
        assert!(text.contains("Tension is building."));
    }

    #[test]
    fn story_event_significance_filters_idle_and_routine_action_changes() {
        let idle_to_gather = SimEvent {
            tick: 10,
            event_type: SimEventType::ActionChanged,
            actor: 1,
            target: None,
            tags: vec!["behavior".to_string()],
            cause: "Idle->GatherWood".to_string(),
            value: 0.0,
        };
        let rest_to_sleep = SimEvent {
            tick: 11,
            event_type: SimEventType::ActionChanged,
            actor: 1,
            target: None,
            tags: vec!["behavior".to_string()],
            cause: "Rest->Sleep".to_string(),
            value: 0.0,
        };
        let flee_to_fight = SimEvent {
            tick: 12,
            event_type: SimEventType::ActionChanged,
            actor: 1,
            target: None,
            tags: vec!["behavior".to_string()],
            cause: "Flee->Fight".to_string(),
            value: 0.0,
        };

        assert!(!is_significant_story_event(&idle_to_gather));
        assert!(!is_significant_story_event(&rest_to_sleep));
        assert!(is_significant_story_event(&flee_to_fight));
    }

    #[test]
    fn format_story_event_locale_uses_active_locale_for_need_labels() {
        let _guard = locale_test_lock().lock().expect("locale test lock");
        assert!(store_fluent_source(
            "ko",
            "NEED_HUNGER = 허기\nSTORY_NEED_CRITICAL = {$actor}이(가) {$need}(으)로 고통받고 있다"
        ));

        let event = SimEvent {
            tick: 10,
            event_type: SimEventType::NeedCritical,
            actor: 1,
            target: None,
            tags: vec!["needs".to_string()],
            cause: "hunger".to_string(),
            value: 0.1,
        };

        let message = format_story_event_locale(&event, "Kaya", None);
        assert_eq!(message.locale_key, "STORY_NEED_CRITICAL");
        assert_eq!(
            message.params,
            vec![
                ("actor".to_string(), "Kaya".to_string()),
                ("need".to_string(), "허기".to_string()),
            ]
        );

        clear_fluent_source("ko");
    }

    #[test]
    fn format_story_event_locale_localizes_task_completed_params() {
        let _guard = locale_test_lock().lock().expect("locale test lock");
        assert!(store_fluent_source(
            "en",
            "STATUS_GATHER_WOOD = Gathering Wood\nSTORY_TASK_COMPLETED = {actor} finished {task}"
        ));

        let event = SimEvent {
            tick: 12,
            event_type: SimEventType::TaskCompleted,
            actor: 1,
            target: None,
            tags: vec!["task".to_string()],
            cause: "GatherWood".to_string(),
            value: 1.0,
        };

        let message = format_story_event_locale(&event, "Kaya", None);
        assert_eq!(message.locale_key, "STORY_TASK_COMPLETED");
        assert_eq!(
            message.params,
            vec![
                ("actor".to_string(), "Kaya".to_string()),
                ("task".to_string(), "Gathering Wood".to_string()),
            ]
        );

        clear_fluent_source("en");
    }

    #[test]
    fn bridge_entity_list_surfaces_hunger_and_missing_needs_fallback() {
        let mut world = hecs::World::new();
        let mut band_store = BandStore::new();
        let band_id = BandId(7);

        let mut hungry_identity = Identity::default();
        hungry_identity.name = "Kaya".to_string();
        hungry_identity.band_id = Some(band_id);
        hungry_identity.settlement_id = Some(SettlementId(5));
        let mut hungry_age = Age::default();
        hungry_age.years = 12.0;
        let mut hungry_needs = Needs::default();
        hungry_needs.set(NeedType::Hunger, 0.42);
        let mut hungry_behavior = Behavior::default();
        hungry_behavior.job = "builder".to_string();
        hungry_behavior.current_action = ActionType::GatherWood;
        let hungry_entity = world.spawn((hungry_identity, hungry_age, hungry_needs, hungry_behavior));
        let hungry_entity_id = EntityId(hungry_entity.id() as u64);
        let mut band = Band::new(band_id, "Oak".to_string(), vec![hungry_entity_id], 0, None);
        band.leader = Some(hungry_entity_id);
        band_store.insert(band);

        let mut fallback_identity = Identity::default();
        fallback_identity.name = "NoNeeds".to_string();
        let mut fallback_age = Age::default();
        fallback_age.years = 8.0;
        world.spawn((fallback_identity, fallback_age));

        let rows = collect_entity_list_rows(&world, &band_store);
        let mut rows_by_name: std::collections::BTreeMap<String, EntityListRowSnapshot> =
            std::collections::BTreeMap::new();

        for row in rows {
            rows_by_name.insert(row.name.clone(), row);
        }

        let kaya = rows_by_name.get("Kaya").expect("Kaya row");
        let fallback = rows_by_name.get("NoNeeds").expect("NoNeeds row");

        assert!((kaya.hunger - 0.42).abs() < 1.0e-6);
        assert_eq!(kaya.settlement_id, 5);
        assert_eq!(kaya.current_action, "GatherWood");
        assert!(kaya.is_leader);
        assert_eq!(fallback.hunger, 0.0);
        assert_eq!(fallback.settlement_id, -1);
        assert_eq!(fallback.current_action, "Idle");
        assert!(!fallback.is_leader);
    }

    #[test]
    fn narrative_display_data_includes_labels_and_visibility_flags() {
        let data = NarrativeDisplayData {
            personality_text: "성격 서사".to_string(),
            event_text: String::new(),
            inner_text: "내면 독백".to_string(),
            show_personality: true,
            show_event: false,
            show_inner: true,
            show_personality_shimmer: false,
            show_event_shimmer: true,
            show_inner_shimmer: false,
            show_disabled_overlay: false,
            ai_icon_state: 2,
            ai_label_tooltip: "AI".to_string(),
            panel_title: "서사".to_string(),
            section_labels: ["성격".to_string(), "사건".to_string(), "내면".to_string()],
            disabled_message: String::new(),
            ai_generated: true,
            entity_id: 42,
        };
        assert_eq!(data.panel_title, "서사");
        assert!(data.show_personality);
        assert!(!data.show_event);
        assert_eq!(data.section_labels[2], "내면");
    }

    #[test]
    fn stage1_archetype_high_extraversion() {
        let label = archetype_label_key_from_axes([0.5, 0.5, 0.95, 0.5, 0.5, 0.5]);
        assert_eq!(label, "ARCHETYPE_BOLD_EXPLORER");
    }

    #[test]
    fn stage1_archetype_low_honesty() {
        let label = archetype_label_key_from_axes([0.05, 0.5, 0.5, 0.5, 0.5, 0.5]);
        assert_eq!(label, "ARCHETYPE_CUNNING_OPPORTUNIST");
    }

    #[test]
    fn stage1_all_12_archetypes_reachable() {
        let mut seen: std::collections::HashSet<&'static str> = std::collections::HashSet::new();
        for axis in 0..6 {
            for value in [0.05_f64, 0.95_f64] {
                let mut axes = [0.5; 6];
                axes[axis] = value;
                seen.insert(archetype_label_key_from_axes(axes));
            }
        }
        assert_eq!(seen.len(), 12);
    }

    #[test]
    fn stage1_archetype_neutral_returns_something() {
        let label = archetype_label_key_from_axes([0.5; 6]);
        assert!(!label.is_empty());
    }

    #[test]
    fn validates_walkable_length() {
        let mut input = base_input();
        input.walkable.pop();
        let err = pathfind_from_flat(input).unwrap_err();
        assert_eq!(
            err,
            PathfindError::InvalidWalkableLength {
                expected: 16,
                got: 15
            }
        );
    }

    #[test]
    fn validates_move_cost_length() {
        let mut input = base_input();
        input.move_cost.pop();
        let err = pathfind_from_flat(input).unwrap_err();
        assert_eq!(
            err,
            PathfindError::InvalidMoveCostLength {
                expected: 16,
                got: 15
            }
        );
    }

    #[test]
    fn returns_path_on_valid_input() {
        let input = base_input();
        let path = pathfind_from_flat(input).expect("pathfinding should succeed");
        assert_eq!(path.first().copied(), Some(GridPos::new(0, 0)));
        assert_eq!(path.last().copied(), Some(GridPos::new(3, 3)));
    }

    #[test]
    fn pathfind_grid_accepts_byte_walkable_flags() {
        let walkable = vec![1_u8; 16];
        let move_cost = vec![1.0_f32; 16];
        let path = pathfind_grid_bytes(4, 4, &walkable, &move_cost, 0, 0, 3, 3, 200)
            .expect("pathfinding should succeed");
        assert_eq!(path.first().copied(), Some(GridPos::new(0, 0)));
        assert_eq!(path.last().copied(), Some(GridPos::new(3, 3)));
    }

    #[test]
    fn pathfind_grid_rejects_invalid_dimensions() {
        let walkable = vec![1_u8; 16];
        let move_cost = vec![1.0_f32; 16];
        let err = pathfind_grid_bytes(0, 4, &walkable, &move_cost, 0, 0, 3, 3, 200)
            .expect_err("zero width must fail");
        assert_eq!(
            err,
            PathfindError::InvalidDimensions {
                width: 0,
                height: 4
            }
        );
    }

    #[test]
    fn pathfind_grid_returns_singleton_for_stationary_query() {
        let walkable = vec![0_u8; 16];
        let move_cost = vec![1.0_f32; 16];
        let path = pathfind_grid_bytes(4, 4, &walkable, &move_cost, 2, 2, 2, 2, 200)
            .expect("stationary query should succeed");
        assert_eq!(path, vec![GridPos::new(2, 2)]);
    }

    #[test]
    fn pathfind_grid_returns_empty_when_start_is_out_of_bounds() {
        let walkable = vec![1_u8; 16];
        let move_cost = vec![1.0_f32; 16];
        let path = pathfind_grid_bytes(4, 4, &walkable, &move_cost, -1, 0, 3, 3, 200)
            .expect("out-of-bounds start should be handled");
        assert!(path.is_empty());
    }

    #[test]
    fn pathfind_grid_batch_processes_multiple_queries() {
        let walkable = vec![1_u8; 25];
        let move_cost = vec![1.0_f32; 25];
        let from = vec![(0, 0), (4, 0), (0, 4)];
        let to = vec![(4, 4), (0, 4), (4, 0)];

        let groups = pathfind_grid_batch_bytes(5, 5, &walkable, &move_cost, &from, &to, 200)
            .expect("batch pathfinding should succeed");
        assert_eq!(groups.len(), 3);
        assert_eq!(groups[0].first().copied(), Some(GridPos::new(0, 0)));
        assert_eq!(groups[0].last().copied(), Some(GridPos::new(4, 4)));
    }

    #[test]
    fn pathfind_grid_batch_rejects_mismatched_input_lengths() {
        let walkable = vec![1_u8; 16];
        let move_cost = vec![1.0_f32; 16];
        let from = vec![(0, 0), (1, 1)];
        let to = vec![(3, 3)];

        let err = pathfind_grid_batch_bytes(4, 4, &walkable, &move_cost, &from, &to, 200)
            .expect_err("mismatched input lengths must fail");
        assert_eq!(
            err,
            PathfindError::MismatchedBatchLength {
                from_len: 2,
                to_len: 1
            }
        );
    }

    #[test]
    fn pathfind_grid_batch_xy_matches_tuple_results() {
        let walkable = vec![1_u8; 36];
        let move_cost = vec![1.0_f32; 36];
        let from = vec![(0, 0), (5, 0), (0, 5), (2, 3)];
        let to = vec![(5, 5), (0, 5), (5, 0), (4, 1)];

        let mut from_xy: Vec<i32> = Vec::with_capacity(from.len() * 2);
        let mut to_xy: Vec<i32> = Vec::with_capacity(to.len() * 2);
        for idx in 0..from.len() {
            from_xy.push(from[idx].0);
            from_xy.push(from[idx].1);
            to_xy.push(to[idx].0);
            to_xy.push(to[idx].1);
        }

        let grouped = pathfind_grid_batch_bytes(6, 6, &walkable, &move_cost, &from, &to, 400)
            .expect("tuple batch should succeed");
        let grouped_xy =
            pathfind_grid_batch_xy_bytes(6, 6, &walkable, &move_cost, &from_xy, &to_xy, 400)
                .expect("xy batch should succeed");

        assert_eq!(grouped_xy, grouped);
    }

    #[test]
    fn pathfind_grid_batch_vec2_matches_tuple_results() {
        let walkable = vec![1_u8; 36];
        let move_cost = vec![1.0_f32; 36];
        let from = vec![(0, 0), (5, 0), (0, 5), (2, 3)];
        let to = vec![(5, 5), (0, 5), (5, 0), (4, 1)];
        let from_vec2 = vec![
            Vector2::new(0.0, 0.0),
            Vector2::new(5.0, 0.0),
            Vector2::new(0.0, 5.0),
            Vector2::new(2.0, 3.0),
        ];
        let to_vec2 = vec![
            Vector2::new(5.0, 5.0),
            Vector2::new(0.0, 5.0),
            Vector2::new(5.0, 0.0),
            Vector2::new(4.0, 1.0),
        ];

        let grouped = pathfind_grid_batch_bytes(6, 6, &walkable, &move_cost, &from, &to, 400)
            .expect("tuple batch should succeed");
        let grouped_vec2 =
            pathfind_grid_batch_vec2_bytes(6, 6, &walkable, &move_cost, &from_vec2, &to_vec2, 400)
                .expect("vec2 batch should succeed");

        assert_eq!(grouped_vec2, grouped);
    }

    #[test]
    fn pathfind_grid_batch_returns_singletons_for_stationary_queries() {
        let walkable = vec![0_u8; 25];
        let move_cost = vec![1.0_f32; 25];
        let from = vec![(1, 1), (2, 3), (4, 0)];
        let to = vec![(1, 1), (2, 3), (4, 0)];

        let grouped = pathfind_grid_batch_bytes(5, 5, &walkable, &move_cost, &from, &to, 200)
            .expect("stationary tuple batch should succeed");
        assert_eq!(
            grouped,
            vec![
                vec![GridPos::new(1, 1)],
                vec![GridPos::new(2, 3)],
                vec![GridPos::new(4, 0)]
            ]
        );

        let from_xy = vec![1, 1, 2, 3, 4, 0];
        let to_xy = vec![1, 1, 2, 3, 4, 0];
        let grouped_xy =
            pathfind_grid_batch_xy_bytes(5, 5, &walkable, &move_cost, &from_xy, &to_xy, 200)
                .expect("stationary xy batch should succeed");
        assert_eq!(grouped_xy, grouped);

        let from_vec2 = vec![
            Vector2::new(1.0, 1.0),
            Vector2::new(2.0, 3.0),
            Vector2::new(4.0, 0.0),
        ];
        let to_vec2 = vec![
            Vector2::new(1.0, 1.0),
            Vector2::new(2.0, 3.0),
            Vector2::new(4.0, 0.0),
        ];
        let grouped_vec2 =
            pathfind_grid_batch_vec2_bytes(5, 5, &walkable, &move_cost, &from_vec2, &to_vec2, 200)
                .expect("stationary vec2 batch should succeed");
        assert_eq!(grouped_vec2, grouped);
    }

    #[test]
    fn pathfind_grid_batch_returns_empty_for_out_of_bounds_start() {
        let walkable = vec![1_u8; 25];
        let move_cost = vec![1.0_f32; 25];
        let from = vec![(-1, 0), (0, 0)];
        let to = vec![(4, 4), (4, 4)];

        let grouped = pathfind_grid_batch_bytes(5, 5, &walkable, &move_cost, &from, &to, 200)
            .expect("batch should succeed");
        assert_eq!(grouped.len(), 2);
        assert!(grouped[0].is_empty());
        assert!(!grouped[1].is_empty());

        let from_xy = vec![-1, 0, 0, 0];
        let to_xy = vec![4, 4, 4, 4];
        let grouped_xy =
            pathfind_grid_batch_xy_bytes(5, 5, &walkable, &move_cost, &from_xy, &to_xy, 200)
                .expect("xy batch should succeed");
        assert_eq!(grouped_xy, grouped);
    }

    #[test]
    fn pathfind_grid_batch_xy_rejects_odd_length_inputs() {
        let walkable = vec![1_u8; 16];
        let move_cost = vec![1.0_f32; 16];
        let from_xy = vec![0, 0, 1];
        let to_xy = vec![3, 3, 2];

        let err = pathfind_grid_batch_xy_bytes(4, 4, &walkable, &move_cost, &from_xy, &to_xy, 200)
            .expect_err("odd-length xy arrays must fail");
        assert_eq!(
            err,
            PathfindError::MismatchedBatchLength {
                from_len: 3,
                to_len: 3
            }
        );
    }

    #[test]
    fn pathfind_grid_batch_xy_rejects_invalid_dimensions() {
        let walkable = vec![1_u8; 16];
        let move_cost = vec![1.0_f32; 16];
        let from_xy = vec![0, 0];
        let to_xy = vec![1, 1];

        let err = pathfind_grid_batch_xy_bytes(4, 0, &walkable, &move_cost, &from_xy, &to_xy, 200)
            .expect_err("zero height must fail");
        assert_eq!(
            err,
            PathfindError::InvalidDimensions {
                width: 4,
                height: 0
            }
        );
    }

    #[test]
    fn backend_dispatch_counters_track_resolved_modes() {
        reset_dispatch_counts();

        let walkable = vec![1_u8; 16];
        let move_cost = vec![1.0_f32; 16];
        let (cpu_before, gpu_before) = read_dispatch_counts();
        let _ = dispatch_pathfind_grid_bytes(
            PATHFIND_BACKEND_CPU,
            4,
            4,
            &walkable,
            &move_cost,
            0,
            0,
            3,
            3,
            200,
        )
        .expect("cpu dispatch should succeed");
        let (cpu_after_cpu_call, gpu_after_cpu_call) = read_dispatch_counts();
        let _ = dispatch_pathfind_grid_bytes(
            PATHFIND_BACKEND_GPU,
            4,
            4,
            &walkable,
            &move_cost,
            0,
            0,
            3,
            3,
            200,
        )
        .expect("gpu dispatch should succeed");
        let (cpu_after_gpu_call, gpu_after_gpu_call) = read_dispatch_counts();

        assert!(cpu_after_cpu_call + gpu_after_cpu_call >= cpu_before + gpu_before + 1);
        assert!(
            cpu_after_gpu_call + gpu_after_gpu_call >= cpu_after_cpu_call + gpu_after_cpu_call + 1
        );
        assert!(cpu_after_gpu_call >= cpu_after_cpu_call + 1);
        assert_eq!(gpu_after_gpu_call, gpu_after_cpu_call);
    }

    #[test]
    fn parses_pathfinding_backend_modes() {
        assert_eq!(parse_pathfind_backend("auto"), Some(PATHFIND_BACKEND_AUTO));
        assert_eq!(parse_pathfind_backend("cpu"), Some(PATHFIND_BACKEND_CPU));
        assert_eq!(parse_pathfind_backend("gpu"), Some(PATHFIND_BACKEND_GPU));
        assert_eq!(parse_pathfind_backend("GPU"), Some(PATHFIND_BACKEND_GPU));
        assert_eq!(parse_pathfind_backend("unknown"), None);
    }

    #[test]
    fn resolves_pathfinding_backend_with_runtime_gate() {
        assert_eq!(resolve_backend_mode(PATHFIND_BACKEND_CPU), "cpu");
        assert_eq!(resolve_backend_mode(PATHFIND_BACKEND_GPU), "cpu");
        assert_eq!(resolve_backend_mode(PATHFIND_BACKEND_AUTO), "cpu");
    }

    #[test]
    fn backend_dispatch_single_matches_cpu_path() {
        let walkable = vec![1_u8; 16];
        let move_cost = vec![1.0_f32; 16];
        let cpu = dispatch_pathfind_grid_bytes(
            PATHFIND_BACKEND_CPU,
            4,
            4,
            &walkable,
            &move_cost,
            0,
            0,
            3,
            3,
            200,
        )
        .expect("cpu dispatch should succeed");

        let auto = dispatch_pathfind_grid_bytes(
            PATHFIND_BACKEND_AUTO,
            4,
            4,
            &walkable,
            &move_cost,
            0,
            0,
            3,
            3,
            200,
        )
        .expect("auto dispatch should succeed");

        let gpu = dispatch_pathfind_grid_bytes(
            PATHFIND_BACKEND_GPU,
            4,
            4,
            &walkable,
            &move_cost,
            0,
            0,
            3,
            3,
            200,
        )
        .expect("gpu dispatch should succeed");

        assert_eq!(auto, cpu);
        assert_eq!(gpu, cpu);
    }

    #[test]
    fn backend_dispatch_batch_modes_match_cpu_path() {
        let walkable = vec![1_u8; 36];
        let move_cost = vec![1.0_f32; 36];
        let from_vec2 = vec![
            Vector2::new(0.0, 0.0),
            Vector2::new(5.0, 0.0),
            Vector2::new(0.0, 5.0),
        ];
        let to_vec2 = vec![
            Vector2::new(5.0, 5.0),
            Vector2::new(0.0, 5.0),
            Vector2::new(5.0, 0.0),
        ];
        let from_xy = vec![0, 0, 5, 0, 0, 5];
        let to_xy = vec![5, 5, 0, 5, 5, 0];

        let cpu_vec2 = dispatch_pathfind_grid_batch_vec2_bytes(
            PATHFIND_BACKEND_CPU,
            6,
            6,
            &walkable,
            &move_cost,
            &from_vec2,
            &to_vec2,
            300,
        )
        .expect("cpu vec2 dispatch should succeed");
        let auto_vec2 = dispatch_pathfind_grid_batch_vec2_bytes(
            PATHFIND_BACKEND_AUTO,
            6,
            6,
            &walkable,
            &move_cost,
            &from_vec2,
            &to_vec2,
            300,
        )
        .expect("auto vec2 dispatch should succeed");
        let gpu_vec2 = dispatch_pathfind_grid_batch_vec2_bytes(
            PATHFIND_BACKEND_GPU,
            6,
            6,
            &walkable,
            &move_cost,
            &from_vec2,
            &to_vec2,
            300,
        )
        .expect("gpu vec2 dispatch should succeed");

        let cpu_xy = dispatch_pathfind_grid_batch_xy_bytes(
            PATHFIND_BACKEND_CPU,
            6,
            6,
            &walkable,
            &move_cost,
            &from_xy,
            &to_xy,
            300,
        )
        .expect("cpu xy dispatch should succeed");
        let auto_xy = dispatch_pathfind_grid_batch_xy_bytes(
            PATHFIND_BACKEND_AUTO,
            6,
            6,
            &walkable,
            &move_cost,
            &from_xy,
            &to_xy,
            300,
        )
        .expect("auto xy dispatch should succeed");
        let gpu_xy = dispatch_pathfind_grid_batch_xy_bytes(
            PATHFIND_BACKEND_GPU,
            6,
            6,
            &walkable,
            &move_cost,
            &from_xy,
            &to_xy,
            300,
        )
        .expect("gpu xy dispatch should succeed");

        assert_eq!(auto_vec2, cpu_vec2);
        assert_eq!(gpu_vec2, cpu_vec2);
        assert_eq!(auto_xy, cpu_xy);
        assert_eq!(gpu_xy, cpu_xy);
    }

    #[test]
    fn public_backend_mode_helpers_roundtrip_and_validate() {
        let previous = get_pathfind_backend_mode().to_string();
        assert!(!has_gpu_pathfind_backend());

        assert!(set_pathfind_backend_mode("cpu"));
        assert_eq!(get_pathfind_backend_mode(), "cpu");
        assert_eq!(resolve_pathfind_backend_mode(), "cpu");

        assert!(set_pathfind_backend_mode("auto"));
        assert_eq!(get_pathfind_backend_mode(), "auto");
        assert_eq!(resolve_pathfind_backend_mode(), "cpu");

        assert!(!set_pathfind_backend_mode("invalid-mode"));
        assert!(set_pathfind_backend_mode(&previous));
    }

    #[test]
    fn public_dispatch_counter_helpers_track_dispatch_paths() {
        let previous = get_pathfind_backend_mode().to_string();
        assert!(set_pathfind_backend_mode("cpu"));
        reset_pathfind_backend_dispatch_counts();

        let walkable = vec![1_u8; 16];
        let move_cost = vec![1.0_f32; 16];
        let from = vec![(0, 0), (1, 1)];
        let to = vec![(3, 3), (2, 2)];
        let from_xy = vec![0, 0, 1, 1];
        let to_xy = vec![3, 3, 2, 2];

        let (cpu_before, gpu_before) = pathfind_backend_dispatch_counts();
        let _ = pathfind_grid_batch_dispatch_bytes(4, 4, &walkable, &move_cost, &from, &to, 200)
            .expect("dispatch tuple batch should succeed");
        let _ = pathfind_grid_batch_xy_dispatch_bytes(
            4, 4, &walkable, &move_cost, &from_xy, &to_xy, 200,
        )
        .expect("dispatch xy batch should succeed");
        let (cpu_after, gpu_after) = pathfind_backend_dispatch_counts();

        assert!(cpu_after >= cpu_before + 2);
        assert_eq!(gpu_after, gpu_before);
        assert!(set_pathfind_backend_mode(&previous));
    }

    #[test]
    fn ws2_roundtrip_preserves_snapshot_scalars() {
        let snapshot = EngineSnapshot {
            tick: 42,
            year: 3,
            day_of_year: 12,
            entity_count: 10,
            settlement_count: 2,
            system_count: 7,
            events_dispatched: 99,
        };
        let encoded = encode_ws2_blob(&snapshot).expect("ws2 encode should succeed");
        let decoded = decode_ws2_blob(&encoded).expect("ws2 decode should succeed");
        assert_eq!(decoded.tick, 42);
        assert_eq!(decoded.year, 3);
        assert_eq!(decoded.day_of_year, 12);
        assert_eq!(decoded.entity_count, 10);
        assert_eq!(decoded.settlement_count, 2);
        assert_eq!(decoded.system_count, 7);
        assert_eq!(decoded.events_dispatched, 99);
    }

    #[test]
    fn ws2_decode_rejects_invalid_magic() {
        let mut bytes = vec![0_u8; 16];
        bytes[0] = b'B';
        bytes[1] = b'A';
        bytes[2] = b'D';
        assert!(decode_ws2_blob(&bytes).is_none());
    }

    #[test]
    fn game_event_type_id_maps_new_v2_event_variants() {
        assert_eq!(
            super::runtime_events::game_event_type_id(&GameEvent::EntitySpawned {
                entity_id: EntityId(1),
            }),
            super::runtime_events::EVENT_TYPE_ID_ENTITY_SPAWNED
        );
        assert_eq!(
            super::runtime_events::game_event_type_id(&GameEvent::ResourceGathered {
                entity_id: EntityId(2),
                resource: "food".to_string(),
                amount: 3.0,
            }),
            super::runtime_events::EVENT_TYPE_ID_RESOURCE_GATHERED
        );
        assert_eq!(
            super::runtime_events::game_event_type_id(&GameEvent::SocialEventOccurred {
                event_type: "action_chosen:forage".to_string(),
                participants: vec![EntityId(4), EntityId(7)],
            }),
            super::runtime_events::EVENT_TYPE_ID_SOCIAL_EVENT_OCCURRED
        );
        assert_eq!(
            super::runtime_events::game_event_type_id(&GameEvent::TechDiscovered {
                settlement_id: SettlementId(5),
                tech_id: "agriculture".to_string(),
            }),
            super::runtime_events::EVENT_TYPE_ID_TECH_DISCOVERED
        );
        assert_eq!(
            super::runtime_events::game_event_type_id(&GameEvent::EraAdvanced {
                settlement_id: SettlementId(9),
                new_era: "bronze_age".to_string(),
            }),
            super::runtime_events::EVENT_TYPE_ID_ERA_ADVANCED
        );
    }

    #[test]
    fn action_target_resource_key_maps_foraging_actions() {
        assert_eq!(
            super::action_target_resource_key(ActionType::Forage),
            "food"
        );
        assert_eq!(
            super::action_target_resource_key(ActionType::GatherHerbs),
            "food"
        );
        assert_eq!(
            super::action_target_resource_key(ActionType::GatherWood),
            "wood"
        );
        assert_eq!(
            super::action_target_resource_key(ActionType::GatherStone),
            "stone"
        );
        assert_eq!(
            super::action_target_resource_key(ActionType::Build),
            "building"
        );
        assert_eq!(super::action_target_resource_key(ActionType::Idle), "");
    }

    #[test]
    fn narrative_cache_any_text_detects_populated_fields() {
        let mut cache = NarrativeCache::default();
        assert!(!super::narrative_cache_has_any_text(&cache));
        cache.last_inner_monologue = Some("생각".to_string());
        assert!(super::narrative_cache_has_any_text(&cache));
    }

    #[test]
    fn narrative_cache_complete_and_fresh_requires_all_sections() {
        let mut cache = NarrativeCache {
            personality_desc: Some("성격".to_string()),
            last_event_narrative: Some("사건".to_string()),
            last_inner_monologue: Some("내면".to_string()),
            cache_tick: 100,
            cache_ttl_ticks: 60,
        };
        assert!(super::narrative_cache_is_complete_and_fresh(&cache, 120));
        cache.last_event_narrative = None;
        assert!(!super::narrative_cache_is_complete_and_fresh(&cache, 120));
        cache.last_event_narrative = Some("사건".to_string());
        assert!(!super::narrative_cache_is_complete_and_fresh(&cache, 200));
    }

    #[test]
    fn click_narrative_request_sequences_personality_then_event_then_inner() {
        let current_tick = 500_u64;
        let recent_event = super::NarrativeRecentEvent {
            event_type: SimEventType::SocialConflict,
            cause: "argument".to_string(),
            target_name: Some("Rin".to_string()),
        };

        let personality_plan =
            super::plan_narrative_request(None, current_tick, Some(&recent_event))
                .expect("personality request should be planned first");
        assert_eq!(personality_plan.variant, LlmPromptVariant::Personality);
        assert!(personality_plan.recent_event_type.is_none());

        let cache_after_personality = NarrativeCache {
            personality_desc: Some("성격".to_string()),
            cache_tick: current_tick,
            cache_ttl_ticks: 3600,
            ..NarrativeCache::default()
        };
        let event_plan = super::plan_narrative_request(
            Some(&cache_after_personality),
            current_tick,
            Some(&recent_event),
        )
        .expect("event narrative should be planned second");
        assert_eq!(event_plan.variant, LlmPromptVariant::Narrative);
        assert_eq!(
            event_plan.recent_event_type.as_deref(),
            Some("social_conflict")
        );

        let cache_after_event = NarrativeCache {
            personality_desc: Some("성격".to_string()),
            last_event_narrative: Some("사건".to_string()),
            cache_tick: current_tick,
            cache_ttl_ticks: 3600,
            ..NarrativeCache::default()
        };
        let inner_plan = super::plan_narrative_request(
            Some(&cache_after_event),
            current_tick,
            Some(&recent_event),
        )
        .expect("inner monologue should be planned last");
        assert_eq!(inner_plan.variant, LlmPromptVariant::Narrative);
        assert!(inner_plan.recent_event_type.is_none());
    }

    #[test]
    fn click_pending_is_not_preempted_before_timeout() {
        let pending = LlmPending {
            request_id: 7,
            request_type: LlmRequestType::Layer4Narrative,
            submitted_tick: 100,
            timeout_ticks: 100,
        };
        assert!(!super::pending_request_should_be_preempted(&pending, 150));
        assert!(!super::pending_request_should_be_preempted(&pending, 200));
    }

    #[test]
    fn chronicle_entry_lite_legacy_summary_keeps_layered_fields() {
        let entry = ChronicleEntryLite {
            entry_id: ChronicleEntryId(4),
            start_tick: 20,
            end_tick: 21,
            event_family: "chronicle.test.food".to_string(),
            event_type: ChronicleEventType::InfluenceAttraction,
            cause: ChronicleEventCause::Food,
            headline: ChronicleHeadline {
                locale_key: "CHRONICLE_HEADLINE_TEST".to_string(),
                params: BTreeMap::new(),
            },
            capsule: ChronicleCapsule {
                locale_key: "CHRONICLE_CAPSULE_TEST".to_string(),
                params: BTreeMap::new(),
            },
            dossier_stub: ChronicleDossierStub {
                locale_key: "CHRONICLE_DOSSIER_STUB_TEST".to_string(),
                params: BTreeMap::new(),
                detail_tags: vec!["cause".to_string()],
            },
            entity_ref: ChronicleSubjectRefLite {
                entity_id: Some(EntityId(11)),
                display_name: Some("Aria".to_string()),
                ref_state: ChronicleEntityRefState::Alive,
            },
            location_ref: ChronicleLocationRefLite {
                tile_x: 1,
                tile_y: 2,
                region_label: None,
            },
            significance: 7.0,
            significance_category: ChronicleSignificanceCategory::Major,
            significance_meta: ChronicleSignificanceMeta {
                base_score: 6.0,
                cause_bonus: 1.0,
                group_bonus: 0.0,
                repeat_penalty: 0.0,
                final_score: 7.0,
                reason_tags: vec!["cause:food".to_string()],
            },
            queue_bucket: ChronicleQueueBucket::Visible,
            status: ChronicleEntryStatus::Published,
            surfaced_tick: Some(21),
            displacement_reason: None,
            queue_transitions: Vec::new(),
            thread_id: None,
        };
        let summary = entry.to_legacy_summary();
        assert_eq!(summary.title, "CHRONICLE_HEADLINE_TEST");
        assert_eq!(summary.description, "CHRONICLE_CAPSULE_TEST");
        assert_eq!(summary.category, ChronicleSignificanceCategory::Major);
    }

    #[test]
    fn chronicle_snapshot_revision_from_arg_accepts_only_non_negative_values() {
        assert_eq!(
            super::chronicle_snapshot_revision_from_arg(7),
            Some(sim_engine::ChronicleSnapshotRevision(7))
        );
        assert_eq!(super::chronicle_snapshot_revision_from_arg(-1), None);
    }
}
