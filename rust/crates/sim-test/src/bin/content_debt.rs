//! Regenerate the authored inert-content ledger without changing manual debt notes.
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let catalog = sim_core::colony::content::Catalog::from_sources(
        sim_core::colony::content::RawSources::bundled(),
        None,
    )?;
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../docs/v8/CONTENT_DEBT.md");
    let existing = std::fs::read_to_string(&path)?;
    let prefix = existing
        .split("<!-- generated-content-debt -->")
        .next()
        .unwrap_or(&existing);
    std::fs::write(
        path,
        format!(
            "{}\n{}",
            prefix.trim_end(),
            sim_core::colony::content_debt::render(&catalog)
        ),
    )?;
    Ok(())
}
