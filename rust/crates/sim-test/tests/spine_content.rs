//! Authored debt must remain synchronized with the resolved catalogue.
use sim_core::colony::content::{Catalog, RawSources};

#[test]
fn content_debt_matches_data() {
    // Given the same resolved data used by the live game.
    let catalog = Catalog::from_sources(RawSources::bundled(), None).expect("valid bundled data");
    // When the ledger is generated.
    let generated = sim_core::colony::content_debt::render(&catalog);
    // Then every authored inert row is represented exactly once, preserving manual notes.
    let ledger = include_str!("../../../../docs/v8/CONTENT_DEBT.md");
    let (_, actual) = ledger
        .split_once("<!-- generated-content-debt -->")
        .expect("generated ledger section");
    assert_eq!(
        format!("<!-- generated-content-debt -->{actual}"),
        generated
    );
}
