//! Bridge reads for the M-READ diagnosis framework, why-not records,
//! requests and the current-task plaque. All reads are presentation-only
//! and never modify simulation state.

use godot::prelude::*;
use sim_core::colony::ZoneKind;
use sim_core::components::AgentId;
use sim_engine::SimEngine;

// ── Deterministic syllable name (matches PersonNames.gd) ──────────────

const EN: &[&str] = &[
    "a", "i", "o", "e", "ka", "ki", "ku", "ko", "ta", "ti", "tu", "to", "na", "ni", "nu", "no",
    "ma", "mi", "mu", "mo", "ra", "ri", "ru", "ro", "sa", "si", "su", "so", "ha", "hi", "hu", "ho",
    "ya", "yu", "yo", "wa", "ba", "bi", "bo", "bu", "da", "do", "ga", "go", "de", "ge",
];

fn mix(mut x: u32) -> u32 {
    x ^= 0x9E3779B9;
    x = (x ^ (x >> 16)).wrapping_mul(0x45D9F3B);
    x = (x ^ (x >> 16)).wrapping_mul(0x45D9F3B);
    x ^ (x >> 16)
}

/// Deterministic presented name for an agent id, matching PersonNames.gd.
pub fn person_name(agent_id: AgentId) -> String {
    let h = mix(mix(agent_id as u32) ^ 0x2545F491);
    let count = if h.is_multiple_of(4) { 2 } else { 3 };
    let mut syllables: Vec<usize> = (0..count)
        .map(|i| ((h >> (5 * i + 2)) as usize) % 46)
        .collect();
    // Blocklist check (simplified — full check in GDScript).
    let blocked = |s: &[usize]| -> bool {
        let joined: String = s.iter().map(|i| EN[*i]).collect();
        ["nig", "kak", "baka"].iter().any(|b| joined.contains(b))
    };
    let mut attempts = 0u32;
    while blocked(&syllables) {
        let last = syllables.len() - 1;
        syllables[last] = (syllables[last] + 1) % 46;
        attempts += 1;
        if attempts.is_multiple_of(46) {
            for i in (0..last).rev() {
                syllables[i] = (syllables[i] + 1) % 46;
                if syllables[i] != 0 {
                    break;
                }
            }
        }
        if attempts > 200 {
            break;
        }
    }
    let joined: String = syllables.iter().map(|i| EN[*i]).collect();
    let mut chars = joined.chars();
    match chars.next() {
        Some(c) => c.to_uppercase().to_string() + chars.as_str(),
        None => String::new(),
    }
}

// ── Settlement diagnosis (cached) ──────────────────────────────────────

