//! M-ART2 appearance read (`sim_bridge::ffi::appearance`, MART2_GOAL 3.2): the look and face variant are pure
//! functions of the person (AgentId, life stage, sex, top) — equal across a save/load round trip and in a twin
//! engine; hair and variant spread over every value; elders take only the older variants and children the child
//! band; the rows line up with the agent snapshot; the text renderer's `--people-json` carries the same looks
//! (C2 agreement extended to appearance); the baseline world's columns are consistent.
//!
//! Expected values are computed here from the documented formula and constants (`data/appearance.ron`), not by
//! calling the function under test. M-ART2.1 adds the kin columns (`mothers`, `fathers`, `partners`) the LOD
//! household rings read: row-aligned with the agent snapshot and equal to each entity's `Kin` component.
use sim_bridge::ffi::{
    collect_agent_snapshot, collect_appearance, collect_kin_columns, AppearanceRow,
};
use sim_core::colony::people::Apparel;
use sim_core::components::{Agent, Child, Elder, Kin, Position, Sex};
use sim_engine::SimEngine;
use sim_systems::save;
use sim_test::probe::ascii::{self, AsciiArgs};
use sim_test::probe::colony::{load_script, resolve, ScriptEntry};
use std::collections::{BTreeMap, BTreeSet};

// The constants of `sim-bridge/data/appearance.ron`, restated for the independent expectation.
const VARIANT_MULTIPLIER: u128 = 2_654_435_761;
const HAIR_MULTIPLIER: u128 = 2_246_822_519;
const HAIR_SHIFT: u32 = 16;
const VARIANTS: u128 = 6;
const ELDER_VARIANTS: [u8; 2] = [3, 4];
const SEXES: u32 = 2;
const TOPS: u32 = 3;
const HAIRS: u128 = 3;

fn expected_hair(id: u64) -> u8 {
    ((((u128::from(id) * HAIR_MULTIPLIER) % (1u128 << 32)) >> HAIR_SHIFT) % HAIRS) as u8
}

fn expected_variant(id: u64, elder: bool) -> u8 {
    let h = (u128::from(id) * VARIANT_MULTIPLIER) % (1u128 << 32);
    if elder {
        ELDER_VARIANTS[(h % 2) as usize]
    } else {
        (h % VARIANTS) as u8
    }
}

/// The expected row of a person from its components (life stage as the snapshot: Child, else Elder, else adult).
fn expected_row(id: u64, child: bool, elder: bool, female: bool, top_code: u8) -> AppearanceRow {
    let age: u8 = if child { 0 } else { 1 };
    let sex = u8::from(female);
    let hair = expected_hair(id);
    let look = ((u32::from(age) * SEXES + u32::from(sex)) * TOPS + u32::from(top_code))
        * HAIRS as u32
        + u32::from(hair);
    AppearanceRow {
        agent_id: id,
        sex,
        age,
        top: top_code,
        hair,
        look,
        variant: expected_variant(id, elder && !child),
    }
}

/// Every person's expected row in AgentId order, read from the components directly.
fn expected(engine: &SimEngine) -> BTreeMap<u64, AppearanceRow> {
    engine
        .world
        .query::<(
            &Agent,
            &Position,
            Option<&Child>,
            Option<&Elder>,
            Option<&Sex>,
            Option<&Apparel>,
        )>()
        .iter()
        .map(|(_, (a, _, c, e, s, ap))| {
            let top = ap.map_or(0, |ap| ap.top.code());
            let row = expected_row(
                a.id,
                c.is_some(),
                e.is_some(),
                s.is_some_and(|s| s.female),
                top,
            );
            (a.id, row)
        })
        .collect()
}

fn by_id(rows: &[AppearanceRow]) -> BTreeMap<u64, AppearanceRow> {
    rows.iter().map(|r| (r.agent_id, *r)).collect()
}

/// The appearance rows line up with the agent snapshot's rows (same length, same AgentId per index).
fn assert_aligned(label: &str, engine: &SimEngine) {
    let snap: Vec<u64> = collect_agent_snapshot(&engine.world)
        .iter()
        .map(|r| r.agent_id)
        .collect();
    let looks: Vec<u64> = collect_appearance(&engine.world)
        .iter()
        .map(|r| r.agent_id)
        .collect();
    assert_eq!(snap, looks, "{label}: rows in snapshot order");
}

fn u_script() -> Vec<ScriptEntry> {
    let path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../tools/v8/u_script.json");
    let entries = load_script(&path).expect("u_script.json loads");
    assert!(!entries.is_empty());
    entries
}

fn advance(engine: &mut SimEngine, script: &[ScriptEntry], to: u64) {
    while engine.current_tick() < to {
        let tick = engine.current_tick();
        let mut avoid = Vec::new();
        for entry in script.iter().filter(|e| e.tick == tick) {
            for cmd in resolve(engine, &entry.spec, &mut avoid).expect("script resolves") {
                let _ = sim_systems::commands::submit(engine, cmd);
            }
        }
        engine.tick();
    }
}

