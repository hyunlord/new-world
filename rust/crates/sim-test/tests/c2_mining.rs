//! C2-b ordinary mining: admission, catalog yield, exact ledger, depletion, save and the
//! genuine C2-a format22 adapter.
use sim_core::colony::content::installed_or_bundled;
use sim_core::colony::knowledge::{self, Knowledge, Source};
use sim_core::colony::{Command, Designation, RejectReason};
use sim_core::components::Agent;
use sim_systems::commands::submit;
use sim_systems::runtime::jobs::items::chebyshev;

fn scene() -> (sim_engine::SimEngine, (u32, u32)) {
    let mut e = sim_test::colony_engine(0);
    let camp = sim_systems::worldgen::colony::hearth(&e.resources).expect("camp");
    let tile = e
        .resources
        .ore
        .deposits
        .keys()
        .copied()
        .min_by_key(|&t| (chebyshev(t, camp), t.1, t.0))
        .expect("seed0 deposit");
    let people: Vec<_> = e.world.query::<&Agent>().iter().map(|(x, _)| x).collect();
    for entity in people {
        let mut k = e
            .world
            .get::<&Knowledge>(entity)
            .map(|k| (*k).clone())
            .unwrap_or_default();
        for name in ["mining", "prospecting"] {
            k.set(knowledge::id(name).expect(name), 800, Source::Seeded, 0);
        }
        e.world.insert_one(entity, k).expect("grant");
    }
    (e, tile)
}

fn mark(e: &mut sim_engine::SimEngine, t: (u32, u32)) -> sim_core::colony::CommandResult {
    submit(
        e,
        Command::DesignateMine {
            x0: t.0,
            y0: t.1,
            x1: t.0,
            y1: t.1,
        },
    )
}

#[test]
fn hidden_deposits_refuse_marks_and_revealed_ones_are_mined_exactly() {
    let (mut e, tile) = scene();
    let result = mark(&mut e, tile);
    assert!(!result.accepted && result.reason == Some(RejectReason::Nothing));
    // Reveal by an ordinary drafted walk beside the deposit (a real witness).
    sim_systems::runtime::nav::ensure(&mut e.resources);
    let camp = sim_systems::worldgen::colony::hearth(&e.resources).expect("camp");
    let near = (tile.1.saturating_sub(1)..=tile.1 + 1)
        .flat_map(|y| (tile.0.saturating_sub(1)..=tile.0 + 1).map(move |x| (x, y)))
        .filter(|t| e.resources.nav.passable(*t) && e.resources.nav.reachable(camp, *t))
        .min_by_key(|t| (chebyshev(*t, camp), t.1, t.0))
        .expect("approach");
    let walker = e
        .world
        .query::<&Agent>()
        .iter()
        .map(|(_, a)| a.id)
        .min()
        .expect("agent");
    for cmd in [
        Command::SetDrafted {
            agent: walker,
            drafted: true,
        },
        Command::DraftMove {
            agent: walker,
            x: near.0,
            y: near.1,
        },
    ] {
        assert!(submit(&mut e, cmd).accepted);
    }
    while !e.resources.ore.deposits[&tile].revealed && e.current_tick() < 3_000 {
        e.tick();
    }
    assert!(
        submit(
            &mut e,
            Command::SetDrafted {
                agent: walker,
                drafted: false
            }
        )
        .accepted
    );
    assert!(mark(&mut e, tile).accepted);
    assert_eq!(
        e.resources.colony.designations.get(&tile),
        Some(&Designation::Mine)
    );
    let def = installed_or_bundled()
        .outcrop(e.resources.ore.deposits[&tile].kind)
        .expect("def")
        .clone();
    let start = e.resources.ore.deposits[&tile].remaining;
    for _ in 0..(40 * 1_440) {
        e.tick();
        if e.resources.ore.deposits[&tile].remaining == 0 {
            break;
        }
    }
    let mined = u64::from(start - e.resources.ore.deposits[&tile].remaining);
    assert!(mined > 0, "an ordinary worker mined");
    assert_eq!(
        e.resources.colony.item_ledger.created_of(def.yield_item),
        mined
    );
    let recorded: u32 = e.resources.metal.mined.values().sum();
    assert_eq!(u64::from(recorded), mined);
    sim_systems::runtime::jobs::invariants::check(&e.world, &e.resources).expect("ledger");
    if e.resources.ore.deposits[&tile].remaining == 0 {
        assert!(
            !e.resources.colony.designations.contains_key(&tile),
            "exhausted mark removed"
        );
        assert!(e.resources.ore.deposits.contains_key(&tile), "geology kept");
        assert!(!mark(&mut e, tile).accepted);
    }
    let bytes = sim_systems::save::save(&mut e).expect("save23");
    let mut loaded = sim_systems::save::load(&bytes).expect("load23");
    assert_eq!(sim_systems::save::save(&mut loaded).expect("resave"), bytes);
    assert_eq!(loaded.resources.metal, e.resources.metal);
}

#[test]
fn tin_goods_follow_knowledge_and_tin_outcrop_absence() {
    use sim_systems::runtime::neighbours::trade::{eligible_goods, GoodsInputs};
    let catalog = installed_or_bundled();
    let data = sim_core::rules::neighbour_data().expect("neighbours");
    let rule = sim_core::rules::NeighbourRule::m5_5().expect("rule");
    let tin = catalog.item_by_name("tin_ingot").expect("tin");
    let has_tin = |inputs: &GoodsInputs| {
        eligible_goods(catalog, &data.goods, &rule, data.goods_policy, inputs)
            .iter()
            .any(|row| row.kind == tin)
    };
    let mut inputs = GoodsInputs::default();
    assert!(!has_tin(&inputs), "unknown tin sourcing");
    inputs.colony_known = knowledge::bit(knowledge::id("tin_sourcing").expect("node"));
    assert!(has_tin(&inputs), "known and no tin outcrop");
    inputs.outcrop_yield_tags.insert("metal_tin".into());
    assert!(!has_tin(&inputs), "a tin outcrop exists");
}

/// Genuine format22 saves written by the unmodified C2-a candidate (external path).
#[test]
fn genuine_c2a_format22_saves_load_with_mining_counter_zero() {
    let Ok(dir) = std::env::var("C2_A22_FIXTURES") else {
        return;
    };
    for name in ["outcrop-found.bin", "day2.wsave"] {
        let bytes = std::fs::read(std::path::Path::new(&dir).join(name)).expect("fixture");
        assert_eq!(&bytes[8..12], &22u32.to_le_bytes());
        let mut e = sim_systems::save::load(&bytes).expect("format22 adapter");
        let skills: Vec<_> = e
            .world
            .query::<&sim_core::colony::skills::Skills>()
            .iter()
            .map(|(_, s)| s.completed)
            .collect();
        assert!(skills.iter().all(|c| c.len() == 27 && c[26] == 0));
        assert!(e.resources.metal.is_empty());
        let current = sim_systems::save::save(&mut e).expect("resave as 23");
        assert_eq!(&current[8..12], &23u32.to_le_bytes());
        e.tick();
    }
}
