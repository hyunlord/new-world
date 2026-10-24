//! Build script (M-ART2, MART2_DECISIONS): embeds every material definition file under
//! `rust/crates/sim-core/data/material/**/*.ron` into the bridge, so `ffi::material_category` can name the
//! registry category of any `MaterialId` the tile grid holds without reading the disk at run time (Godot
//! loads the library from anywhere). A material file added under that directory is picked up by data
//! alone (ADR 0033): the generated list is rebuilt whenever the directory changes.
//!
//! Output: `$OUT_DIR/material_files.rs` with `MATERIAL_FILES: &[(&str, &str)]` — (path relative to the
//! material directory, file text), sorted by path so the list is the same on every machine.
use std::error::Error;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

fn ron_files(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), Box<dyn Error>> {
    for entry in std::fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            ron_files(&path, out)?;
        } else if path.extension().is_some_and(|e| e == "ron") {
            out.push(path);
        }
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let manifest = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR")?);
    let dir = manifest.join("../sim-core/data/material").canonicalize()?;
    println!("cargo:rerun-if-changed={}", dir.display());
    println!("cargo:rerun-if-changed=build.rs");
    let mut files = Vec::new();
    ron_files(&dir, &mut files)?;
    files.sort();
    let mut code = String::from(
        "/// Every sim-core material definition file: (path under `data/material`, text), by path.\n\
         pub(crate) const MATERIAL_FILES: &[(&str, &str)] = &[\n",
    );
    for file in &files {
        let rel = file
            .strip_prefix(&dir)?
            .to_string_lossy()
            .replace('\\', "/");
        let abs = file.to_string_lossy();
        writeln!(code, "    ({rel:?}, include_str!({abs:?})),")?;
    }
    code.push_str("];\n");
    let out = PathBuf::from(std::env::var("OUT_DIR")?).join("material_files.rs");
    std::fs::write(out, code)?;
    Ok(())
}
