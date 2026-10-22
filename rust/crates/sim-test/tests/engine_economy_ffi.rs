//! M2 bridge collectors (`sim_bridge::ffi::economy`): workshops with bills,
//! work priorities, recipes, one person's economy, and the bill/priority
//! commands parsed from names — content and order, headless.

mod colony_support;

use colony_support::{build_workshop, drop_items, job_world_m2, worker};
use sim_bridge::ffi::{
    collect_agent_economy, collect_colony_overlay, collect_recipes, collect_work_priorities,
    collect_workshops, levels_in_work_order, submit_economy, EconomyCommand,
};
use sim_core::colony::{BlueprintKind, ItemKind, ItemSpec, RejectReason, Skills, Tool};
use sim_core::components::Agent;

#[test]
fn workshops_list_bills_in_order_and_commands_parse_names() {
    let mut engine = job_world_m2(32, 32);
    worker(&mut engine, 3, 3);
    let bench = build_workshop(&mut engine, BlueprintKind::WoodBench, (12, 12), (4, 4));
    let spot = build_workshop(&mut engine, BlueprintKind::KnappingSpot, (20, 12), (4, 4));
    let add = |recipe: &str, mode: &str, count: i64, shop: u32| EconomyCommand::AddBill {
        workshop: i64::from(shop),
        recipe: recipe.into(),
        mode: mode.into(),
        count,
    };
    assert!(submit_economy(&mut engine, &add("knap_blade", "make", 6, spot)).accepted);
    assert!(submit_economy(&mut engine, &add("haft_axe", "make", 2, bench)).accepted);
    assert!(submit_economy(&mut engine, &add("split_firewood", "maintain", 40, bench)).accepted);
    let wrong = submit_economy(&mut engine, &add("knap_blade", "make", 1, bench));
    assert_eq!(wrong.reason, Some(RejectReason::WrongWorkshop));
    let unknown = submit_economy(&mut engine, &add("bronze", "make", 1, bench));
    assert_eq!(unknown.reason, Some(RejectReason::Unknown));
    let rows = collect_workshops(&engine.resources);
    assert_eq!(
        rows.iter().map(|r| (r.id, r.kind)).collect::<Vec<_>>(),
        vec![(bench, 2), (spot, 1)]
    );
    // The bench's auto-bill was replaced by the player's firewood bill; haft first.
    let bench_row = &rows[0];
    let recipes: Vec<(u8, u8, u16, bool)> = bench_row
        .bills
        .iter()
        .map(|b| (b.recipe, b.mode, b.count, b.auto))
        .collect();
    assert_eq!(recipes, vec![(1, 0, 2, false), (2, 1, 40, false)]);
    assert_eq!((bench_row.rect, bench_row.work), ((12, 12, 2, 1), (13, 13)));
    let bills: Vec<u32> = bench_row.bills.iter().map(|b| b.id).collect();
    let mv = EconomyCommand::MoveBill {
        workshop: i64::from(bench),
        bill: i64::from(bills[1]),
        up: true,
    };
    assert!(submit_economy(&mut engine, &mv).accepted);
    let after: Vec<u32> = collect_workshops(&engine.resources)[0]
        .bills
        .iter()
        .map(|b| b.id)
        .collect();
    assert_eq!(after, vec![bills[1], bills[0]]);
    // The recipe table in code order.
    let table: Vec<(u8, &str, u8)> = collect_recipes().iter().map(|r| (r.0, r.1, r.2)).collect();
    assert_eq!(
        table,
        vec![
            (0, "knap_blade", 1),
            (1, "haft_axe", 2),
            (2, "split_firewood", 2),
            // M4: pressure flaking, ground stone and the pottery chain.
            (3, "knap_fine_blade", 1),
            (4, "grind_axe", 9),
            (5, "form_pot", 10),
            (6, "fire_pot", 11)
        ]
    );
    // A workshop and its site appear in the overlay as nothing else.
    let overlay = collect_colony_overlay(&engine.world, &engine.resources);
    assert!(overlay.sites.is_empty() && overlay.nodes.is_empty());
}

