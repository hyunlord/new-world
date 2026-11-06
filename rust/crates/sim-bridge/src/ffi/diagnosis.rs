//! Bridge reads for the M-READ diagnosis framework, why-not records,
//! requests and the current-task plaque. All reads are presentation-only
//! and never modify simulation state.

use godot::prelude::*;
use sim_core::colony::ZoneKind;
use sim_engine::SimEngine;

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
    let weaving = collect_weaving_diagnosis(engine);
    dict.set("first_woven_clothing", weaving.first_clothing);
    dict.set("cloth_short", weaving.cloth_short);
    if weaving.cloth_short {
        check_ids.push("weaving_clothes");
        text_keys.push("S2_DIAG_CLOTHES");
        severities.push(1);
        let mut tags = VarArray::new();
        tags.push(&Variant::from("textile"));
        tags_arr.push(&Variant::from(tags));
        params_arr.push(&Variant::from(VarDictionary::new()));
        let mut chain = VarArray::new();
        for (key, parameter, value) in weaving.chain {
            let mut link = VarDictionary::new();
            link.set("subject_kind", 0);
            link.set("check_id", "weaving_supply");
            link.set("text_key", key);
            link.set("severity", 1);
            let mut params = VarDictionary::new();
            if !parameter.is_empty() {
                params.set(parameter, GString::from(value.as_str()));
            }
            link.set("params", params);
            chain.push(&Variant::from(link));
        }
        chains_arr.push(&Variant::from(chain));
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
    let mut name_keys = PackedStringArray::new();
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

        let node_name_key = dictionary
            .and_then(|d| d.node_label(entry.node.index()))
            .map(|l| l.name_key.to_owned())
            .unwrap_or_default();
        nodes.push(&GString::from(&node_name));
        name_keys.push(&GString::from(&node_name_key));
        bands.push(&GString::from(band_name));
        fact_names.push(&GString::from(fact_name));
        ops.push(&GString::from(entry.op_str()));
        thresholds.push(entry.threshold);
        actuals.push(entry.actual);
        ticks.push(entry.tick as i64);
    }

    dict.set("nodes", nodes);
    dict.set("name_keys", name_keys);
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
            voice_names.push(&GString::from("")); // GDScript fills from voice id
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
            dict.set("voice_name", GString::from("")); // GDScript fills from voice id
            dict.set("steward_active", top.steward_active);
            // The rule's match tags, so the plaque can pick the settlement check behind the request.
            let mut match_tags = PackedStringArray::new();
            if let Some(rule) = sim_core::colony::requests::requests_data()
                .and_then(|data| data.rules.iter().find(|rule| rule.id == top.rule_id))
            {
                for tag in &rule.match_tags {
                    match_tags.push(&GString::from(tag.as_str()));
                }
            }
            dict.set("match_tags", match_tags);
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

/// Read-only clothing progress and the first missing physical input chain.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct WeavingDiagnosis {
    /// A textile garment has actually been crafted, not merely researched.
    pub first_clothing: bool,
    /// Adults still lack clothing after textile work has begun.
    pub cloth_short: bool,
    /// Localized cause key, localized parameter name, and content name key.
    pub chain: Vec<(&'static str, &'static str, String)>,
}

