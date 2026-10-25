//! Real bill admission/payment followed by the registered finish boundary.
use super::{colony_support, dto::*};
use sim_core::colony::knowledge::{self, Knowledge, Source};
use sim_core::colony::materials::FLINT;
use sim_core::colony::{
    content, BillMode, BlueprintKind, Command, ItemKind, ItemSpec, Job, JobKind, JobStage,
    RecipeKind, SkillKind, Skills,
};
use sim_core::components::{Agent, Temperament};
use sim_core::rules::ColonyRule;
use sim_engine::SimEngine;
use sim_systems::runtime::jobs::{cancel, craft, invariants};

fn tally(engine: &SimEngine) -> TallyRow {
    let (completed_count, quality_sum) = engine
        .resources
        .colony
        .tallies
        .craft_quality
        .get(&(1, 0))
        .copied()
        .unwrap_or_default();
    TallyRow {
        agent_id: 1,
        recipe_code: 0,
        completed_count,
        quality_sum,
    }
}

fn ledger(engine: &SimEngine) -> Vec<LedgerRow> {
    [ItemKind::Flint, ItemKind::FlintBlade]
        .into_iter()
        .map(|kind| {
            let t = invariants::ledger_of(&engine.world, &engine.resources, kind);
            LedgerRow {
                item_code: kind.code(),
                created: t.created,
                consumed: t.consumed,
                traded_in: t.traded_in,
                traded_out: t.traded_out,
                ground: t.ground,
                carried: t.carried,
                held: t.held,
                delivered: t.delivered,
                shelved: t.shelved,
            }
        })
        .collect()
}

