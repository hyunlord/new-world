//! M-SPINE data validation, authored lanes and high-node identity regressions.
use super::*;
use crate::colony::content::{Catalog, NamedSource, RawSources};

#[test]
fn spine_knowledge_validator() {
    // Given an installed-size map with one additional authored node.
    let base = RawSources::bundled();
    let node = |status: &str, discover: &str| {
        format!(
        "(schema_version:1, absent_markers:[], knowledge:[(id:\"spine_probe\",lane:\"tools\",era:\"iron\",tier:4,status:{status},debt:Some(\"C3: missing station\"),threshold:300,prereqs:[],difficulty:1,discover:{discover},gates:[Recipe(\"future_smelting\")],marker_weight:0,seed:None,name_key:\"SPINE_PROBE\")])"
    )
    };
    let selected = |text: String| {
        Catalog::from_sources(
            base.clone(),
            Some(RawSources {
                sources: vec![NamedSource {
                    name: "knowledge.ron".into(),
                    bytes: text.into_bytes(),
                }],
            }),
        )
    };
    // When a connected gate dangles it is rejected; inert data keeps it as debt.
    assert!(selected(node("Connected", "[Practice(job_kind:\"craft\",count:1)]")).is_err());
    let wrong_workshop = node("Connected", "[Practice(job_kind:\"craft\",count:1)]")
        .replace("Recipe(\"future_smelting\")", "Workshop(\"log_wall\")");
    assert!(
        selected(wrong_workshop).is_err(),
        "workshop gates need a workshop completion"
    );
    let inert =
        selected(node("Inert", "[Practice(job_kind:\"craft\",count:1)]")).expect("inert node");
    // Then no unresolved inert gate can enable or block a runtime recipe.
    let added = inert.knowledge_defs().nodes.last().expect("node");
    assert_eq!(added.status, ContentStatus::Inert);
    assert!(added.gates.is_empty());
    assert_eq!(added.inert_gates, ["Recipe(\"future_smelting\")"]);
    assert!(
        selected(node("Inert", "[]")).is_err(),
        "non-seeded content needs a cause"
    );
}

#[test]
fn spine_high_node_identity_does_not_alias_a_seeded_entry() {
    // Given two nodes whose historical five-bit ids would alias.
    let old = KnowledgeEntry::new(3, Source::Seeded, 600, 0);
    let high = KnowledgeEntry::new(35, Source::Migrant, 600, 0);
    // When packed entries are constructed, both id and source survive.
    assert_eq!((old.id(), old.source()), (3, Source::Seeded));
    assert_eq!((high.id(), high.source()), (35, Source::Migrant));
    // Then masks and personal-entry lookup keep the nodes independent.
    assert_eq!(bit(3) & bit(35), 0);
    assert_eq!(ids_of(bit(3) | bit(35)).collect::<Vec<_>>(), [3, 35]);
    assert_eq!(bit(64), 0, "foreign ids never wrap onto a live node");
}

#[test]
fn spine_eras_from_data() {
    // Given thresholds authored differently from the bundled 250/750 ladder.
    let text = include_str!("../../../data/colony/eras.ron")
        .replace("transition_permille: 250", "transition_permille: 100")
        .replace("era_permille: 750", "era_permille: 600");
    let eras = crate::colony::eras::EraCatalog::parse(&text).expect("eras");
    // When practitioners' coverage advances and then disappears, classification regresses.
    assert_eq!(eras.current(&[0, 1000, 0, 0]), (1, 2));
    assert_eq!(eras.current(&[0, 1000, 100, 0]), (2, 1));
    assert_eq!(eras.current(&[0, 1000, 600, 0]), (2, 2));
    assert_eq!(eras.current(&[0, 0, 0, 0]), (0, 2));
    assert_eq!(
        eras.current(&[1000, 0, 300, 0]),
        (0, 2),
        "transition cannot skip the next era"
    );
    // Then legacy names resolve through data to the canonical lanes.
    assert_eq!(eras.lane_order("craft"), eras.lane_order("tools"));
    assert_eq!(eras.lane_order("mine"), eras.lane_order("tools"));
}