#[test]
fn appearance_is_a_pure_function_across_save_load_and_a_twin() {
    let script = u_script();
    let mut a = sim_test::colony_engine(3);
    let mut b = sim_test::colony_engine(3);
    advance(&mut a, &script, 2_000);
    advance(&mut b, &script, 2_000);
    let rows_a = collect_appearance(&a.world);
    // Observed: the 10 founders (and anyone born or joined by tick 2,000).
    assert!(rows_a.len() >= 10, "{} people", rows_a.len());
    assert_aligned("colony seed 3", &a);
    assert_eq!(by_id(&rows_a), expected(&a), "formula");
    // The twin engine (same seed, same orders) gives the same looks.
    assert_eq!(rows_a, collect_appearance(&b.world), "twin");
    // Save → load: the same AgentIds give the same looks and variants.
    let bytes = save::save(&mut a).expect("save");
    let mut loaded = save::load(&bytes).expect("load");
    assert_eq!(
        by_id(&collect_appearance(&loaded.world)),
        by_id(&rows_a),
        "after load"
    );
    assert_aligned("loaded", &loaded);
    // And they keep agreeing as both run on (hair never changes; the variant moves only with the life stage).
    advance(&mut a, &script, 3_000);
    advance(&mut loaded, &script, 3_000);
    assert_eq!(
        by_id(&collect_appearance(&loaded.world)),
        by_id(&collect_appearance(&a.world))
    );
    for (id, row) in by_id(&collect_appearance(&a.world)) {
        if let Some(before) = by_id(&rows_a).get(&id) {
            assert_eq!(row.hair, before.hair, "agent {id}: hair is fixed");
        }
    }
}

#[test]
fn hair_and_variant_spread_elders_take_older_faces_children_the_child_band() {
    // 200 founders on the stress world (baseline rules): ≥ 30 people.
    let mut engine = sim_bridge::ffi::build_stress_engine(5, 200);
    let mut ids: Vec<(u64, hecs::Entity)> = engine
        .world
        .query::<&Agent>()
        .iter()
        .map(|(e, a)| (a.id, e))
        .collect();
    ids.sort_unstable();
    assert!(ids.len() >= 30, "{} people", ids.len());
    // Make the first 40 elders and the next 40 children (a fixture: the collector is a pure read).
    for (n, (_, entity)) in ids.iter().enumerate() {
        if n < 40 {
            let _ = engine.world.remove_one::<Child>(*entity);
            engine.world.insert_one(*entity, Elder).expect("live");
        } else if n < 80 {
            let _ = engine.world.remove_one::<Elder>(*entity);
            engine
                .world
                .insert_one(*entity, Child { born_tick: 0 })
                .expect("live");
        } else {
            let _ = engine.world.remove_one::<Child>(*entity);
            let _ = engine.world.remove_one::<Elder>(*entity);
        }
    }
    assert_aligned("stress", &engine);
    let rows = collect_appearance(&engine.world);
    assert_eq!(by_id(&rows), expected(&engine), "formula");
    let stage: BTreeMap<u64, usize> = ids
        .iter()
        .enumerate()
        .map(|(n, (id, _))| (*id, n))
        .collect();
    let mut variants = BTreeSet::new();
    let mut elder_variants = BTreeSet::new();
    let mut hairs = BTreeSet::new();
    let mut sexes = BTreeSet::new();
    for row in &rows {
        let n = stage[&row.agent_id];
        hairs.insert(row.hair);
        sexes.insert(row.sex);
        assert!(row.look < 36, "{row:?}");
        if n < 40 {
            assert!(
                row.variant == 3 || row.variant == 4,
                "elder {row:?} takes an older face"
            );
            assert_eq!(row.look / 18, 1, "an elder keeps the adult look");
            elder_variants.insert(row.variant);
        } else if n < 80 {
            assert_eq!((row.age, row.look / 18), (0, 0), "child band {row:?}");
            variants.insert(row.variant);
        } else {
            assert_eq!(row.age, 1);
            variants.insert(row.variant);
        }
    }
    assert_eq!(
        variants,
        (0..6).collect::<BTreeSet<u8>>(),
        "all six variants"
    );
    assert_eq!(elder_variants, BTreeSet::from([3, 4]), "both older faces");
    assert_eq!(hairs, (0..3).collect::<BTreeSet<u8>>(), "all three hairs");
    assert_eq!(sexes, BTreeSet::from([0, 1]), "both sexes");
}

#[test]
fn a_person_without_sex_counts_as_male_and_tops_follow_apparel() {
    use sim_core::colony::people::Top;
    let mut world = hecs::World::new();
    world.spawn((Agent { id: 7 }, Position::new(1, 1)));
    world.spawn((Agent { id: 8 }, Position::new(2, 1), Sex { female: true }));
    world.spawn((
        Agent { id: 9 },
        Position::new(3, 1),
        Sex { female: false },
        Apparel {
            top: Top::FurCloak,
            since_day: 0,
        },
    ));
    let rows = by_id(&collect_appearance(&world));
    assert_eq!(rows[&7], expected_row(7, false, false, false, 0));
    assert_eq!(rows[&8], expected_row(8, false, false, true, 0));
    assert_eq!(rows[&9], expected_row(9, false, false, false, 2));
    assert_eq!((rows[&7].sex, rows[&8].sex, rows[&9].top), (0, 1, 2));
}

