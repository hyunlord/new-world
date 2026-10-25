//! Pre-award craft inputs and complete persistent-tally reconciliation, including dead agents.
use super::manifest::Config;
use super::types::{CraftEvent, GatingNode, InvalidReason, MaterialTerm, TallyRow};
use sim_core::colony::{content, knowledge, Job, JobKind, JobStage, Skills};
use sim_core::components::{Agent, Temperament};
use sim_engine::SimEngine;
use std::collections::{BTreeMap, BTreeSet};

pub type Tallies = BTreeMap<(u64, u32), (u64, u64)>;

pub fn rows(tallies: &Tallies) -> Vec<TallyRow> {
    tallies
        .iter()
        .map(
            |(&(agent_id, recipe_code), &(completed_count, quality_sum))| TallyRow {
                agent_id,
                recipe_code,
                completed_count,
                quality_sum,
            },
        )
        .collect()
}

pub fn capture(engine: &SimEngine, config: &Config) -> BTreeMap<u64, CraftEvent> {
    let res = &engine.resources;
    let mut out = BTreeMap::new();
    for (entity, (agent, job)) in engine.world.query::<(&Agent, &Job)>().iter() {
        let (JobKind::Craft { workshop, bill, at }, JobStage::Working { ticks_left }) =
            (job.kind, job.stage)
        else {
            continue;
        };
        let Some(recipe) = res
            .colony
            .workshops
            .get(&workshop)
            .and_then(|w| w.bill(bill))
            .and_then(|b| b.recipe.recipe())
        else {
            continue;
        };
        let skill = engine.world.get::<&Skills>(entity).ok();
        let raw_skill = skill
            .as_ref()
            .map(|s| s.get(recipe.skill))
            .unwrap_or_default();
        let temperament = engine.world.get::<&Temperament>(entity).ok();
        let known = engine.world.get::<&knowledge::Knowledge>(entity).ok();
        let gate = knowledge::gates().recipe(recipe.id);
        let gating_nodes = knowledge::ids_of(gate)
            .map(|node_id| {
                let entry = known.as_ref().and_then(|k| k.get(node_id));
                GatingNode {
                    node_id,
                    entry_present: entry.is_some(),
                    proficiency: entry.map_or(0, |e| e.proficiency),
                    last_used: entry.map(|e| e.last_used),
                    admission_threshold: knowledge::defs().threshold(node_id),
                }
            })
            .collect();
        let stats = job.material.and_then(|m| res.colony.materials.stats(m));
        let material_term = match recipe.material_term {
            content::MaterialTerm::None => MaterialTerm::None,
            content::MaterialTerm::Hardness { offset, divisor } => {
                MaterialTerm::Hardness { offset, divisor }
            }
        };
        let (count, sum) = res
            .colony
            .tallies
            .craft_quality
            .get(&(agent.id, recipe.id.code()))
            .copied()
            .unwrap_or((0, 0));
        out.insert(
            agent.id,
            CraftEvent {
                seed: res.world_seed,
                agent_id: agent.id,
                executed_tick: engine.current_tick(),
                workshop_id: workshop,
                bill_id: bill,
                recipe_code: recipe.id.code(),
                recipe_identity: recipe.identity.id.clone(),
                catalog_revision: config.manifest.catalog_revision.clone(),
                output_item_code: recipe.output.0.code(),
                output_count: recipe.output.1,
                at,
                job_stage: "working",
                ticks_left,
                skill_code: recipe.skill.code(),
                skill_present: skill.is_some(),
                xp_before: raw_skill.xp,
                level_before: raw_skill.level,
                material_id: job.material.map(|m| m.raw()),
                material_hardness: stats.map(|s| s.hardness),
                hardness_percent: stats.map(|s| s.hardness_percent()),
                temperament_present: temperament.is_some(),
                persistence: temperament.as_ref().map(|t| t.p),
                knowledge_enabled: sim_systems::runtime::knowledge::rule(res).is_some(),
                gate_mask: gate,
                gating_nodes,
                recipe_quality_add: recipe.quality_add,
                material_term,
                tally_before_count: count,
                tally_after_count: count,
                tally_before_quality_sum: sum,
                tally_after_quality_sum: sum,
                completion_count_delta: 0,
                actual_quality: 0,
                alive_after_tick: true,
                arithmetic_manifest_sha256: config.manifest.arithmetic_manifest_sha256.clone(),
            },
        );
    }
    out
}

pub fn reconcile(
    before: &Tallies,
    after: &Tallies,
    snapshots: &BTreeMap<u64, CraftEvent>,
    alive: &BTreeSet<u64>,
    tick: u64,
) -> (Vec<CraftEvent>, Vec<InvalidReason>) {
    let keys: BTreeSet<_> = before.keys().chain(after.keys()).copied().collect();
    let mut events = Vec::new();
    let mut errors = Vec::new();
    for key @ (agent, recipe) in keys {
        let previous = before.get(&key).copied().unwrap_or((0, 0));
        let Some(&current) = after.get(&key) else {
            errors.push(invalid(
                "TALLY_REMOVED",
                tick,
                agent,
                format!("recipe {recipe}"),
            ));
            continue;
        };
        let (Some(count), Some(sum)) = (
            current.0.checked_sub(previous.0),
            current.1.checked_sub(previous.1),
        ) else {
            errors.push(invalid(
                "TALLY_REGRESSED",
                tick,
                agent,
                format!("{previous:?} -> {current:?}"),
            ));
            continue;
        };
        if count == 0 && sum == 0 {
            continue;
        }
        if count != 1 || sum > 100 {
            errors.push(invalid(
                "TALLY_DELTA",
                tick,
                agent,
                format!("recipe {recipe}: count={count} sum={sum}"),
            ));
            continue;
        }
        // Non-primary completions still participate in full-map monotonicity checks.
        if recipe != 0 {
            continue;
        }
        let Some(snapshot) = snapshots.get(&agent) else {
            errors.push(invalid(
                "MISSING_CRAFT_SNAPSHOT",
                tick,
                agent,
                format!("recipe {recipe}"),
            ));
            continue;
        };
        if snapshot.recipe_code != recipe
            || snapshot.recipe_identity != "knap_blade"
            || snapshot.output_count != 1
            || snapshot.executed_tick != tick
            || snapshot.ticks_left > 1
            || snapshot.tally_before_count != previous.0
            || snapshot.tally_before_quality_sum != previous.1
        {
            errors.push(invalid(
                "CRAFT_INPUT_MISMATCH",
                tick,
                agent,
                format!("recipe {recipe}"),
            ));
            continue;
        }
        let mut event = snapshot.clone();
        event.tally_after_count = current.0;
        event.tally_after_quality_sum = current.1;
        event.completion_count_delta = count;
        event.actual_quality = sum as u8; // sum <=100 checked above.
        event.alive_after_tick = alive.contains(&agent);
        events.push(event);
    }
    (events, errors)
}

pub fn invalid(code: &str, tick: u64, agent: u64, detail: String) -> InvalidReason {
    InvalidReason {
        code: code.into(),
        executed_tick: Some(tick),
        agent_id: Some(agent),
        detail,
    }
}
