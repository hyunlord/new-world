//! C2 authored bronze coverage, stable ordinals and real exchange discovery evidence.

use sim_core::colony::content::{Catalog, RawSources};
use sim_core::colony::knowledge::{self, Cond, ContentStatus, Gate, Seeding};
use sim_systems::runtime::knowledge::discover::holds;

const ADDED: [&str; 47] = [
    "charcoal",
    "prospecting",
    "mining",
    "copper_smelting",
    "cold_hammering",
    "mould_casting",
    "tin_sourcing",
    "bronze_alloy",
    "annealing",
    "lost_wax",
    "beekeeping",
    "wheel",
    "cart",
    "cattle",
    "plough",
    "irrigation",
    "terrace",
    "wool_spinning",
    "mud_brick_house",
    "lime_plaster",
    "stone_masonry",
    "chief_hall",
    "chieftainship",
    "shrine",
    "marriage_alliance",
    "palisade",
    "ditch",
    "bronze_weapons",
    "composite_bow",
    "caravan",
    "trading_post",
    "seal_stamp",
    "proto_writing",
    "standard_weights",
    "solar_calendar",
    "reed_boat",
    "sail",
    "wine",
    "olive_press",
    "glass_beads",
    "faience",
    "trepanation",
    "surgery",
    "drum",
    "flute",
    "mask",
    "tattoo",
];
const CONNECTED: [&str; 16] = [
    "charcoal",
    "prospecting",
    "mining",
    "copper_smelting",
    "cold_hammering",
    "mould_casting",
    "tin_sourcing",
    "bronze_alloy",
    "wheel",
    "cattle",
    "wool_spinning",
    "mud_brick_house",
    "stone_masonry",
    "palisade",
    "bronze_weapons",
    "composite_bow",
];

#[test]
fn c2_knowledge_validator() {
    let catalog = Catalog::from_sources(RawSources::bundled(), None).expect("C2 catalogue");
    let defs = catalog.knowledge_defs();
    assert_eq!(defs.nodes.len(), 129);
    assert_eq!(defs.id_of("bone_setting"), Some(81));
    for (offset, name) in ADDED.into_iter().enumerate() {
        let node = defs.get(defs.id_of(name).expect(name)).expect(name);
        assert_eq!(
            usize::from(node.id),
            82 + offset,
            "append-only ordinal: {name}"
        );
        assert_eq!(node.era.code(), 2, "bronze: {name}");
        assert_eq!(node.seed, Seeding::Nobody, "normal-world seed: {name}");
        assert_eq!(node.threshold, 300, "authored initial threshold: {name}");
        assert!(
            !node.discover.is_empty(),
            "physical discovery cause: {name}"
        );
        assert!((1..=4).contains(&node.tier), "authored tier: {name}");
        assert_eq!(
            node.status == ContentStatus::Connected,
            CONNECTED.contains(&name),
            "{name}"
        );
        if node.status == ContentStatus::Connected {
            for required in &node.prereqs {
                assert_eq!(
                    defs.get(*required).expect("prerequisite").status,
                    ContentStatus::Connected,
                    "{name}"
                );
            }
        } else {
            let debt = node.debt.as_deref().expect("named future pass");
            assert!(
                ["C3:", "M8:", "M9:", "M-GROW:", "M-BAL:"]
                    .iter()
                    .any(|prefix| debt.starts_with(prefix)),
                "{name}: {debt}"
            );
            assert!(node.gates.is_empty(), "inert runtime gate: {name}");
            assert_eq!(node.marker_weight, 0, "inert era marker: {name}");
        }
    }
    let markers: Vec<_> = defs
        .nodes
        .iter()
        .filter(|n| n.era.code() == 2 && n.marker_weight > 0)
        .map(|n| (n.name.as_str(), n.marker_weight))
        .collect();
    assert_eq!(
        markers,
        [
            ("mud_brick", 1),
            ("copper_smelting", 3),
            ("mould_casting", 2),
            ("bronze_alloy", 3),
            ("wheel", 2)
        ]
    );
    // The bible's existing flax_retting entry is already the spinning node's physical recipe.
    assert_eq!(defs.id_of("flax_retting"), None);
    let spinning = defs
        .get(defs.id_of("spinning").expect("spinning"))
        .expect("spinning");
    assert!(spinning.gates.contains(&Gate::Recipe(
        catalog.recipe_by_name("ret_flax").expect("retting")
    )));
    let wool = defs
        .get(defs.id_of("wool_spinning").expect("wool spinning"))
        .expect("wool spinning");
    assert!(
        !wool
            .discover
            .iter()
            .any(|cond| matches!(cond, Cond::Stockpile(..))),
        "wool yield cannot require its own output to discover"
    );
    let source = catalog.source_text("knowledge.ron").expect("authored map");
    let invalid = source.replace("Trades(1)", "Trades(0)");
    assert_ne!(source, invalid);
    assert!(knowledge::validate(&invalid, &catalog).is_err());
}