#[test]
fn priorities_and_one_persons_economy() {
    let mut engine = sim_test::colony_engine(0);
    let rows = collect_work_priorities(&engine.world);
    assert!(rows.len() >= 7);
    assert!(
        rows.windows(2).all(|w| w[0].agent < w[1].agent),
        "AgentId order"
    );
    for r in &rows {
        let quarry = if r.elder { 0 } else { 3 };
        // M3: mining (code 6) follows quarrying for elders.
        assert_eq!(r.levels, [3, 3, quarry, 3, 3, 3, quarry]);
    }
    let first = rows[0].agent;
    let set = EconomyCommand::SetWorkPriority {
        agent: first as i64,
        work: "chop".into(),
        level: 0,
    };
    assert!(submit_economy(&mut engine, &set).accepted);
    let bad = EconomyCommand::SetWorkPriority {
        agent: first as i64,
        work: "dance".into(),
        level: 1,
    };
    assert_eq!(
        submit_economy(&mut engine, &bad).reason,
        Some(RejectReason::Unknown)
    );
    assert_eq!(collect_work_priorities(&engine.world)[0].levels[3], 0);
    // M3: the bridge lists levels in `work_types` (WorkType::ALL) order, where
    // mining (code 6) is fourth; a code-order flatten showed chop's level under
    // the mining column.
    let mine = EconomyCommand::SetWorkPriority {
        agent: first as i64,
        work: "mine".into(),
        level: 1,
    };
    assert!(submit_economy(&mut engine, &mine).accepted);
    let row = collect_work_priorities(&engine.world)[0];
    let order: Vec<&str> = sim_core::colony::WorkType::ALL
        .iter()
        .map(|w| w.name())
        .collect();
    assert_eq!(
        order,
        ["craft", "build", "quarry", "mine", "chop", "split", "haul"]
    );
    assert_eq!(levels_in_work_order(&row)[3], 1, "mine");
    assert_eq!(levels_in_work_order(&row)[4], 0, "chop");
    // Economy of one person: skills, a tool, a load.
    let entity = engine
        .world
        .query::<&Agent>()
        .iter()
        .find(|(_, a)| a.id == first)
        .map(|(e, _)| e)
        .expect("the person");
    let mut skills = Skills::default();
    skills.train(sim_core::colony::SkillKind::Knap, 300);
    let axe = ItemSpec {
        kind: ItemKind::StoneAxe,
        material: Some(sim_core::colony::materials::FLINT),
        quality: 41,
    };
    let _ = engine
        .world
        .insert(entity, (skills, Tool { item: axe, uses: 3 }));
    let row = collect_agent_economy(&engine.world, &engine.resources, first).expect("found");
    assert_eq!((row.levels[0], row.xp[0]), (4, 300));
    assert_eq!(row.tool, Some((5, 41, "flint".to_string())));
    assert_eq!(row.carry, None);
    assert!(collect_agent_economy(&engine.world, &engine.resources, 999_999).is_none());
    // The overlay lists the colony's nodes in tile order with their kinds.
    let overlay = collect_colony_overlay(&engine.world, &engine.resources);
    assert_eq!(overlay.nodes.len(), engine.resources.colony.nodes.len());
    assert!(overlay.nodes.windows(2).all(|w| w[0].0 < w[1].0));
    drop_items(&mut engine, (1, 1), ItemSpec::raw(ItemKind::Flint, None), 3);
}

#[test]
fn the_baseline_has_no_economy() {
    let engine = sim_test::baseline_engine(0);
    assert!(collect_workshops(&engine.resources).is_empty());
    let overlay = collect_colony_overlay(&engine.world, &engine.resources);
    assert!(overlay.nodes.is_empty() && overlay.designation_kinds.is_empty());
}
