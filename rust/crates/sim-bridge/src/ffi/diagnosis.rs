//! Bridge reads for the M-READ diagnosis framework, why-not records,
//! requests and the current-task plaque. All reads are presentation-only
//! and never modify simulation state.

use godot::prelude::*;
use sim_engine::SimEngine;

// ── Settlement diagnosis (cached) ──────────────────────────────────────

/// Collect the cached settlement diagnosis as parallel arrays.
pub fn collect_settlement_diagnosis(engine: &SimEngine) -> VarDictionary {
    let cache = &engine.resources.diagnosis_cache;
    let mut dict = VarDictionary::new();
    dict.set("tick", cache.tick as i64);

    let mut check_ids = PackedStringArray::new();
    let mut text_keys = PackedStringArray::new();
    let mut severities = PackedInt32Array::new();
    let mut tags_arr = VarArray::new();
    let mut chains_arr = VarArray::new();
    let mut params_arr = VarArray::new();

    // Use settlement 0 (the colony's single settlement).
    if let Some(entries) = cache.results.get(&0) {
        for entry in entries {
            check_ids.push(&GString::from(&entry.check_id));
            text_keys.push(&GString::from(&entry.text_key));
            severities.push(i32::from(entry.severity));

            let mut tag_list = VarArray::new();
            for tag in &entry.tags {
                tag_list.push(&Variant::from(GString::from(tag.as_str())));
            }
            tags_arr.push(&Variant::from(tag_list));

            let mut param_dict = VarDictionary::new();
            for (k, v) in &entry.params {
                param_dict.set(GString::from(k.as_str()), GString::from(v.as_str()));
            }
            params_arr.push(&Variant::from(param_dict));

            let mut chain_list = VarArray::new();
            for link in &entry.chain {
                let mut link_dict = VarDictionary::new();
                link_dict.set("subject_kind", i32::from(link.subject_kind));
                link_dict.set("check_id", GString::from(&link.check_id));
                link_dict.set("text_key", GString::from(&link.text_key));
                link_dict.set("severity", i32::from(link.severity));
                let mut lp = VarDictionary::new();
                for (k, v) in &link.params {
                    lp.set(GString::from(k.as_str()), GString::from(v.as_str()));
                }
                link_dict.set("params", lp);
                chain_list.push(&Variant::from(link_dict));
            }
            chains_arr.push(&Variant::from(chain_list));
        }
    }
    dict.set("check_ids", check_ids);
    dict.set("text_keys", text_keys);
    dict.set("severities", severities);
    dict.set("tags", tags_arr);
    dict.set("params", params_arr);
    dict.set("chains", chains_arr);
    dict
}

// ── Why-not records ────────────────────────────────────────────────────

/// Collect the latest why-not records for a person.
pub fn collect_why_not(engine: &SimEngine, entity_bits: u64) -> VarDictionary {
    let mut dict = VarDictionary::new();
    let entity = match hecs::Entity::from_bits(entity_bits) {
        Some(e) => e,
        None => return dict,
    };
    let agent = match engine.world.get::<&sim_core::components::Agent>(entity) {
        Ok(a) => a,
        Err(_) => return dict,
    };
    let wn = match engine.resources.mind.why_not.get(&agent.id) {
        Some(w) => w,
        None => return dict,
    };

    let mut nodes = PackedStringArray::new();
    let mut bands = PackedStringArray::new();
    let mut fact_names = PackedStringArray::new();
    let mut ops = PackedStringArray::new();
    let mut thresholds = PackedInt64Array::new();
    let mut actuals = PackedInt64Array::new();
    let mut ticks = PackedInt64Array::new();

    // Look up node names from the mind dictionary.
    let dictionary = engine
        .resources
        .mind
        .history
        .as_ref()
        .map(|h| h.dictionary());

    for entry in wn.latest().iter().take(2) {
        let node_name = dictionary
            .and_then(|d| d.node_label(entry.node.index()))
            .map(|l| l.name.to_owned())
            .unwrap_or_else(|| format!("node_{}", entry.node.index()));
        let band_name =
            sim_core::colony::mind_records::MindRecordDictionary::band_name(entry.band.index())
                .unwrap_or("unknown");

        nodes.push(&GString::from(&node_name));
        bands.push(&GString::from(band_name));
        fact_names.push(&GString::from(&entry.fact_name));
        ops.push(&GString::from(&entry.op));
        thresholds.push(entry.threshold);
        actuals.push(entry.actual);
        ticks.push(entry.tick as i64);
    }

    dict.set("nodes", nodes);
    dict.set("bands", bands);
    dict.set("fact_names", fact_names);
    dict.set("ops", ops);
    dict.set("thresholds", thresholds);
    dict.set("actuals", actuals);
    dict.set("ticks", ticks);
    dict.set("byte_size", wn.byte_size() as i64);
    dict
}