#[test]
fn the_text_renderer_people_json_carries_the_bridge_looks() {
    let script_path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../tools/v8/u_script.json");
    let dir = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("engine_appearance_ffi");
    std::fs::create_dir_all(&dir).expect("tmp dir");
    let json_path = dir.join("people.json");
    let args = AsciiArgs {
        seed: 100,
        tick: 900,
        rect: None,
        commands: Some(script_path.clone()),
        legend: false,
        out: Some(dir.join("grid.txt")),
        people_json: Some(json_path.clone()),
    };
    ascii::run(&args).expect("text renderer runs");
    let text = std::fs::read_to_string(&json_path).expect("people json written");
    let json: serde_json::Value = serde_json::from_str(&text).expect("json");
    let printed: BTreeMap<u64, (u64, u64, u64, u64, u64)> = json["appearance"]
        .as_array()
        .expect("appearance array")
        .iter()
        .map(|p| {
            let n = |k: &str| p[k].as_u64().expect(k);
            (
                n("agent_id"),
                (n("x"), n("y"), n("sex"), n("look"), n("variant")),
            )
        })
        .collect();
    // The same run, read through the bridge collectors.
    let script = load_script(&script_path).expect("script");
    let (engine, _) = ascii::engine_at(100, &script, 900).expect("run");
    let snap = collect_agent_snapshot(&engine.world);
    let looks = collect_appearance(&engine.world);
    let bridge: BTreeMap<u64, (u64, u64, u64, u64, u64)> = snap
        .iter()
        .zip(&looks)
        .map(|(s, l)| {
            (
                s.agent_id,
                (
                    u64::from(s.x),
                    u64::from(s.y),
                    u64::from(l.sex),
                    u64::from(l.look),
                    u64::from(l.variant),
                ),
            )
        })
        .collect();
    // Observed: the 10 founders of seed 100 at tick 900.
    assert!(bridge.len() >= 10, "{} people", bridge.len());
    assert_eq!(printed, bridge, "text renderer = bridge, every person");
    assert_eq!(json["count"].as_u64(), Some(bridge.len() as u64));
    // The people tiles agree with the appearance rows' tiles (one entry per person).
    let mut tiles: Vec<(u64, u64)> = json["people"]
        .as_array()
        .expect("people")
        .iter()
        .map(|t| (t[0].as_u64().expect("x"), t[1].as_u64().expect("y")))
        .collect();
    tiles.sort_unstable();
    let mut from_looks: Vec<(u64, u64)> = printed.values().map(|v| (v.0, v.1)).collect();
    from_looks.sort_unstable();
    assert_eq!(tiles, from_looks);
}

#[test]
fn the_baseline_rows_are_consistent() {
    let mut engine = sim_test::baseline_engine(0);
    for _ in 0..100 {
        engine.tick();
    }
    assert_aligned("baseline", &engine);
    let rows = collect_appearance(&engine.world);
    assert_eq!(rows.len(), collect_agent_snapshot(&engine.world).len());
    assert!(!rows.is_empty());
    assert_eq!(by_id(&rows), expected(&engine));
    // An empty world has no rows.
    assert!(collect_appearance(&hecs::World::new()).is_empty());
}

#[test]
fn kin_columns_line_up_with_the_snapshot_and_the_kin_component() {
    let mut engine = sim_test::baseline_engine(3);
    let ticks_per_day = engine.resources.ticks_per_day.max(1);
    // Run until the world holds both a recorded mother and a partner (births and pairings), at most 120 days.
    let mut day = 0;
    loop {
        let kin = collect_kin_columns(&engine.world);
        if kin.iter().any(|r| r[0].is_some()) && kin.iter().any(|r| r[2].is_some()) {
            break;
        }
        assert!(day < 120, "no mother and partner recorded within 120 days");
        for _ in 0..ticks_per_day {
            engine.tick();
        }
        day += 1;
    }
    let rows = collect_agent_snapshot(&engine.world);
    let kin = collect_kin_columns(&engine.world);
    assert_eq!(rows.len(), kin.len());
    assert!(rows.len() > 10, "a populated world: {} people", rows.len());
    let mut with_kin = 0;
    for (row, k) in rows.iter().zip(&kin) {
        let entity = hecs::Entity::from_bits(row.entity_bits).expect("snapshot entity bits");
        let expected = engine
            .world
            .get::<&Kin>(entity)
            .map(|c| [c.mother, c.father, c.partner])
            .unwrap_or([None; 3]);
        if engine.world.get::<&Kin>(entity).is_ok() {
            with_kin += 1;
        }
        assert_eq!(*k, expected, "kin of entity {}", row.entity_bits);
        // The row is the same person as the snapshot row (Agent of that entity).
        assert!(engine.world.get::<&Agent>(entity).is_ok());
    }
    assert!(with_kin > 0);
}
