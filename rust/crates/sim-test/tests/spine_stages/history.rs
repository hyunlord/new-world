//! Immutable generation history assertions, shared by the one canonical chain.
use super::*;

pub(super) fn check(
    engine: &mut SimEngine,
    id: u32,
    original: &sim_core::colony::building_site::BuildingSite,
) {
    let catalog = sim_core::colony::content::installed_or_bundled();
    let log_value = i64::try_from(
        catalog
            .building_templates()
            .buildings
            .iter()
            .position(|template| template.id == "log_house")
            .expect("log template"),
    )
    .expect("small catalogue");
    let stone_value = i64::try_from(
        catalog
            .building_templates()
            .buildings
            .iter()
            .position(|template| template.id == "stone_house")
            .expect("stone template"),
    )
    .expect("small catalogue");
    let original_entry = original.completion_entry.expect("original completion");
    assert_eq!(
        engine.resources.chronicle.entries()[original_entry as usize].value,
        log_value,
        "old log house completion retains its name after current site becomes stone"
    );
    let new = &engine.resources.colony.buildings.sites[&id];
    assert_ne!(new.completion_entry, original.completion_entry);
    let changed = original
        .pieces
        .iter()
        .zip(&new.pieces)
        .filter(|(old, new)| old.kind != new.kind)
        .count();
    assert_eq!(
        changed, 11,
        "only ring walls changed; door and furnishings retained"
    );
    assert_eq!(
        engine.resources.chronicle.cause_of(original_entry),
        Some(CauseRef::Building {
            site: id,
            record: original.source_record,
            origin: original.cause,
        }),
        "old Zone origin remains immutable after direct upgrade"
    );
    let upgraded_entry = new.completion_entry.expect("upgraded completion");
    assert_eq!(
        engine.resources.chronicle.entries()[upgraded_entry as usize].value,
        stone_value
    );
    assert_eq!(
        engine.resources.chronicle.cause_of(upgraded_entry),
        Some(CauseRef::Building {
            site: id,
            record: new.source_record,
            origin: BuildCause::Command,
        })
    );
    let bytes = sim_systems::save::save(engine).expect("save upgrade provenance");
    let loaded = sim_systems::save::load(&bytes).expect("load upgrade provenance");
    assert_eq!(
        loaded.resources.chronicle.cause_of(original_entry),
        Some(CauseRef::Building {
            site: id,
            record: original.source_record,
            origin: original.cause,
        })
    );
    assert_eq!(
        loaded.resources.chronicle.entries()[original_entry as usize].value,
        log_value
    );
    assert_eq!(
        loaded.resources.chronicle.entries()[upgraded_entry as usize].value,
        stone_value
    );
}