// ── Requests ───────────────────────────────────────────────────────────

/// Collect active requests for the settlement.
pub fn collect_requests(engine: &SimEngine) -> VarDictionary {
    let cache = &engine.resources.requests_cache;
    let mut dict = VarDictionary::new();
    dict.set("tick", cache.tick as i64);

    let mut rule_ids = PackedStringArray::new();
    let mut text_keys = PackedStringArray::new();
    let mut action_keys = PackedStringArray::new();
    let mut priorities = PackedInt32Array::new();
    let mut counts = PackedInt32Array::new();
    let mut voices = PackedInt64Array::new();
    let mut voice_names = PackedStringArray::new();
    let mut steward_active = VarArray::new();

    if let Some(reqs) = cache.results.get(&0) {
        for req in reqs {
            rule_ids.push(&GString::from(&req.rule_id));
            text_keys.push(&GString::from(&req.text_key));
            action_keys.push(&GString::from(&req.action_key));
            priorities.push(req.priority as i32);
            counts.push(req.count as i32);
            voices.push(req.voice as i64);
            voice_names.push(&GString::from(&req.voice_name));
            steward_active.push(&Variant::from(req.steward_active));
        }
    }
    dict.set("rule_ids", rule_ids);
    dict.set("text_keys", text_keys);
    dict.set("action_keys", action_keys);
    dict.set("priorities", priorities);
    dict.set("counts", counts);
    dict.set("voices", voices);
    dict.set("voice_names", voice_names);
    dict.set("steward_active", steward_active);
    dict
}

// ── Current task ───────────────────────────────────────────────────────

/// Collect the current task: top request, or the next unfinished goal.
/// Returns a dictionary with the task description and an optional marker tile.
pub fn collect_current_task(engine: &SimEngine) -> VarDictionary {
    let mut dict = VarDictionary::new();

    // Try the top request first.
    if let Some(reqs) = engine.resources.requests_cache.results.get(&0) {
        if let Some(top) = reqs.first() {
            dict.set("source", GString::from("request"));
            dict.set("text_key", GString::from(&top.text_key));
            dict.set("action_key", GString::from(&top.action_key));
            dict.set("voice", top.voice as i64);
            dict.set("voice_name", GString::from(&top.voice_name));
            dict.set("steward_active", top.steward_active);
            // Marker tile: find a valid tile for the suggested action.
            // For now, return the formation tile of the first settlement.
            for (_, settlement) in engine
                .world
                .query::<&sim_core::components::Settlement>()
                .iter()
            {
                let (fx, fy) = settlement.formation_tile;
                dict.set("marker_x", fx as i32);
                dict.set("marker_y", fy as i32);
                dict.set("has_marker", true);
                break;
            }
            return dict;
        }
    }

    // Fall back to the next unfinished goal (from goals.json — not in Rust).
    dict.set("source", GString::from("none"));
    dict.set("has_marker", false);
    dict
}