/// Reuse authored recipe dependencies without creating orders or cached sim state.
pub fn collect_weaving_diagnosis(engine: &SimEngine) -> WeavingDiagnosis {
    use sim_core::colony::people::Apparel;
    use sim_core::colony::{Carry, ItemKind, Tool};
    use sim_core::components::{Agent, Child};
    use std::collections::BTreeMap;
    let res = &engine.resources;
    if res.rules.colony.is_none() {
        return WeavingDiagnosis::default();
    }
    let textile = |kind: ItemKind| kind.definition().tags.iter().any(|tag| tag == "textile");
    let first_clothing =
        res.colony.item_ledger.crafted.iter().any(|(kind, count)| {
            *count > 0 && textile(*kind) && kind.definition().apparel.is_some()
        });
    let started = res
        .colony
        .item_ledger
        .crafted
        .iter()
        .any(|(kind, count)| *count > 0 && textile(*kind))
        || !res.processing.pending.is_empty();
    let unclothed = engine
        .world
        .query::<(&Agent, Option<&Child>, Option<&Apparel>)>()
        .iter()
        .any(|(_, (_, child, apparel))| child.is_none() && apparel.is_none());
    let mut out = WeavingDiagnosis {
        first_clothing,
        cloth_short: started && unclothed,
        chain: Vec::new(),
    };
    if !out.cloth_short {
        return out;
    }
    let mut counts: BTreeMap<ItemKind, u32> = BTreeMap::new();
    for stack in res.colony.ground_items.values().chain(
        res.colony
            .workshops
            .values()
            .flat_map(|shop| shop.shelf.iter()),
    ) {
        *counts.entry(stack.kind).or_default() += u32::from(stack.count);
    }
    for (_, carry) in engine.world.query::<&Carry>().iter() {
        *counts.entry(carry.kind).or_default() += u32::from(carry.count);
    }
    for (_, tool) in engine.world.query::<&Tool>().iter() {
        *counts.entry(tool.item.kind).or_default() += 1;
    }
    if counts
        .iter()
        .any(|(kind, count)| *count > 0 && kind.definition().apparel.is_some())
    {
        out.chain.push((
            if res.colony.dress_off {
                "S2_DIAG_DRESS_PAUSED"
            } else if res.colony.ground_items.iter().any(|(tile, stack)| {
                stack.count > 0
                    && stack.kind.definition().apparel.is_some()
                    && res.colony.zone_of_tile.contains_key(tile)
                    && !res
                        .colony
                        .reservations
                        .contains_key(&sim_core::colony::ReserveKey::Stack(*tile))
            }) {
                "S2_DIAG_DRESS_READY"
            } else {
                "S2_DIAG_DRESS_STORAGE"
            },
            "",
            String::new(),
        ));
        return out;
    }
    let Some(mut wanted) =
        ItemKind::all().find(|kind| textile(*kind) && kind.definition().apparel.is_some())
    else {
        return out;
    };
    let catalog = sim_core::colony::content::installed_or_bundled();
    let mut seen = std::collections::BTreeSet::new();
    while seen.insert(wanted) && seen.len() <= 6 {
        if let Some(batch) = res
            .processing
            .pending
            .iter()
            .find(|batch| batch.output.kind == wanted)
        {
            let wet = res
                .colony
                .workshops
                .get(&batch.workshop)
                .is_some_and(|shop| {
                    !batch.requires_river
                        || sim_systems::runtime::jobs::processing::wet(res, shop.top_left)
                });
            let key = if batch.remaining_ticks == 0 {
                "S2_DIAG_OUTPUT"
            } else if !wet {
                "S2_DIAG_DRY"
            } else {
                "S2_DIAG_RETTING"
            };
            out.chain.push((key, "", String::new()));
            break;
        }
        let Some(recipe) = catalog
            .recipe_definitions()
            .iter()
            .find(|recipe| recipe.output.0 == wanted)
        else {
            out.chain.push((
                "S2_DIAG_GATHER",
                "item_key",
                wanted.definition().identity.name_key.clone(),
            ));
            break;
        };
        if let Some((missing, _)) = recipe
            .inputs
            .iter()
            .find(|(kind, need)| counts.get(kind).copied().unwrap_or(0) < u32::from(*need))
        {
            out.chain.push((
                "S2_DIAG_INPUT",
                "item_key",
                missing.definition().identity.name_key.clone(),
            ));
            wanted = *missing;
            continue;
        }
        let Some(shop) = res
            .colony
            .workshops
            .values()
            .find(|shop| recipe.supports_workshop(shop.kind))
        else {
            out.chain.push((
                "S2_DIAG_WORKSHOP",
                "workshop_key",
                recipe.workshop.definition().identity.name_key.clone(),
            ));
            break;
        };
        if recipe
            .process
            .as_ref()
            .is_some_and(|process| process.requires_river)
            && !sim_systems::runtime::jobs::processing::wet(res, shop.top_left)
        {
            out.chain.push(("S2_DIAG_DRY", "", String::new()));
        } else if let Some(tool) = recipe
            .tool
            .as_ref()
            .filter(|_| recipe.shelf_tool(shop).is_none())
        {
            out.chain.push((
                "S2_DIAG_TOOL",
                "tool_key",
                tool.preferred_item.definition().identity.name_key.clone(),
            ));
        } else if sim_systems::runtime::jobs::processing::pending_count(res, shop.id) > 0 {
            let blocked = res
                .processing
                .pending
                .iter()
                .any(|batch| batch.workshop == shop.id && batch.remaining_ticks == 0);
            out.chain.push((
                if blocked {
                    "S2_DIAG_OUTPUT"
                } else {
                    "S2_DIAG_RETTING"
                },
                "",
                String::new(),
            ));
        } else {
            out.chain.push((
                "S2_DIAG_BILL",
                "workshop_key",
                shop.kind.definition().identity.name_key.clone(),
            ));
        }
        break;
    }
    out
}

