//! C1 authored neolithic coverage, executable gates and negative catalogue validation.

use sim_core::colony::content::{Catalog, RawSources};
use sim_core::colony::knowledge::{self, bit, ContentStatus, Gate, Seeding};

const ADDED: [&str; 39] = [
    "animal_taming",
    "herding",
    "dairy",
    "dog_taming",
    "selective_breeding",
    "querns",
    "oven",
    "fermentation",
    "granary",
    "food_preservation",
    "barley",
    "lentil",
    "pea",
    "millet",
    "rye",
    "orchard",
    "crop_rotation",
    "hoe",
    "sickle",
    "adze",
    "wattle_daub",
    "thatch",
    "mud_brick",
    "well",
    "fence",
    "megalith",
    "dyeing",
    "leather_armor",
    "bow",
    "dugout_canoe",
    "barter",
    "tally",
    "lunar_calendar",
    "ritual",
    "elder_council",
    "kinship",
    "taboo",
    "herbalism",
    "bone_setting",
];

#[test]
fn c1_knowledge_validator() {
    let catalog = Catalog::from_sources(RawSources::bundled(), None).expect("C1 catalogue");
    let defs = catalog.knowledge_defs();
    // The historical 43-node prefix is retained; every requested new node is distinct.
    assert_eq!(defs.nodes.len(), 43 + ADDED.len());
    assert_eq!(defs.id_of("weather_lore"), Some(42));
    for (offset, name) in ADDED.into_iter().enumerate() {
        let id = defs.id_of(name).expect(name);
        assert_eq!(usize::from(id), 43 + offset, "stable ordinal: {name}");
        let node = defs.get(id).expect(name);
        assert_eq!(node.era.code(), 1, "neolithic: {name}");
        assert_eq!(node.seed, Seeding::Nobody, "normal-world seed: {name}");
        assert!(!node.discover.is_empty(), "discovery cause: {name}");
        assert!((1..=4).contains(&node.tier), "authored tier: {name}");
        match node.status {
            ContentStatus::Connected => {
                for prerequisite in &node.prereqs {
                    let required = defs.get(*prerequisite).expect("prerequisite");
                    assert_eq!(
                        required.status,
                        ContentStatus::Connected,
                        "{name} cannot discover through inert {}",
                        required.name
                    );
                }
            }
            ContentStatus::Inert => {
                assert!(node.debt.as_deref().is_some_and(|debt| !debt.is_empty()));
                assert!(node.gates.is_empty(), "inert runtime gate: {name}");
            }
        }
    }
    let taming = defs
        .get(defs.id_of("animal_taming").expect("taming id"))
        .expect("taming node");
    assert_eq!(taming.marker_weight, 2);

    // Reuse the spine validator: invalid connected references fail, and uncaused nodes fail.
    let source = catalog.source_text("knowledge.ron").expect("authored map");
    let missing_recipe = source.replace("Recipe(\"make_bow\")", "Recipe(\"missing_c1_bow\")");
    assert_ne!(source, missing_recipe);
    assert!(knowledge::validate(&missing_recipe, &catalog).is_err());
    let missing_cause = source.replace(
        "discover: [Knows(\"cultivation\", 300), Stockpile(\"rye\", 2)]",
        "discover: []",
    );
    assert_ne!(source, missing_cause);
    assert!(knowledge::validate(&missing_cause, &catalog).is_err());
}

#[test]
fn c1_connected_recipes_require_their_actual_practitioner_bits() {
    let catalog = Catalog::from_sources(RawSources::bundled(), None).expect("C1 catalogue");
    let defs = catalog.knowledge_defs();
    let gates = catalog.knowledge_gates();
    let mut checked = 0;
    for name in [
        "dairy",
        "querns",
        "oven",
        "fermentation",
        "food_preservation",
        "hoe",
        "sickle",
        "adze",
        "dyeing",
        "leather_armor",
        "bow",
    ] {
        let node = defs.get(defs.id_of(name).expect(name)).expect(name);
        assert_eq!(node.status, ContentStatus::Connected, "{name}");
        let recipes: Vec<_> = node
            .gates
            .iter()
            .filter_map(|gate| match gate {
                Gate::Recipe(recipe) => Some(*recipe),
                _ => None,
            })
            .collect();
        assert!(!recipes.is_empty(), "executable recipe: {name}");
        for recipe in recipes {
            assert_ne!(gates.recipe(recipe) & bit(node.id), 0, "{name}");
            checked += 1;
        }
    }
    assert_eq!(checked, 15, "all authored recipe gates covered");
    let bow = defs.id_of("bow").expect("bow");
    assert!(bow >= 64, "C1 requires the expanded practitioner mask");
    assert_ne!(bit(bow), 0);
    assert_eq!(bit(bow) & bit(bow - 64), 0, "high ids never alias");
}

#[test]
fn c1_partial_content_debt_keeps_connected_effects_distinct_from_inert() {
    let catalog = Catalog::from_sources(RawSources::bundled(), None).expect("C1 catalogue");
    let rendered = sim_core::colony::content_debt::render(&catalog);
    for (id, kind, status, pass) in [
        ("hoe", "knowledge", "Connected", "M-BAL"),
        ("hoe", "item", "Connected", "M-BAL"),
        ("animal_taming", "knowledge", "Connected", "C1-b"),
        ("dairy", "knowledge", "Connected", "C1-b"),
        ("barter", "knowledge", "Inert", "M7"),
        ("herbalism", "knowledge", "Inert", "M9"),
        ("wild_boar", "animal", "Inert", "M-BAL"),
    ] {
        let prefix = format!("| {id} | {kind} | ");
        let rows: Vec<_> = rendered
            .lines()
            .filter(|line| line.starts_with(&prefix))
            .collect();
        assert_eq!(rows.len(), 1, "one authored debt row: {kind}/{id}");
        assert!(
            rows[0].contains(&format!(" | {status} | ")),
            "truthful effect status: {kind}/{id}"
        );
        assert!(
            rows[0].ends_with(&format!(" | {pass} |")),
            "specific planned pass: {kind}/{id}"
        );
    }
    let boar = rendered
        .lines()
        .find(|line| line.starts_with("| wild_boar | animal | "))
        .expect("authored animal fallback debt");
    assert!(
        boar.contains("seed5") && boar.contains("starvation"),
        "fallback cause remains visible: {boar}"
    );
    assert!(
        !rendered
            .lines()
            .any(|line| line.starts_with("| querns | knowledge | ")),
        "fully connected querns recipes has no fabricated debt"
    );
}
