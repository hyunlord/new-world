//! Opt-in read-only observations for explicitly named ordinary integration worlds.
//! External harness hashes saved bytes and authenticates invocation/source lineage.
use serde_json::{json, Value};
use sim_core::chronicle::ChronicleKind;
use sim_core::components::{Agent, BodyHealth};
use sim_engine::SimEngine;
use std::path::PathBuf;
mod external;

pub struct Watch(Option<Active>);
struct Active {
    directory: PathBuf,
    initial: Value,
    founding: Value,
    prefix: Option<Value>,
}

#[derive(Clone)]
pub struct SaveLink(Option<Link>);
#[derive(Clone)]
struct Link {
    artifact: PathBuf,
    checkpoint: PathBuf,
    state: Value,
    founding: Value,
}

fn snapshot(engine: &SimEngine) -> Value {
    let mut raw: Vec<u64> = engine
        .world
        .query::<&Agent>()
        .iter()
        .map(|(_, agent)| agent.id)
        .collect();
    raw.sort_unstable();
    let mut alive: Vec<u64> = engine
        .world
        .query::<(&Agent, Option<&BodyHealth>)>()
        .iter()
        .filter(|(_, (_, health))| health.is_none_or(|h| !h.is_dead()))
        .map(|(_, (agent, _))| agent.id)
        .collect();
    alive.sort_unstable();
    let enabled = engine
        .resources
        .rules
        .colony
        .as_ref()
        .is_some_and(|rules| rules.knowledge.is_some());
    let mut lost = Vec::new();
    let mut ordered = true;
    let mut previous = 0;
    for entry in engine.resources.chronicle.entries() {
        ordered &= entry.tick >= previous && entry.tick <= engine.current_tick();
        previous = entry.tick;
        if entry.kind == ChronicleKind::KnowledgeLost {
            let index = usize::try_from(entry.value).expect("native knowledge node is nonnegative");
            let node = sim_core::colony::knowledge::defs()
                .nodes
                .get(index)
                .expect("native knowledge node exists");
            lost.push(json!({"tick": entry.tick, "node": node.name, "holder": entry.subject}));
        }
    }
    assert!(
        enabled || lost.is_empty(),
        "knowledge-disabled world contains native losses"
    );
    let commands: Vec<Value> = engine.resources.colony.command_log.iter().map(|row| {
        json!({"tick": row.tick, "command": sim_test::probe::colony::command_json(row.tick, &row.command),
            "accepted": row.result.accepted, "count": row.result.count,
            "result_debug": format!("{:?}", row.result)})
    }).collect();
    let t = &engine.resources.tallies;
    json!({"tick": engine.current_tick(), "alive": alive.len(), "alive_ids": alive,
        "raw_agent_population": raw.len(), "raw_agent_ids": raw,
        "census_semantics": "alive excludes BodyHealth::is_dead; raw includes every Agent; wrapper must check ordinary endpoint agreement",
        "knowledge_capability": {"enabled": enabled, "source": "resources.rules.colony.knowledge"},
        "knowledge_lost": if enabled { Some(lost) } else { None }, "history_order_valid": ordered,
        "chronicle_entries": engine.resources.chronicle.entries().len(), "command_log": commands,
        "births": t.births, "deaths": {"starvation": t.deaths_starvation,
            "dehydration": t.deaths_dehydration, "exposure": t.deaths_exposure,
            "combat": t.deaths_combat, "old_age": t.deaths_old_age}})
}

fn write_json(path: &std::path::Path, value: &Value) {
    use std::io::Write;
    let pending = path.with_extension("pending");
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&pending)
        .expect("fresh pending evidence");
    serde_json::to_writer_pretty(&mut file, value).expect("serialize evidence");
    file.write_all(b"\n").expect("newline");
    file.sync_all().expect("sync evidence");
    std::fs::hard_link(&pending, path).expect("publish evidence without replacement");
    std::fs::remove_file(pending).expect("remove own published pending link");
}

