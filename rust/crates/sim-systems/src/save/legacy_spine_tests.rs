//! Frozen predecessor and current-format M-SPINE save boundary tests.

use super::*;
use sim_core::colony::knowledge::{Knowledge, Source};

#[test]
fn spine_old_knowledge_column_preserves_source_observation_and_suffix() {
    sim_core::colony::content::bootstrap(sim_core::colony::content::BootSelection::Bundled)
        .expect("bundled catalog");
    let old = vec![KnowledgeV17 {
        entries: vec![
            EntryV17 {
                last_used: 71,
                proficiency: 500,
                tag: 3 | (3 << 5),
                observed5: 17,
            },
            EntryV17 {
                last_used: 91,
                proficiency: 700,
                tag: 30 | (5 << 5),
                observed5: 0,
            },
        ],
        practice: u32::MAX,
    }];
    let bytes = knowledge_column(&super::super::encode(&old).expect("old bytes"))
        .expect("column migration");
    let current: Vec<Knowledge> = super::super::decode(&bytes).expect("current knowledge");
    let entry = current[0].get(3).expect("old node retained");
    assert_eq!(
        (entry.last_used, entry.proficiency, entry.observed()),
        (71, 500, 85)
    );
    assert_eq!(entry.source(), Source::Observed);
    assert_eq!(
        current[0].get(30).expect("high old id").source(),
        Source::Migrant
    );
    assert!(!current[0].practises(0), "untrusted cached mask rebuilt");
}

#[test]
fn spine_old_skills_preserve_xp_without_inventing_completed_jobs() {
    use sim_core::colony::{skills::Skill, JobKind, SkillKind, Skills};
    let mut entries = [Skill::default(); SkillKind::ALL.len()];
    entries[0] = Skill { level: 4, xp: 300 };
    let bytes = super::super::encode(&vec![(entries,)]).expect("old skill bytes");
    let current: Vec<Skills> =
        super::super::decode(&skills_column(&bytes).expect("migrate")).expect("current skills");
    assert_eq!(current[0].entries, entries);
    assert_eq!(current[0].completed, [0u32; JobKind::NAMES.len()]);
}

#[test]
fn spine_save_roundtrip_keeps_high_ids_era_vector_and_practice_counts() {
    use sim_core::colony::{JobKind, Skills};
    sim_core::colony::content::bootstrap(sim_core::colony::content::BootSelection::Bundled)
        .expect("bundled catalog");
    let mut engine = sim_engine::SimEngine::new(4, 4, sim_core::MaterialRegistry::new());
    let mut knowledge = Knowledge::default();
    knowledge.set(42, 777, Source::Discovered, 100);
    let mut skills = Skills::default();
    skills.complete(JobKind::Build { site: (1, 2) });
    let person = engine.world.spawn((knowledge.clone(), skills));
    engine.resources.colony.tech.era_progress = vec![1000, 500, 0, 0];
    engine.resources.colony.tech.current_era = (1, 1);
    let bytes = super::super::save(&mut engine).expect("current save");
    let mut loaded = super::super::load_with(&bytes, |_| {}).expect("reload");
    assert_eq!(
        *loaded.world.get::<&Knowledge>(person).expect("knowledge"),
        knowledge
    );
    assert_eq!(
        *loaded.world.get::<&Skills>(person).expect("skills"),
        skills
    );
    assert_eq!(loaded.resources.colony.tech.era_progress, [1000, 500, 0, 0]);
    assert_eq!(loaded.resources.colony.tech.current_era, (1, 1));
    assert_eq!(super::super::save(&mut loaded).expect("stable save"), bytes);
}

#[test]
fn spine_widened_mask_keeps_all_predecessor_bytes() {
    for value in [0, 250, 251, 65_535, u32::MAX] {
        assert_eq!(
            super::super::encode(&value).expect("u32"),
            super::super::encode(&u64::from(value)).expect("u64")
        );
    }
}

#[test]
fn spine_catalog_admission_rejects_foreign_revision() {
    let catalog = Catalog::from_sources(RawSources::bundled(), None).expect("catalog");
    assert!(!accepts_catalog("unknown-bundle", &catalog).expect("rejected"));
    assert!(accepts_catalog(REVISION, &catalog).expect("pinned baseline"));
}

#[test]
fn spine_legacy_aggregate_preserves_following_field_alignment() {
    use sim_core::colony::{knowledge::TechAggregate, legacy_v7::LegacyTechAggregate};
    let bytes = super::super::encode(&(
        Vec::<sim_core::colony::knowledge::TechState>::new(),
        643u16,
        11u16,
        27u64,
        65537u64,
    ))
    .expect("exact historical fields and suffix");
    let (old, suffix): (LegacyTechAggregate, u64) = super::super::decode(&bytes)
        .expect("frozen aggregate does not consume the following field");
    let current: TechAggregate = old.into();
    assert_eq!(
        (
            current.progress_permille,
            current.adults,
            current.version,
            suffix
        ),
        (643, 11, 27, 65537)
    );
    assert!(
        current.era_progress.is_empty(),
        "historical saves cannot invent era measurements"
    );
}

#[test]
fn spine_v17_revision_is_admitted_before_body_decoding_only_for_pinned_bundle() {
    sim_core::colony::content::bootstrap(sim_core::colony::content::BootSelection::Bundled)
        .expect("bundled catalog");
    for (revision, rejected_catalog) in [("foreign-v17-pack", true), (REVISION, false)] {
        let envelope = super::super::Envelope {
            revision: revision.into(),
            payload: vec![255],
        };
        let mut bytes = super::super::MAGIC.to_vec();
        bytes.extend_from_slice(&17u32.to_le_bytes());
        bytes.extend(super::super::encode(&envelope).expect("envelope"));
        let error = super::super::load(&bytes)
            .err()
            .expect("invalid body must fail")
            .to_string();
        assert_eq!(error.contains("does not match installed"), rejected_catalog);
    }
}
