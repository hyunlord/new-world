//! Optional parent-side process evidence, outside admission and child measurement.
use serde_json::{json, Value};
use std::ffi::OsStr;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::{Command, ExitStatus};

pub(super) struct Run<'a> {
    pub output: &'a Path,
    pub kind: &'a str,
    pub scene: &'a str,
    pub repeat: u32,
}

pub(super) struct Journal {
    file: File,
    sequence: u64,
    child_index: u64,
}

impl Journal {
    pub(super) fn from_environment(out: &Path) -> Result<Option<Self>, String> {
        let setting = std::env::var_os("WORLDSIM_GATE_OBSERVER");
        Self::with_setting(out, setting.as_deref())
    }

    fn with_setting(out: &Path, setting: Option<&OsStr>) -> Result<Option<Self>, String> {
        if !crate::probe::ProbeArgs::gate_observations_enabled(false, setting)? {
            return Ok(None);
        }
        let file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(out.join("child-processes.jsonl"))
            .map_err(|error| error.to_string())?;
        Ok(Some(Self {
            file,
            sequence: 0,
            child_index: 0,
        }))
    }

    fn record(&mut self, mut row: Value, phase: &str) -> Result<(), String> {
        row["schema"] = json!("bench_child_process_v1");
        row["sequence"] = json!(self.sequence);
        row["child_index"] = json!(self.child_index);
        row["phase"] = json!(phase);
        self.sequence += 1;
        serde_json::to_writer(&mut self.file, &row).map_err(|error| error.to_string())?;
        self.file
            .write_all(b"\n")
            .map_err(|error| error.to_string())?;
        self.file.flush().map_err(|error| error.to_string())
    }

    fn run(&mut self, command: &mut Command, run: &Run<'_>) -> Result<ExitStatus, String> {
        let output = run.output;
        let argv: Vec<&str> = std::iter::once(command.get_program())
            .chain(command.get_args())
            .map(|arg| arg.to_str().ok_or("non-UTF8 benchmark argument"))
            .collect::<Result<_, _>>()?;
        let parent_cwd = std::env::current_dir().map_err(|error| error.to_string())?;
        let cwd = command
            .get_current_dir()
            .unwrap_or(&parent_cwd)
            .canonicalize()
            .map_err(|error| error.to_string())?;
        // --out is interpreted by the child relative to its effective working directory.
        let output = if output.is_absolute() {
            output.to_path_buf()
        } else {
            cwd.join(output)
        };
        let parent = output
            .parent()
            .ok_or("output lacks parent")?
            .canonicalize()
            .map_err(|error| error.to_string())?;
        let output = parent.join(output.file_name().ok_or("output lacks filename")?);
        let mut row = json!({"parent_pid":std::process::id(), "pid":null,
            "argv":argv, "cwd":cwd, "output":output, "kind":run.kind,
            "scene":run.scene, "repeat":run.repeat, "native_exit":null,
            "signal":null, "success":null, "error":null});
        self.record(row.clone(), "launch")?;
        let mut child = match command.spawn() {
            Ok(child) => child,
            Err(error) => {
                row["error"] = json!(error.to_string());
                self.record(row, "spawn_failed")?;
                self.child_index += 1;
                return Err(error.to_string());
            }
        };
        row["pid"] = json!(child.id());
        let spawn_record = self.record(row.clone(), "spawned");
        // Never leave the actual child unawaited because recording its PID failed.
        let result = child.wait();
        let terminal_record = match &result {
            Ok(status) => {
                row["native_exit"] = json!(status.code());
                row["success"] = json!(status.success());
                #[cfg(unix)]
                {
                    use std::os::unix::process::ExitStatusExt;
                    row["signal"] = json!(status.signal());
                }
                self.record(row, "completed")
            }
            Err(error) => {
                row["error"] = json!(error.to_string());
                self.record(row, "wait_failed")
            }
        };
        self.child_index += 1;
        spawn_record?;
        terminal_record?;
        result.map_err(|error| error.to_string())
    }
}

pub(super) fn status(
    journal: &mut Option<Journal>,
    command: &mut Command,
    run: &Run<'_>,
) -> Result<ExitStatus, String> {
    match journal {
        Some(journal) => journal.run(command, run),
        None => command.status().map_err(|error| error.to_string()),
    }
}

#[cfg(test)]
#[path = "bench_process_tests.rs"]
mod tests;
