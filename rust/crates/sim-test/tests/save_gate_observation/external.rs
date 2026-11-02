//! External loaded prefixes have unknown founding census until independently authenticated.
use super::{write_json, Watch};
use serde_json::{json, Value};
use sim_engine::SimEngine;
impl Watch {
    pub fn external(test: &str, branch: &str, engine: &SimEngine, bytes: &[u8]) -> Self {
        let mut watch = Self::begin(test, engine.resources.world_seed, branch, engine);
        if let Some(active) = watch.0.as_mut() {
            active.founding = Value::Null;
            active.prefix = Some(
                json!({"kind": "external_legacy_save", "founding_census": null,
                "external_hash_authentication_required": true}),
            );
        }
        let link = watch.save(engine, bytes, "external_input");
        if let (Some(active), Some(link)) = (watch.0.as_mut(), link.0) {
            active.prefix = Some(
                json!({"kind": "external_legacy_save", "save_artifact": link.artifact,
                "checkpoint": link.checkpoint, "founding_census": null,
                "external_hash_authentication_required": true}),
            );
            write_json(
                &active.directory.join("external-prefix.json"),
                &active.prefix.clone().expect("prefix"),
            );
        }
        watch
    }

    pub fn retain_file(&self, path: &std::path::Path, label: &str) {
        let Some(active) = &self.0 else {
            return;
        };
        use std::io::Write;
        assert!(label
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || c == b'_'));
        let bytes = std::fs::read(path).expect("actual producer input");
        let output = active.directory.join(format!("input_{label}.bin"));
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&output)
            .expect("fresh input evidence");
        file.write_all(&bytes).expect("retain exact input bytes");
        file.sync_all().expect("sync input");
        write_json(
            &active.directory.join(format!("input_{label}.json")),
            &json!({"source_path": path,
            "artifact": output, "byte_count": bytes.len(), "external_sha256_required": true}),
        );
    }
}
