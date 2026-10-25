//! Existing knowledge regression tests, retained under the original module path.
use super::*;
use crate::colony::{BlueprintKind, Designation, ItemKind, RecipeKind};
const KNOWLEDGE_RON: &str = include_str!("../../../data/colony/knowledge.ron");

#[test]
fn the_map_parses_to_24_nodes_with_an_acyclic_prereq_graph() {
    let d = defs();
    assert_eq!(d.nodes.len(), 24);
    assert!(d.nodes.len() <= MAX_NODES);
    // Every prereq points at another node, and the graph has no cycle
    // (a depth-first search from each node never revisits its path).
    fn cyclic(d: &KnowledgeDefs, at: KnowledgeId, path: &mut Vec<KnowledgeId>) -> bool {
        if path.contains(&at) {
            return true;
        }
        path.push(at);
        let bad = d.nodes[usize::from(at)]
            .prereqs
            .iter()
            .any(|p| cyclic(d, *p, path));
        path.pop();
        bad
    }
    for n in &d.nodes {
        assert!(
            !cyclic(d, n.id, &mut Vec::new()),
            "{} is in a cycle",
            n.name
        );
        assert!(!n.prereqs.contains(&n.id));
    }
    // Spot values from the goal table (3.1).
    let knap = id("flint_knapping").and_then(|i| d.get(i)).cloned();
    assert_eq!(
        knap.as_ref().map(|k| (k.threshold, k.seed)),
        Some((300, Seeding::Count(3)))
    );
    let fire = id("pot_firing").and_then(|i| d.get(i)).cloned();
    assert_eq!(
        fire.map(|k| (k.marker_weight, k.era, k.discover)),
        Some((
            3,
            Era::Neolithic,
            vec![Cond::Stockpile(ItemKind::RawPot, 2)]
        ))
    );
    // Markers 3 + 3 + 2 + 1 present, farming 3 and weaving 2 absent: 14.
    assert_eq!(d.marker_total(), 14);
    let present: u32 = d.nodes.iter().map(|n| u32::from(n.marker_weight)).sum();
    assert_eq!(present, 9);
    assert!(parse_defs("(schema_version: 2, absent_markers: [], knowledge: [])").is_none());
    let bad = KNOWLEDGE_RON.replace("Recipe(\"haft_axe\")", "Recipe(\"bronze\")");
    assert!(parse_defs(&bad).is_none());
    let dup = KNOWLEDGE_RON.replace("(id: \"foraging\"", "(id: \"fire_keeping\"");
    assert!(parse_defs(&dup).is_none());
}

#[test]
fn gate_masks_cover_the_goal_table() {
    let g = gates();
    let b = |name: &str| id(name).map_or(0, bit);
    assert_eq!(g.recipe(RecipeKind::KnapBlade), b("flint_knapping"));
    assert_eq!(g.recipe(RecipeKind::HaftAxe), b("hafting"));
    assert_eq!(g.recipe(RecipeKind::FirePot), b("pot_firing"));
    assert_eq!(g.build(BlueprintKind::Door), b("doorway"));
    assert_eq!(g.build(BlueprintKind::KnappingSpot), b("flint_knapping"));
    assert_eq!(g.build(BlueprintKind::Hut), b("hut_building"));
    assert_eq!(g.designation(Designation::Quarry), b("stone_quarrying"));
    assert_eq!(g.designation(Designation::Dig), b("digging"));
    assert_eq!(g.designation(Designation::ClayPit), b("clay_digging"));
    assert_eq!(g.designation(Designation::Chop), 0);
    assert_eq!(g.apprenticeship(), b("instruction"));
    assert_eq!(g.lore(Lore::Hearth), b("fire_keeping"));
}

#[test]
fn entries_stay_sorted_and_the_bitset_follows_the_thresholds() {
    let knap = id("flint_knapping").unwrap_or(0);
    let door = id("doorway").unwrap_or(0);
    let mut k = Knowledge::default();
    k.set(knap, 299, Source::Oral, 5);
    k.set(door, 200, Source::Seeded, 6);
    assert_eq!(k.entries().iter().map(|e| e.id()).collect::<Vec<_>>(), {
        let mut v = vec![knap, door];
        v.sort_unstable();
        v
    });
    assert!(!k.practises(knap) && k.practises(door));
    assert!(k.raise_to(knap, 350, Source::Oral, 7));
    assert!(!k.raise_to(knap, 300, Source::Oral, 8), "never lowers");
    assert!(k.practises(knap));
    // Days: tick 100 is day 0; tick 1,440 × d is day d.
    let day = |d: u64| d * DAY_TICKS;
    k.use_mask(bit(knap), 2, day(10));
    assert_eq!(
        (k.proficiency(knap), k.get(knap).map(|e| e.last_used)),
        (352, Some(10))
    );
    // Forgetting: unused > 3 days, oral 2 ‰ a step to 0, seeded 1 ‰ to 100.
    let rate = |s: Source| if s == Source::Oral { 2 } else { 1 };
    let floor = |s: Source| if s == Source::Seeded { 100 } else { 0 };
    // On day 13 only the doorway (last used on day 0) fades; knapping was used on day 10.
    assert!(k.decay(day(13), 3, rate, floor));
    assert_eq!((k.proficiency(knap), k.proficiency(door)), (352, 199));
    assert!(!k.practises(door), "199 is below the doorway's 200");
    assert!(k.decay(day(14), 3, rate, floor));
    assert_eq!((k.proficiency(knap), k.proficiency(door)), (350, 198));
    for t in 0..400 {
        k.decay(day(20) + t, 3, rate, floor);
    }
    assert_eq!(k.get(knap), None, "an oral entry at 0 is removed");
    assert_eq!(
        k.proficiency(door),
        100,
        "a seeded entry stops at its floor"
    );
    assert_eq!(k.entries().len(), k.entries().iter().count());
    assert_eq!(std::mem::size_of::<KnowledgeEntry>(), 6);
    assert!(std::mem::size_of::<Knowledge>() <= 24);
    // Observation stops at its cap.
    assert!(k.observe(knap, 5, 250, 900));
    for _ in 0..100 {
        k.observe(knap, 5, 250, 901);
    }
    assert_eq!(k.proficiency(knap), 250);
    assert_eq!(
        k.get(knap).map(|e| (e.source(), e.observed())),
        Some((Source::Observed, 250))
    );
}