/// Collect the cached settlement diagnosis as parallel arrays.
pub fn settlement_diagnosis_to_dict(engine: &SimEngine) -> VarDictionary {
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

// ── Person diagnosis (on demand) ───────────────────────────────────────

/// Evaluate diagnosis checks for a person, on demand.
pub fn person_diagnosis_to_dict(engine: &SimEngine, entity_bits: u64) -> VarDictionary {
    let mut dict = VarDictionary::new();
    let entity = match hecs::Entity::from_bits(entity_bits) {
        Some(e) => e,
        None => return dict,
    };
    if engine
        .world
        .get::<&sim_core::components::Agent>(entity)
        .is_err()
    {
        return dict;
    }
    let data = match sim_core::colony::diagnosis::diagnosis_data() {
        Some(d) => d,
        None => return dict,
    };
    let entries = sim_systems::runtime::diagnosis::evaluate_person(
        &engine.world,
        &engine.resources,
        entity,
        data,
    );

    let mut check_ids = PackedStringArray::new();
    let mut text_keys = PackedStringArray::new();
    let mut severities = PackedInt32Array::new();
    let mut tags_arr = VarArray::new();
    let mut chains_arr = VarArray::new();
    let mut params_arr = VarArray::new();

    for entry in &entries {
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
    dict.set("check_ids", check_ids);
    dict.set("text_keys", text_keys);
    dict.set("severities", severities);
    dict.set("tags", tags_arr);
    dict.set("params", params_arr);
    dict.set("chains", chains_arr);
    dict
}

// ── Subject diagnosis (stockpile / hearth / room) ─────────────────────

/// Evaluate diagnosis checks for a stockpile, hearth or room on demand.
/// `kind` is one of "stockpile", "hearth", "room"; `id` is the zone_id,
/// settlement_id or room_id respectively.
pub fn subject_diagnosis_to_dict(engine: &SimEngine, kind: &str, id: u32) -> VarDictionary {
    let mut dict = VarDictionary::new();
    let data = match sim_core::colony::diagnosis::diagnosis_data() {
        Some(d) => d,
        None => return dict,
    };
    let entries = match kind {
        "stockpile" => sim_systems::runtime::diagnosis::evaluate_stockpile(
            &engine.world,
            &engine.resources,
            id,
            data,
        ),
        "hearth" => sim_systems::runtime::diagnosis::evaluate_hearth(
            &engine.world,
            &engine.resources,
            id,
            data,
        ),
        "room" => sim_systems::runtime::diagnosis::evaluate_room(
            &engine.world,
            &engine.resources,
            id,
            data,
        ),
        _ => return dict,
    };

    let mut check_ids = PackedStringArray::new();
    let mut text_keys = PackedStringArray::new();
    let mut severities = PackedInt32Array::new();
    let mut tags_arr = VarArray::new();
    let mut chains_arr = VarArray::new();
    let mut params_arr = VarArray::new();

    for entry in &entries {
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
pub fn why_not_to_dict(engine: &SimEngine, entity_bits: u64) -> VarDictionary {
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

    // Resolve fact names from the policy's fact list.
    let facts = engine
        .resources
        .mind
        .history
        .as_ref()
        .map(|h| h.dictionary());

    for slot in wn.latest().iter().take(2) {
        let entry = match slot {
            Some(e) => e,
            None => continue,
        };
        let node_name = dictionary
            .and_then(|d| d.node_label(entry.node.index()))
            .map(|l| l.name.to_owned())
            .unwrap_or_else(|| format!("node_{}", entry.node.index()));
        let band_name =
            sim_core::colony::mind_records::MindRecordDictionary::band_name(entry.band.index())
                .unwrap_or("unknown");
        let fact_name = facts
            .and_then(|d| d.fact_name(entry.fact.index()))
            .unwrap_or("?");

        nodes.push(&GString::from(&node_name));
        bands.push(&GString::from(band_name));
        fact_names.push(&GString::from(fact_name));
        ops.push(&GString::from(entry.op_str()));
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
pub fn requests_to_dict(engine: &SimEngine) -> VarDictionary {
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
            voice_names.push(&GString::from(&person_name(req.voice)));
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

// ── Current task with validated marker ─────────────────────────────────

/// Find a valid marker tile for a request's suggested action using the same
/// validity checks the order tools use (§2.4).
fn find_marker_tile(engine: &SimEngine, action_key: &str) -> Option<(u32, u32)> {
    let res = &engine.resources;
    match action_key {
        "REQ_ACTION_STORE_FOOD" | "REQ_ACTION_BUILD_HOUSE" => {
            // Find a tile where a Stockpile can be placed (for food storage)
            // or where a wall can start (for housing).
            let kind_name = if action_key == "REQ_ACTION_BUILD_HOUSE" {
                "LogWall"
            } else {
                "Stockpile"
            };
            // Search near the formation tile of the first settlement.
            let formation = engine
                .world
                .query::<&sim_core::components::Settlement>()
                .iter()
                .map(|(_, s)| s.formation_tile)
                .next()
                .unwrap_or((0, 0));
            let (cx, cy) = formation;
            // Spiral outward from the formation tile to find a valid spot.
            for r in 0..16u32 {
                for dx in 0..=r {
                    for dy in 0..=r {
                        if dx + dy != r {
                            continue;
                        }
                        for (sx, sy) in &[(1i32, 1i32), (1, -1), (-1, 1), (-1, -1)] {
                            let x = cx as i32 + dx as i32 * sx;
                            let y = cy as i32 + dy as i32 * sy;
                            if x >= 0 && y >= 0 && super::colony::can_place(res, kind_name, x, y) {
                                return Some((x as u32, y as u32));
                            }
                        }
                    }
                }
            }
            None
        }
        "REQ_ACTION_CHOP_WOOD" => {
            // Find a tree tile that can be designated for chopping.
            // Trees are in colony.trees; pick the nearest to the formation tile.
            let formation = engine
                .world
                .query::<&sim_core::components::Settlement>()
                .iter()
                .map(|(_, s)| s.formation_tile)
                .next()
                .unwrap_or((0, 0));
            let (cx, cy) = formation;
            res.colony
                .trees
                .keys()
                .filter(|pos| !res.colony.designations.contains_key(pos))
                .min_by_key(|(tx, ty)| {
                    let dx = (*tx as i64) - (cx as i64);
                    let dy = (*ty as i64) - (cy as i64);
                    dx * dx + dy * dy
                })
                .copied()
        }
        _ => {
            // For other actions (water, rest), no specific marker.
            None
        }
    }
}

/// Collect the current task: top request, or the next unfinished goal.
/// Returns a dictionary with the task description and a validated marker tile.
pub fn current_task_to_dict(engine: &SimEngine) -> VarDictionary {
    let mut dict = VarDictionary::new();

    // Try the top request first.
    if let Some(reqs) = engine.resources.requests_cache.results.get(&0) {
        if let Some(top) = reqs.first() {
            dict.set("source", GString::from("request"));
            dict.set("text_key", GString::from(&top.text_key));
            dict.set("action_key", GString::from(&top.action_key));
            dict.set("voice", top.voice as i64);
            dict.set("voice_name", GString::from(&person_name(top.voice)));
            dict.set("steward_active", top.steward_active);
            // Marker tile using the same validity check the order tool uses.
            if let Some((mx, my)) = find_marker_tile(engine, &top.action_key) {
                dict.set("marker_x", mx as i32);
                dict.set("marker_y", my as i32);
                dict.set("has_marker", true);
            } else {
                dict.set("has_marker", false);
            }
            return dict;
        }
    }

    // Fall back to the next unfinished goal.
    // Goals are evaluated on the GDScript side (goals.json); if no request
    // exists, check whether there's a stockpile (the first goal).
    if engine
        .resources
        .colony
        .zones
        .values()
        .any(|z| z.kind == ZoneKind::Stockpile)
    {
        dict.set("source", GString::from("none"));
        dict.set("has_marker", false);
    } else {
        // No stockpile — the first goal is to build one.
        dict.set("source", GString::from("goal"));
        dict.set("text_key", GString::from("DIAG_SETTLEMENT_NO_STOCKPILE"));
        dict.set("action_key", GString::from("REQ_ACTION_STORE_FOOD"));
        if let Some((mx, my)) = find_marker_tile(engine, "REQ_ACTION_STORE_FOOD") {
            dict.set("marker_x", mx as i32);
            dict.set("marker_y", my as i32);
            dict.set("has_marker", true);
        } else {
            dict.set("has_marker", false);
        }
    }
    dict
}
