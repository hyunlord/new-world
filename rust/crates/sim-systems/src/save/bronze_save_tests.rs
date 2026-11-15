//! Published C2-b format23 admission under a C3-appended catalog.
use super::*;

#[test]
fn format23_requires_exact_published_revision_and_preserved_c2_suffix() {
    let catalog = Catalog::from_sources(RawSources::bundled(), None).expect("catalog");
    assert!(identities_preserved(&catalog));
    assert!(accepts_catalog(REVISION, &catalog).expect("genuine published revision"));
    assert!(!accepts_catalog("unknown23", &catalog).expect("unknown revision"));
    assert!(
        !accepts_catalog(&REVISION.replace("231728", "231729"), &catalog)
            .expect("near-miss revision")
    );
    let mut sources = RawSources::bundled();
    let source = sources
        .sources
        .iter_mut()
        .find(|source| source.name == "items.ron")
        .expect("items source");
    let text = std::str::from_utf8(&source.bytes).expect("utf8");
    source.bytes = text
        .replace(
            "serde_name: \"Limestone\"",
            "serde_name: \"OtherLimestone\"",
        )
        .into_bytes();
    let changed = Catalog::from_sources(sources, None).expect("otherwise valid edited identity");
    assert!(
        !identities_preserved(&changed),
        "the C2 suffix is part of authenticated history"
    );
}

#[test]
fn genuine_c2b_save_loads_resaves_stably_and_continues() {
    let Ok(path) = std::env::var("C3_C2B_SAVE_PATH") else {
        return;
    };
    sim_core::colony::content::bootstrap(sim_core::colony::content::BootSelection::Bundled)
        .expect("bundled catalog");
    // Given the published C2-b binary's actual format23 output.
    let bytes = std::fs::read(path).expect("genuine C2-b fixture");
    assert_eq!(&bytes[8..12], &23_u32.to_le_bytes());
    let envelope: super::super::Envelope =
        super::super::decode(super::super::read_header(&bytes).expect("header")).expect("envelope");
    assert_eq!(envelope.revision, REVISION);
    let old: super::super::Body = super::super::decode(&envelope.payload).expect("genuine body");
    // When the production loader authenticates it against the installed catalog.
    let mut restored = super::super::load(&bytes).expect("genuine23 load");
    // Then history survives, the re-save is stable, and twins continue identically.
    assert_eq!(restored.current_tick(), old.current_tick);
    assert_eq!(
        super::super::encode(&restored.resources.chronicle).expect("restored history"),
        super::super::encode(&old.resources.chronicle).expect("original history")
    );
    let current = super::super::save(&mut restored).expect("current23 save");
    assert_eq!(&current[8..12], &super::super::FORMAT_VERSION.to_le_bytes());
    let mut roundtrip = super::super::load(&current).expect("current23 load");
    assert_eq!(
        super::super::save(&mut roundtrip).expect("immediate23 save"),
        current
    );
    for _ in 0..60 {
        restored.tick();
        roundtrip.tick();
    }
    assert_eq!(
        super::super::save(&mut restored).expect("continued genuine"),
        super::super::save(&mut roundtrip).expect("continued roundtrip")
    );
}