pub fn sample(
    seed: u64,
    level: u8,
    proficiency: u16,
    arithmetic_hash: &str,
    rows: &mut Vec<CraftRow>,
) {
    let recipe = RecipeKind::KnapBlade.recipe().expect("primary recipe");
    assert_eq!(
        (
            recipe.id.code(),
            recipe.identity.id.as_str(),
            recipe.work_ticks,
            recipe.quality_add
        ),
        (0, "knap_blade", 45, 0)
    );
    assert_eq!(recipe.inputs, vec![(ItemKind::Flint, 1)]);
    assert_eq!(recipe.output, (ItemKind::FlintBlade, 1));
    let content::MaterialTerm::Hardness { offset, divisor } = recipe.material_term else {
        panic!("primary hardness term")
    };
    assert_eq!((offset, divisor), (50, 5));
    let gate_mask = knowledge::gates().recipe(recipe.id);
    assert_eq!(knowledge::id("flint_knapping"), Some(3));
    assert_eq!(gate_mask, 8);
    let mut engine = colony_support::job_world_m2(32, 32);
    // Reserve allocator ID 0 without creating an extra fixture resident.
    assert_eq!(engine.resources.issue_agent_id(), 0);
    engine.resources.world_seed = seed;
    engine.resources.rules.colony = Some(ColonyRule::m5_5());
    let entity = colony_support::worker(&mut engine, 3, 3);
    assert_eq!(engine.world.get::<&Agent>(entity).expect("worker").id, 1);
    let mut know = Knowledge::default();
    for node in knowledge::ids_of(gate_mask) {
        know.set(node, 1000, Source::Seeded, 0);
    }
    engine
        .world
        .insert_one(entity, know)
        .expect("setup knowledge");
    let shop =
        colony_support::build_workshop(&mut engine, BlueprintKind::KnappingSpot, (12, 12), (4, 4));
    let setup_finished_tick = engine.current_tick();
    let mut know = Knowledge::default();
    for node in knowledge::ids_of(gate_mask) {
        know.set(node, proficiency, Source::Seeded, 0);
    }
    let mut skills = Skills::default();
    skills.train(
        SkillKind::Knap,
        sim_core::colony::skills::XP_TABLE[usize::from(level)],
    );
    engine
        .world
        .insert(entity, (know, skills, Temperament::from_axes([500; 4])))
        .expect("reset fixture inputs");
    colony_support::drop_items(
        &mut engine,
        (20, 20),
        ItemSpec::raw(ItemKind::Flint, Some(FLINT)),
        1,
    );
    let ledger_before = ledger(&engine);
    let primary_tally_before_payment = tally(&engine);
    let tally_key_present_before_payment = engine
        .resources
        .colony
        .tallies
        .craft_quality
        .contains_key(&(1, 0));
    let result = sim_systems::commands::submit(
        &mut engine,
        Command::AddBill {
            workshop: shop,
            recipe: recipe.id,
            mode: BillMode::Make(1),
        },
    );
    let add_bill_result = CommandResult {
        accepted: result.accepted,
        count: result.count,
        reason: result.reason.map(|r| r.name().to_owned()),
    };
    assert!(result.accepted, "ordinary AddBill accepted");
    let used = colony_support::run_until(&mut engine, 3000, |e| {
        matches!(
            colony_support::job_of(e, entity),
            Some(Job {
                kind: JobKind::Craft { .. },
                stage: JobStage::Working { .. },
                ..
            })
        )
    });
    assert!(used < 3000, "paid Working reached");
    assert!(
        engine.current_tick() < 27360,
        "payment precedes fixed finish tick"
    );
    let job = colony_support::job_of(&engine, entity).expect("paid job");
    let JobKind::Craft { workshop, bill, at } = job.kind else {
        panic!("craft")
    };
    let JobStage::Working { ticks_left } = job.stage else {
        panic!("working")
    };
    assert_eq!((workshop, bill), (shop, 1));
    assert_eq!(job.material, Some(FLINT));
    assert!(ticks_left > 0);
    let paid = ledger(&engine);
    let inputs_consumed = paid[0].consumed == ledger_before[0].consumed + 1;
    let primary_tally_at_paid_boundary = tally(&engine);
    assert!(!tally_key_present_before_payment);
    assert_eq!(
        (
            primary_tally_before_payment.completed_count,
            primary_tally_at_paid_boundary.completed_count
        ),
        (0, 0)
    );
    let knowledge_enabled = sim_systems::runtime::knowledge::rule(&engine.resources).is_some();
    let admission_eligible =
        gate_mask & !sim_systems::runtime::knowledge::practice_of(&engine.world, entity) == 0;
    let gating_nodes = {
        let k = engine.world.get::<&Knowledge>(entity).expect("knowledge");
        knowledge::ids_of(gate_mask)
            .map(|node_id| {
                let entry = k.get(node_id);
                GateRow {
                    node_id,
                    entry_present: entry.is_some(),
                    proficiency: entry.map_or(0, |e| e.proficiency),
                    last_used: entry.map(|e| e.last_used),
                    admission_threshold: knowledge::defs().threshold(node_id),
                }
            })
            .collect()
    };
    let skill = engine
        .world
        .get::<&Skills>(entity)
        .expect("skills")
        .get(recipe.skill);
    let material = engine
        .resources
        .colony
        .materials
        .stats(FLINT)
        .expect("flint stats");
    assert_eq!(material.hardness, 700);
    assert_eq!(skill.level, level);
    let bill_done_before = engine.resources.colony.workshops[&shop]
        .bill(bill)
        .expect("bill")
        .done;
    let before = tally(&engine);
    engine.resources.current_tick = 27360;
    craft::finish_craft(&mut engine.world, &mut engine.resources, 1, entity, &job);
    let after = tally(&engine);
    let bill_done_after = engine.resources.colony.workshops[&shop]
        .bill(bill)
        .expect("bill")
        .done;
    cancel::finish(&mut engine.world, &mut engine.resources, entity);
    let ledger_after = ledger(&engine);
    let output_delta = ledger_after[1].created - ledger_before[1].created;
    let cleanup_complete = engine.world.get::<&Job>(entity).is_err();
    let reservation_check_passed = invariants::check(&engine.world, &engine.resources).is_ok();
    let ledger_balanced = [ItemKind::Flint, ItemKind::FlintBlade]
        .into_iter()
        .all(|k| invariants::ledger_of(&engine.world, &engine.resources, k).balanced());
    let completion_count_delta = after.completed_count - before.completed_count;
    let actual_quality = u8::try_from(after.quality_sum - before.quality_sum).expect("one quality");
    rows.push(CraftRow {
        event: CraftEvent {
            seed,
            agent_id: 1,
            executed_tick: 27360,
            workshop_id: shop,
            bill_id: bill,
            recipe_code: recipe.id.code(),
            recipe_identity: recipe.identity.id.clone(),
            catalog_revision: content::installed_or_bundled().revision().to_owned(),
            output_item_code: recipe.output.0.code(),
            output_count: recipe.output.1,
            at: [at.0, at.1],
            job_stage: "working",
            ticks_left,
            skill_code: recipe.skill.code(),
            skill_present: true,
            xp_before: skill.xp,
            level_before: skill.level,
            material_id: job.material.map(|m| m.raw()),
            material_hardness: Some(material.hardness),
            hardness_percent: Some(material.hardness_percent()),
            temperament_present: true,
            persistence: Some(500),
            knowledge_enabled,
            gate_mask,
            gating_nodes,
            recipe_quality_add: recipe.quality_add,
            material_term: MaterialTerm::Hardness { offset, divisor },
            tally_before_count: before.completed_count,
            tally_after_count: after.completed_count,
            tally_before_quality_sum: before.quality_sum,
            tally_after_quality_sum: after.quality_sum,
            completion_count_delta,
            actual_quality,
            alive_after_tick: engine.world.get::<&Agent>(entity).is_ok(),
            arithmetic_manifest_sha256: arithmetic_hash.to_owned(),
        },
        fixture_id: format!("seed-{seed}-level-{level}-proficiency-{proficiency}"),
        cap_arm: if proficiency == 1000 {
            "unrestricted"
        } else {
            "restricted"
        },
        target_level: level,
        target_proficiency: proficiency,
        admission_eligible,
        inputs_consumed,
        ledger_before,
        ledger_after,
        output_delta,
        bill_done_before,
        bill_done_after,
        setup_finished_tick,
        add_bill_result,
        paid_working_observed: true,
        ticks_left_at_paid_boundary: ticks_left,
        primary_tally_before_payment,
        primary_tally_at_paid_boundary,
        tally_key_present_before_payment,
        cleanup_complete,
        ledger_balanced,
        reservation_check_passed,
    });
    assert!(knowledge_enabled && admission_eligible && inputs_consumed);
    assert_eq!(
        (
            completion_count_delta,
            output_delta,
            bill_done_after - bill_done_before
        ),
        (1, 1, 1)
    );
    assert!(cleanup_complete && ledger_balanced && reservation_check_passed);
}
