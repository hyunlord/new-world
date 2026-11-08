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
    let (manual, actual) = ledger
        .split_once("<!-- generated-content-debt -->")
        .expect("generated ledger section");
    assert!(manual.contains("D-S3-STOP"));
    assert!(manual.contains("Stage3 is cancelled."));
    assert_eq!(
        format!("<!-- generated-content-debt -->{actual}"),
        generated
    );
}

#[test]
fn content_debt_covers_each_authored_template_debt_once_in_order() {
    use sim_core::colony::knowledge::ContentStatus;
    let catalog = Catalog::from_sources(RawSources::bundled(), None).expect("valid bundled data");
    let generated = sim_core::colony::content_debt::render(&catalog);
    let template_rows: Vec<&str> = generated
        .lines()
        .filter(|line| line.contains(" | template | "))
        .collect();
    let indebted: Vec<_> = catalog
        .building_templates()
        .buildings
        .iter()
        .filter(|template| {
            template.status == ContentStatus::Inert
                || template
                    .debt
                    .as_deref()
                    .is_some_and(|debt| !debt.trim().is_empty())
        })
        .collect();
    assert!(
        !indebted.is_empty(),
        "bundled future templates are intentionally authored"
    );
    assert_eq!(
        template_rows.len(),
        indebted.len(),
        "no missing or duplicate template row"
    );
    for (line, template) in template_rows.iter().zip(indebted) {
        let prefix = format!(
            "| {} | template | {} | building | ",
            template.id, template.era
        );
        assert!(
            line.starts_with(&prefix),
            "authored template order/era/lane: {line}"
        );
        let status = match template.status {
            ContentStatus::Connected => "Connected",
            ContentStatus::Inert => "Inert",
        };
        assert!(
            line.contains(&format!(" | {status} | ")),
            "template status: {line}"
        );
        let debt = template.debt.as_deref().expect("authored template debt");
        let escaped = debt.replace('|', "\\|").replace(['\n', '\r'], " ");
        assert!(
            line.contains(&escaped),
            "missing authored behavior debt: {line}"
        );
        let planned = line.rsplit('|').nth(1).expect("planned pass cell").trim();
        assert!(!planned.is_empty(), "missing planned pass: {line}");
        assert!(
            planned == "후속 콘텐츠 패스" || debt.contains(planned),
            "planned pass must come from authored debt: {line}",
        );
    }
    for template in &catalog.building_templates().buildings {
        if template.status == ContentStatus::Connected && template.debt.is_none() {
            assert!(
                !generated.contains(&format!("| {} | template | ", template.id)),
                "debt-free connected template has no debt row: {}",
                template.id,
            );
        }
    }
}

#[test]
fn content_debt_covers_reserved_zones_once_without_activating_them() {
    use sim_core::colony::knowledge::ContentStatus;
    let catalog = Catalog::from_sources(RawSources::bundled(), None).expect("valid bundled data");
    let generated = sim_core::colony::content_debt::render(&catalog);
    let rows: Vec<_> = generated
        .lines()
        .filter(|line| line.contains(" | zone | "))
        .collect();
    let zones = catalog.building_templates().zone_kinds();
    let inert: Vec<_> = zones
        .iter()
        .filter(|zone| zone.status == ContentStatus::Inert)
        .collect();
    assert_eq!(
        inert
            .iter()
            .map(|zone| zone.id.as_str())
            .collect::<Vec<_>>(),
        ["storage", "workshop"]
    );
    assert_eq!(rows.len(), inert.len());
    for (row, zone) in rows.iter().zip(inert) {
        assert!(row.starts_with(&format!("| {} | zone |  | building | ", zone.id)));
        let debt = zone.debt.as_deref().expect("validated zone debt");
        let escaped = debt.replace('|', "\\|").replace(['\n', '\r'], " ");
        assert!(row.contains(&escaped));
        assert!(!row
            .rsplit('|')
            .nth(1)
            .expect("planned pass")
            .trim()
            .is_empty());
    }
    for zone in zones
        .iter()
        .filter(|zone| zone.status == ContentStatus::Connected)
    {
        assert!(!generated.contains(&format!("| {} | zone | ", zone.id)));
    }
}