#[test]
fn c2_barter_discovery_reads_successful_exchange_evidence() {
    let mut engine = sim_test::colony_engine(0);
    let condition = Cond::Trades(1);
    assert!(!holds(&engine.world, &engine.resources, condition));
    engine.resources.colony.neighbours.tallies.trades_refused = 10;
    engine.resources.colony.neighbours.tallies.gifts = 10;
    assert!(
        !holds(&engine.world, &engine.resources, condition),
        "refusals and gifts are not exchanges"
    );
    engine.resources.colony.neighbours.tallies.trades = 1;
    assert!(holds(&engine.world, &engine.resources, condition));
    assert!(!holds(&engine.world, &engine.resources, Cond::Trades(2)));
    let barter = knowledge::defs()
        .get(knowledge::defs().id_of("barter").expect("barter"))
        .expect("barter");
    assert_eq!(barter.status, ContentStatus::Connected);
    assert!(barter.discover.contains(&condition));
    let bytes = sim_systems::save::save(&mut engine).expect("trade evidence save");
    let loaded = sim_systems::save::load(&bytes).expect("trade evidence load");
    assert!(holds(&loaded.world, &loaded.resources, condition));
}

#[test]
fn c2_alloy_discovery_can_precede_its_first_tin_request() {
    use sim_core::colony::{ItemKind, ItemSpec, JobKind, Skills};

    let defs = knowledge::defs();
    let alloy = defs
        .get(defs.id_of("bronze_alloy").expect("alloy"))
        .expect("alloy");
    assert_eq!(
        alloy.prereqs,
        [
            defs.id_of("mould_casting").expect("casting"),
            defs.id_of("tin_sourcing").expect("tin sourcing"),
        ],
        "retain the bible's two knowledge prerequisites"
    );
    let mut world = hecs::World::new();
    let mut res = sim_engine::SimResources::new(8, 8, sim_core::MaterialRegistry::new());
    let mut skills = Skills::default();
    skills.completed[usize::from(JobKind::code_for_name("craft").expect("craft"))] = 5;
    world.spawn((skills,));
    assert!(!alloy.discover.iter().all(|c| holds(&world, &res, *c)));
    let copper = ItemKind::from_name("copper_ingot").expect("copper ingot");
    let tin = ItemKind::from_name("tin_ingot").expect("tin ingot");
    res.colony.zone_of_tile.insert((2, 2), 0);
    sim_systems::runtime::jobs::items::put_items(&mut res, (2, 2), ItemSpec::raw(copper, None), 3);
    assert!(!holds(&world, &res, Cond::Stockpile(tin, 1)));
    assert!(
        alloy.discover.iter().all(|c| holds(&world, &res, *c)),
        "copper practice can establish alloy knowledge before the bronze-known tin request"
    );
}