impl Watch {
    pub fn begin(test: &str, seed: u64, branch: &str, engine: &SimEngine) -> Self {
        let flag = std::env::var_os("WORLDSIM_GATE_OBSERVER");
        if flag.is_none() {
            return Self(None);
        }
        assert_eq!(
            flag.as_deref(),
            Some(std::ffi::OsStr::new("1")),
            "explicit observer flag"
        );
        assert!(test
            .bytes()
            .chain(branch.bytes())
            .all(|c| c.is_ascii_alphanumeric() || c == b'_'));
        let root = PathBuf::from(
            std::env::var_os("WORLDSIM_GATE_TEST_OUT").expect("external invocation evidence root"),
        );
        assert!(
            root.is_absolute() && root.is_dir(),
            "existing absolute invocation evidence root"
        );
        assert_eq!(
            root.canonicalize().expect("canonical invocation root"),
            root,
            "no symlink evidence root"
        );
        let parent = root.join(format!("save_load_pid{}", std::process::id()));
        std::fs::create_dir_all(&parent).expect("invocation evidence parent");
        let directory = parent.join(format!("{test}_s{seed}_{branch}"));
        std::fs::create_dir(&directory).expect("exclusive occurrence directory");
        let initial = snapshot(engine);
        write_json(
            &directory.join("initial.json"),
            &json!({"schema_version": 1,
            "test": test, "seed": seed, "branch": branch, "pid": std::process::id(),
            "initial": initial, "history_complete": engine.current_tick() == 0,
            "status": "PENDING_NATIVE_TEST_RESULT"}),
        );
        Self(Some(Active {
            directory,
            founding: initial.clone(),
            initial,
            prefix: None,
        }))
    }

    pub fn loaded(test: &str, seed: u64, branch: &str, engine: &SimEngine, link: SaveLink) -> Self {
        let mut watch = Self::begin(test, seed, branch, engine);
        if let (Some(active), Some(link)) = (watch.0.as_mut(), link.0) {
            assert_eq!(
                active.initial, link.state,
                "loaded native census/history equals saved prefix"
            );
            active.founding = link.founding;
            active.prefix = Some(
                json!({"save_artifact": link.artifact, "checkpoint": link.checkpoint,
                "native_prefix_equal": true, "external_hash_authentication_required": true}),
            );
            write_json(
                &active.directory.join("load-link.json"),
                &json!({"prefix": active.prefix,
                "founding": active.founding, "initial": active.initial}),
            );
        } else {
            assert!(
                watch.0.is_none(),
                "enabled loaded observer requires actual saved prefix"
            );
        }
        watch
    }

    pub fn checkpoint(&self, engine: &SimEngine, label: &str) {
        self.annotate(engine, label, || Value::Null);
    }

    pub fn annotate(&self, engine: &SimEngine, label: &str, detail: impl FnOnce() -> Value) {
        if let Some(active) = &self.0 {
            assert!(label
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || c == b'_'));
            let detail = detail();
            let state = snapshot(engine);
            let record = if detail.is_null() {
                state
            } else {
                json!({"state": state, "detail": detail})
            };
            write_json(
                &active.directory.join(format!("checkpoint_{label}.json")),
                &record,
            );
        }
    }

    pub fn save_file(&self, engine: &SimEngine, path: &std::path::Path, label: &str) -> SaveLink {
        if self.0.is_none() {
            return SaveLink(None);
        }
        let bytes = std::fs::read(path).expect("read actual bridge save before cleanup");
        self.save(engine, &bytes, label)
    }

    pub fn save(&self, engine: &SimEngine, bytes: &[u8], label: &str) -> SaveLink {
        let Some(active) = &self.0 else {
            return SaveLink(None);
        };
        use std::io::Write;
        assert!(label
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || c == b'_'));
        let artifact = active.directory.join(format!("save_{label}.wsave"));
        let checkpoint = active.directory.join(format!("save_{label}.json"));
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&artifact)
            .expect("exclusive saved-byte artifact");
        file.write_all(bytes)
            .expect("retain exact native save bytes");
        file.sync_all().expect("sync saved bytes");
        let state = snapshot(engine);
        write_json(
            &checkpoint,
            &json!({"state": state, "save_artifact": artifact,
            "byte_count": bytes.len(), "magic": bytes.get(..8),
            "input_version_le_bytes": bytes.get(8..12), "current_format_version": sim_systems::save::FORMAT_VERSION,
            "external_sha256_required": true}),
        );
        SaveLink(Some(Link {
            artifact,
            checkpoint,
            state,
            founding: active.founding.clone(),
        }))
    }

    pub fn finish(self, engine: &SimEngine, reason: &str) {
        if let Some(active) = self.0 {
            let terminal = snapshot(engine);
            let initial_tick = active.initial["tick"].as_u64().expect("initial tick");
            write_json(
                &active.directory.join("terminal.json"),
                &json!({"schema_version": 1,
                "initial": active.initial, "founding": active.founding, "terminal": terminal,
                "completed_ticks": engine.current_tick().checked_sub(initial_tick).expect("monotonic engine time"),
                "stop_reason": reason, "prefix": active.prefix,
                "history_complete": active.prefix.is_none() && initial_tick == 0 && terminal["history_order_valid"] == true,
                "loaded_prefix_requires_external_authentication": active.prefix.is_some() || initial_tick != 0,
                "status": "ENDPOINT_REACHED_NATIVE_TEST_RESULT_REQUIRED"}),
            );
        }
    }
}
