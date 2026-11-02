//! Tiny real children only: no benchmark workload, admission wait or simulated world.
use super::*;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(0);
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "bench-process-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).expect("new temp");
        Self(path.canonicalize().expect("canonical temp"))
    }
    fn rows(&self) -> Vec<Value> {
        std::fs::read_to_string(self.0.join("child-processes.jsonl"))
            .expect("journal")
            .lines()
            .map(|line| serde_json::from_str(line).expect("row"))
            .collect()
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[cfg(unix)]
#[test]
fn real_zero_and_nonzero_children_retain_exact_pid_and_sequential_outcomes() {
    let out = Temp::new();
    let mut journal = Journal::with_setting(&out.0, Some(OsStr::new("1"))).expect("enabled");
    for (index, code) in [0, 7].into_iter().enumerate() {
        let pidfile = out.0.join(format!("pid{index}"));
        let output = out.0.join(format!("run{index}.json"));
        let mut command = Command::new("/bin/sh");
        command.current_dir(&out.0).args([
            "-c",
            &format!("echo $$ > '{}'; exit {code}", pidfile.display()),
        ]);
        let result = status(
            &mut journal,
            &mut command,
            &Run {
                output: &output,
                kind: "timed",
                scene: "test",
                repeat: index as u32 + 1,
            },
        )
        .expect("wait");
        assert_eq!(result.code(), Some(code));
        let rows = out.rows();
        let group = &rows[index * 3..index * 3 + 3];
        let actual_pid: u32 = std::fs::read_to_string(pidfile)
            .expect("child pid")
            .trim()
            .parse()
            .expect("pid number");
        assert_eq!(group[0]["phase"], "launch");
        assert_eq!(group[0]["pid"], Value::Null);
        assert_eq!(group[1]["phase"], "spawned");
        assert_eq!(group[1]["pid"], actual_pid);
        assert_eq!(group[2]["phase"], "completed");
        assert_eq!(group[2]["native_exit"], code);
        assert_eq!(group[2]["success"], code == 0);
        assert_eq!(group[2]["signal"], Value::Null);
        for (offset, row) in group.iter().enumerate() {
            assert_eq!(row["sequence"], index * 3 + offset);
            assert_eq!(row["child_index"], index);
            assert_eq!(row["parent_pid"], std::process::id());
            assert_eq!(row["cwd"], out.0.to_str().expect("cwd"));
            assert_eq!(row["output"], output.to_str().expect("out"));
            assert_eq!(row["argv"][0], "/bin/sh");
        }
    }
    assert!(Journal::with_setting(&out.0, Some(OsStr::new("1"))).is_err());
}

#[cfg(unix)]
#[test]
fn disabled_journal_keeps_child_exit_and_creates_no_evidence_file() {
    let out = Temp::new();
    let mut journal = Journal::with_setting(&out.0, None).expect("disabled");
    assert!(journal.is_none());
    let mut child = Command::new("/bin/sh");
    child.args(["-c", "exit 9"]);
    let result = status(
        &mut journal,
        &mut child,
        &Run {
            output: &out.0.join("unused"),
            kind: "rss",
            scene: "test",
            repeat: 1,
        },
    )
    .expect("child");
    assert_eq!(result.code(), Some(9));
    assert!(!out.0.join("child-processes.jsonl").exists());
    assert!(Journal::with_setting(&out.0, Some(OsStr::new("0"))).is_err());
}

#[cfg(unix)]
#[test]
fn signal_and_spawn_failure_never_look_like_success() {
    let out = Temp::new();
    let mut journal = Journal::with_setting(&out.0, Some(OsStr::new("1"))).expect("enabled");
    let output = out.0.join("unused");
    let run = Run {
        output: &output,
        kind: "rss",
        scene: "test",
        repeat: 1,
    };
    let mut child = Command::new("/bin/sh");
    child.args(["-c", "kill -TERM $$"]);
    assert!(!status(&mut journal, &mut child, &run)
        .expect("signal wait")
        .success());
    let mut missing = Command::new(out.0.join("missing-executable"));
    assert!(status(&mut journal, &mut missing, &run).is_err());
    let rows = out.rows();
    assert_eq!(rows[2]["native_exit"], Value::Null);
    assert_eq!(rows[2]["signal"], 15);
    assert_eq!(rows[2]["success"], false);
    assert_eq!(rows[4]["phase"], "spawn_failed");
    assert_eq!(rows[4]["pid"], Value::Null);
    assert!(rows[4]["error"].is_string());
}