#[test]
fn a_load_rebuilds_the_bitset_from_the_entries() {
    let knap = id("flint_knapping").expect("knapping");
    let mut k = Knowledge::default();
    k.set(knap, 600, Source::Seeded, 0);
    let text = ron::to_string(&k).expect("save");
    // A save whose bitset disagrees with its entries (a threshold changed since).
    let tampered = text.replace(&format!("practice:{}", k.practice()), "practice:0");
    assert_ne!(tampered, text, "the bitset is in the saved form");
    let loaded: Knowledge = ron::from_str(&tampered).expect("load");
    assert_eq!(loaded.practice(), bit(knap));
    assert_eq!(loaded, k);
}

#[test]
fn from_entries_sorts_keeps_the_last_of_a_node_and_sets_the_bitset() {
    let knap = id("flint_knapping").expect("knapping");
    let fire = id("fire_keeping").expect("fire");
    let door = id("doorway").expect("doorway");
    let k = Knowledge::from_entries(vec![
        KnowledgeEntry::new(door, Source::Seeded, 150, 0),
        KnowledgeEntry::new(knap, Source::Seeded, 100, 0),
        KnowledgeEntry::new(fire, Source::Seeded, 600, 0),
        KnowledgeEntry::new(knap, Source::Oral, 350, 2),
    ]);
    let ids: Vec<KnowledgeId> = k.entries().iter().map(KnowledgeEntry::id).collect();
    let mut sorted = ids.clone();
    sorted.sort_unstable();
    assert_eq!(ids, sorted, "sorted by id");
    assert_eq!(ids.len(), 3, "one entry per node");
    // The later knapping entry wins: 350 ‰ (threshold 300), heard.
    assert_eq!(
        k.get(knap).map(|e| (e.proficiency, e.source())),
        Some((350, Source::Oral))
    );
    // Practises knapping (350 ≥ 300) and fire keeping (600 ≥ 300), not the doorway (150 < 200).
    assert_eq!(k.practice(), bit(knap) | bit(fire));
    // The same as entry-by-entry `set`.
    let mut by_set = Knowledge::default();
    by_set.set(door, 150, Source::Seeded, 0);
    by_set.set(fire, 600, Source::Seeded, 0);
    by_set.set(knap, 350, Source::Oral, 2 * DAY_TICKS);
    assert_eq!(k, by_set);
}

#[test]
fn quality_caps_and_era_labels_follow_their_tables() {
    assert_eq!(
        (
            quality_cap(0),
            quality_cap(299),
            quality_cap(350),
            quality_cap(1000)
        ),
        (0, 35, 45, 100)
    );
    assert!(QUAL_CAP.windows(2).all(|w| w[0] < w[1]));
    assert_eq!(
        (
            era_label(249),
            era_label(250),
            era_label(749),
            era_label(750)
        ),
        (0, 1, 1, 2)
    );
    let codes: Vec<u8> = Source::ALL.iter().map(|s| s.code()).collect();
    assert_eq!(codes, vec![0, 1, 2, 3, 4, 5]);
    for s in Source::ALL {
        assert_eq!(Source::from_code(s.code()), s);
    }
    // M5.5: a migrant's entry fades and floors as an apprenticed one.
    assert_eq!(
        Source::Migrant.table_index(),
        Source::Apprenticed.table_index()
    );
    let e = KnowledgeEntry::new(17, Source::Migrant, 500, 3);
    assert_eq!(
        (e.id(), e.source(), e.proficiency),
        (17, Source::Migrant, 500)
    );
    let lanes: Vec<u8> = Lane::ALL.iter().map(|l| l.code()).collect();
    assert_eq!(lanes, vec![0, 1, 2, 3, 4, 5]);
}