#[cfg(test)]
mod weaving_tests {
    use super::*;
    use sim_core::colony::{ItemKind, ItemSpec, ItemStack};

    fn engine() -> SimEngine {
        let mut engine = SimEngine::new(4, 4, sim_core::MaterialRegistry::new());
        engine.resources.rules = sim_core::rules::WorldRules::colony(0);
        engine.spawn_agent(1, 1);
        engine
    }

    #[test]
    fn chapter_requires_real_crafted_garment_and_wrap_counts() {
        let mut engine = engine();
        let wrap = ItemKind::from_name("linen_wrap").expect("authored wrap");
        engine
            .resources
            .colony
            .ground_items
            .insert((2, 2), ItemStack::of(ItemSpec::raw(wrap, None), 1));
        assert!(!collect_weaving_diagnosis(&engine).first_clothing);
        engine.resources.colony.item_ledger.crafted.insert(wrap, 1);
        let before = engine.resources.colony.item_ledger.clone();
        let view = collect_weaving_diagnosis(&engine);
        assert!(view.first_clothing && view.cloth_short);
        assert_eq!(view.chain[0].0, "S2_DIAG_DRESS_STORAGE");
        assert_eq!(engine.resources.colony.item_ledger, before);
        engine.resources.colony.dress_off = true;
        assert_eq!(
            collect_weaving_diagnosis(&engine).chain[0].0,
            "S2_DIAG_DRESS_PAUSED"
        );
    }

    #[test]
    fn dressing_diagnosis_requires_unreserved_stockpile_clothing_without_mutation() {
        let mut engine = engine();
        let wrap = ItemKind::from_name("linen_wrap").expect("authored wrap");
        let tile = (2, 2);
        engine.resources.colony.item_ledger.crafted.insert(wrap, 1);
        engine
            .resources
            .colony
            .ground_items
            .insert(tile, ItemStack::of(ItemSpec::raw(wrap, None), 1));
        for (zoned, reserved, paused, expected) in [
            (false, false, false, "S2_DIAG_DRESS_STORAGE"),
            (true, false, false, "S2_DIAG_DRESS_READY"),
            (true, true, false, "S2_DIAG_DRESS_STORAGE"),
            (true, true, true, "S2_DIAG_DRESS_PAUSED"),
        ] {
            if zoned {
                engine.resources.colony.zone_of_tile.insert(tile, 0);
            }
            if reserved {
                engine
                    .resources
                    .colony
                    .reservations
                    .insert(sim_core::colony::ReserveKey::Stack(tile), 0);
            }
            engine.resources.colony.dress_off = paused;
            let before = ron::to_string(&engine.resources.colony).expect("colony state");
            assert_eq!(collect_weaving_diagnosis(&engine).chain[0].0, expected);
            assert_eq!(ron::to_string(&engine.resources.colony).unwrap(), before);
            assert!(engine.resources.colony.command_log.is_empty());
        }
    }

    #[test]
    fn missing_inputs_follow_the_authored_physical_chain() {
        let mut engine = engine();
        let cloth = ItemKind::from_name("linen_cloth").expect("authored cloth");
        engine.resources.colony.item_ledger.crafted.insert(cloth, 1);
        let view = collect_weaving_diagnosis(&engine);
        assert!(!view.first_clothing);
        assert!(view.cloth_short);
        let keys: Vec<&str> = view.chain.iter().map(|entry| entry.2.as_str()).collect();
        assert_eq!(
            keys,
            [
                "ITEM_LINEN_CLOTH",
                "ITEM_LINEN_THREAD",
                "ITEM_FLAX_FIBRE",
                "ITEM_FLAX_STALKS",
                "ITEM_FLAX_STALKS"
            ]
        );
    }

    #[test]
    fn baseline_has_no_clothing_card_or_chapter_completion() {
        let engine = SimEngine::new(4, 4, sim_core::MaterialRegistry::new());
        assert_eq!(
            collect_weaving_diagnosis(&engine),
            WeavingDiagnosis::default()
        );
    }
}
