//! C3 authored iron coverage: stable ordinals, connected prerequisites and inert debt.

use sim_core::colony::content::{Catalog, RawSources};
use sim_core::colony::knowledge::{Cond, ContentStatus, Seeding};

const ADDED: [&str; 37] = [
    "iron_smelting",
    "forging",
    "iron_tools",
    "iron_weapons",
    "steel_edge",
    "iron_plough",
    "horse",
    "paved_road",
    "bridge",
    "aqueduct",
    "reservoir",
    "stone_house",
    "town_wall",
    "gate",
    "tower",
    "market",
    "coinage",
    "warehouse",
    "codified_law",
    "taxation",
    "bureaucracy",
    "writing",
    "school",
    "map_making",
    "militia",
    "shield_wall",
    "siege_ladder",
    "polytheism",
    "temple",
    "geometry",
    "sundial",
    "rotary_quern",
    "watermill",
    "silk",
    "dye_purple",
    "falconry",
    "veterinary",
];
const CONNECTED: [&str; 8] = [
    "iron_smelting",
    "forging",
    "iron_tools",
    "iron_weapons",
    "paved_road",
    "stone_house",
    "town_wall",
    "gate",
];

#[test]
fn c3_knowledge_validator() {
    let catalog = Catalog::from_sources(RawSources::bundled(), None).expect("C3 catalogue");
    let defs = catalog.knowledge_defs();
    assert_eq!(defs.nodes.len(), 166);
    assert_eq!(defs.id_of("tattoo"), Some(128));
    for (offset, name) in ADDED.into_iter().enumerate() {
        let node = defs.get(defs.id_of(name).expect(name)).expect(name);
        assert_eq!(
            usize::from(node.id),
            129 + offset,
            "append-only ordinal: {name}"
        );
        assert_eq!(node.era.code(), 3, "iron: {name}");
        assert_eq!(node.seed, Seeding::Nobody, "normal-world seed: {name}");
        assert_eq!(node.threshold, 300, "authored initial threshold: {name}");
        assert_eq!(node.difficulty, node.tier.min(3), "difficulty: {name}");
        assert!(
            !node.discover.is_empty(),
            "physical discovery cause: {name}"
        );
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
                ["M-NATION:", "M8:", "M9:", "M-GROW:", "M-BAL:"]
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
        .filter(|n| n.era.code() == 3 && n.marker_weight > 0)
        .map(|n| (n.name.as_str(), n.marker_weight))
        .collect();
    assert_eq!(
        markers,
        [
            ("iron_smelting", 3),
            ("forging", 3),
            ("iron_tools", 2),
            ("paved_road", 1),
            ("stone_house", 1)
        ]
    );
    // Discovery is AND-only; no ore clause keeps ore-only and bog-only worlds alike able to learn smelting.
    let smelting = defs
        .get(defs.id_of("iron_smelting").expect("iron smelting"))
        .expect("iron smelting");
    assert!(!smelting.discover.iter().any(|cond| matches!(
        cond,
        Cond::Stockpile(item, _) if item.definition().tags.iter().any(|tag| tag == "ore")
    )));
}
