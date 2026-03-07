use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use crossbeam_channel::{bounded, Receiver, Sender, TrySendError};
use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use sim_core::config;

use crate::llm_worker::{llm_worker_loop, LlmRequest, LlmRequestMeta, LlmResponse};

/// Parsed LLM configuration used to start llama-server and the worker thread.
#[derive(Debug, Clone)]
pub struct LlmConfig {
    /// Whether runtime init should attempt to auto-start the LLM layer.
    pub enabled_default: bool,
    /// Server host.
    pub host: String,
    /// Server port.
    pub port: u16,
    /// Absolute path to the llama-server binary.
    pub server_binary: PathBuf,
    /// Absolute path to the GGUF model.
    pub model_path: PathBuf,
    /// Absolute path to the prompt template directory.
    pub prompt_dir: PathBuf,
    /// Absolute path to the Layer 3 grammar file.
    pub grammar_path: PathBuf,
    /// Pre-loaded grammar source.
    pub layer3_grammar: String,
    /// Context size passed to llama-server.
    pub context_size: u32,
    /// Max tokens for Layer 3.
    pub max_tokens_l3: u32,
    /// Max tokens for Layer 4.
    pub max_tokens_l4: u32,
    /// llama-server thread count.
    pub threads: u32,
    /// llama-server batch thread count.
    pub threads_batch: u32,
    /// Temperature used for Layer 3.
    pub temperature_l3: f64,
    /// Temperature used for Layer 4.
    pub temperature_l4: f64,
    /// Bounded queue capacity.
    pub queue_capacity: usize,
    /// Health-check retry attempts.
    pub healthcheck_attempts: u32,
    /// Delay between health-check retries.
    pub healthcheck_interval_ms: u64,
    /// Grace period before force-killing the server process.
    pub shutdown_grace_ms: u64,
    /// Model identifier emitted for diagnostics.
    pub model_id: String,
}

/// Snapshot of the currently exposed LLM runtime status.
#[derive(Debug, Clone, Serialize)]
pub struct LlmStatusSnapshot {
    /// Whether the llama-server process is currently running.
    pub running: bool,
    /// Model identifier or empty string.
    pub model: String,
    /// Current queue depth for pending requests.
    pub queue_depth: usize,
}

#[derive(Debug, Deserialize)]
struct LlmConfigFile {
    enabled_default: Option<bool>,
    host: Option<String>,
    port: Option<u16>,
    server_binary: Option<String>,
    model_path: Option<String>,
    prompt_dir: Option<String>,
    grammar_path: Option<String>,
    context_size: Option<u32>,
    max_tokens_l3: Option<u32>,
    max_tokens_l4: Option<u32>,
    threads: Option<u32>,
    threads_batch: Option<u32>,
    temperature_l3: Option<f64>,
    temperature_l4: Option<f64>,
    queue_capacity: Option<usize>,
    healthcheck_attempts: Option<u32>,
    healthcheck_interval_ms: Option<u64>,
    shutdown_grace_ms: Option<u64>,
}

struct LlmChannels {
    request_tx: Sender<LlmRequest>,
    request_rx: Receiver<LlmRequest>,
    response_rx: Receiver<LlmResponse>,
}

struct LlmServerProcess {
    child: Child,
}

/// Runtime owner for the external llama-server process and worker transport.
pub struct LlmRuntime {
    config: LlmConfig,
    server: Option<LlmServerProcess>,
    channels: Option<LlmChannels>,
    worker: Option<JoinHandle<()>>,
    in_flight: HashMap<u64, LlmRequestMeta>,
    next_request_id: u64,
}

/// Errors emitted by the LLM runtime manager.
#[derive(Debug, Error)]
pub enum LlmRuntimeError {
    #[error("llm config IO failed: {0}")]
    ConfigIo(#[from] std::io::Error),
    #[error("llm config parse failed: {0}")]
    ConfigParse(#[from] toml::de::Error),
    #[error("llama-server binary not found: {0}")]
    MissingServerBinary(String),
    #[error("LLM model not found: {0}")]
    MissingModel(String),
    #[error("spawn llama-server failed: {0}")]
    SpawnFailed(String),
    #[error("llama-server health check failed")]
    HealthCheckFailed,
    #[error("LLM runtime is unavailable")]
    Unavailable,
    #[error("LLM request queue is full")]
    QueueFull,
}

impl LlmConfig {
    /// Loads the default config file, falling back to shared defaults when absent.
    pub fn load_default() -> Result<Self, LlmRuntimeError> {
        let project_root = project_root();
        let config_path = project_root.join(config::LLM_CONFIG_PATH);
        let file = if config_path.is_file() {
            let raw = fs::read_to_string(&config_path)?;
            toml::from_str::<LlmConfigFile>(&raw)?
        } else {
            LlmConfigFile {
                enabled_default: None,
                host: None,
                port: None,
                server_binary: None,
                model_path: None,
                prompt_dir: None,
                grammar_path: None,
                context_size: None,
                max_tokens_l3: None,
                max_tokens_l4: None,
                threads: None,
                threads_batch: None,
                temperature_l3: None,
                temperature_l4: None,
                queue_capacity: None,
                healthcheck_attempts: None,
                healthcheck_interval_ms: None,
                shutdown_grace_ms: None,
            }
        };

        let host = file
            .host
            .unwrap_or_else(|| config::LLM_SERVER_HOST.to_string());
        let port = file.port.unwrap_or(config::LLM_SERVER_PORT);
        let server_binary = project_root.join(
            file.server_binary
                .unwrap_or_else(|| config::LLM_SERVER_BINARY.to_string()),
        );
        let model_path = project_root.join(
            file.model_path
                .unwrap_or_else(|| config::LLM_MODEL_PATH.to_string()),
        );
        let prompt_dir = project_root.join(
            file.prompt_dir
                .unwrap_or_else(|| config::LLM_PROMPT_DIR.to_string()),
        );
        let grammar_path = project_root.join(
            file.grammar_path
                .unwrap_or_else(|| config::LLM_LAYER3_GRAMMAR_PATH.to_string()),
        );
        let layer3_grammar = fs::read_to_string(&grammar_path).unwrap_or_default();
        let model_id = model_path
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or("qwen3.5-0.8b-q4km")
            .to_string();

        Ok(Self {
            enabled_default: file
                .enabled_default
                .unwrap_or(config::LLM_ENABLED_DEFAULT),
            host,
            port,
            server_binary,
            model_path,
            prompt_dir,
            grammar_path,
            layer3_grammar,
            context_size: file.context_size.unwrap_or(config::LLM_CONTEXT_SIZE),
            max_tokens_l3: file.max_tokens_l3.unwrap_or(config::LLM_MAX_TOKENS_L3),
            max_tokens_l4: file.max_tokens_l4.unwrap_or(config::LLM_MAX_TOKENS_L4),
            threads: file.threads.unwrap_or(config::LLM_THREADS),
            threads_batch: file
                .threads_batch
                .unwrap_or(config::LLM_THREADS_BATCH),
            temperature_l3: file
                .temperature_l3
                .unwrap_or(config::LLM_TEMPERATURE_L3),
            temperature_l4: file
                .temperature_l4
                .unwrap_or(config::LLM_TEMPERATURE_L4),
            queue_capacity: file
                .queue_capacity
                .unwrap_or(config::LLM_QUEUE_CAPACITY),
            healthcheck_attempts: file
                .healthcheck_attempts
                .unwrap_or(config::LLM_HEALTHCHECK_ATTEMPTS),
            healthcheck_interval_ms: file
                .healthcheck_interval_ms
                .unwrap_or(config::LLM_HEALTHCHECK_INTERVAL_MS),
            shutdown_grace_ms: file
                .shutdown_grace_ms
                .unwrap_or(config::LLM_SHUTDOWN_GRACE_MS),
            model_id,
        })
    }

    /// Returns the base HTTP URL for llama-server.
    pub fn base_url(&self) -> String {
        format!("http://{}:{}", self.host, self.port)
    }
}

impl Default for LlmRuntime {
    fn default() -> Self {
        Self::new(LlmConfig::load_default().unwrap_or_else(|_| default_config()))
    }
}

impl LlmRuntime {
    /// Creates a runtime manager from the provided config.
    pub fn new(config: LlmConfig) -> Self {
        Self {
            config,
            server: None,
            channels: None,
            worker: None,
            in_flight: HashMap::new(),
            next_request_id: 1,
        }
    }

    /// Returns the loaded configuration.
    pub fn config(&self) -> &LlmConfig {
        &self.config
    }

    /// Starts llama-server and the worker thread if they are not already running.
    pub fn start(&mut self) -> Result<(), LlmRuntimeError> {
        if self.is_running() {
            return Ok(());
        }
        if !self.config.server_binary.is_file() {
            return Err(LlmRuntimeError::MissingServerBinary(
                self.config.server_binary.display().to_string(),
            ));
        }
        if !self.config.model_path.is_file() {
            return Err(LlmRuntimeError::MissingModel(
                self.config.model_path.display().to_string(),
            ));
        }

        let child = Command::new(&self.config.server_binary)
            .args([
                "-m",
                self.config.model_path.to_string_lossy().as_ref(),
                "--host",
                self.config.host.as_str(),
                "--port",
                &self.config.port.to_string(),
                "--jinja",
                "--reasoning-format",
                "none",
                "--reasoning-budget",
                "0",
                "-np",
                "1",
                "-c",
                &self.config.context_size.to_string(),
                "-n",
                &self.config.max_tokens_l4.to_string(),
                "-t",
                &self.config.threads.to_string(),
                "-tb",
                &self.config.threads_batch.to_string(),
                "-b",
                "512",
                "-ub",
                "256",
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| LlmRuntimeError::SpawnFailed(error.to_string()))?;

        let mut process = LlmServerProcess { child };
        if !wait_for_health(&self.config, &mut process) {
            let _ = stop_process(&mut process.child, self.config.shutdown_grace_ms);
            return Err(LlmRuntimeError::HealthCheckFailed);
        }

        let (request_tx, request_rx) = bounded::<LlmRequest>(self.config.queue_capacity);
        let (response_tx, response_rx) = bounded::<LlmResponse>(self.config.queue_capacity);
        let worker_config = self.config.clone();
        let worker_rx = request_rx.clone();
        let worker_handle = thread::spawn(move || {
            llm_worker_loop(worker_rx, response_tx, worker_config);
        });

        self.server = Some(process);
        self.channels = Some(LlmChannels {
            request_tx,
            request_rx,
            response_rx,
        });
        self.worker = Some(worker_handle);
        Ok(())
    }

    /// Stops the worker thread and the external llama-server process.
    pub fn stop(&mut self) {
        self.in_flight.clear();
        self.channels = None;
        if let Some(handle) = self.worker.take() {
            let _ = handle.join();
        }
        if let Some(mut process) = self.server.take() {
            let _ = stop_process(&mut process.child, self.config.shutdown_grace_ms);
        }
    }

    /// Returns true when the server process is alive.
    pub fn is_running(&self) -> bool {
        self.server.is_some()
    }

    /// Returns true when the runtime can currently accept requests.
    pub fn is_available(&self) -> bool {
        self.is_running() && self.channels.is_some()
    }

    /// Serializes the current status as a JSON string for SimBridge callers.
    pub fn status_json(&self) -> String {
        let queue_depth = self
            .channels
            .as_ref()
            .map(|channels| channels.request_rx.len())
            .unwrap_or(0);
        let snapshot = LlmStatusSnapshot {
            running: self.is_running(),
            model: if self.is_available() {
                self.config.model_id.clone()
            } else {
                String::new()
            },
            queue_depth,
        };
        serde_json::to_string(&snapshot).unwrap_or_else(|_| {
            "{\"running\":false,\"model\":\"\",\"queue_depth\":0}".to_string()
        })
    }

    /// Attempts to enqueue an LLM request without blocking the game thread.
    pub fn submit_request(&mut self, mut request: LlmRequest) -> Result<u64, LlmRuntimeError> {
        if !self.is_available() {
            return Err(LlmRuntimeError::Unavailable);
        }
        let channels = self.channels.as_ref().ok_or(LlmRuntimeError::Unavailable)?;
        let request_id = self.next_request_id;
        self.next_request_id = self.next_request_id.saturating_add(1);
        request.request_id = request_id;
        match channels.request_tx.try_send(request.clone()) {
            Ok(()) => {
                self.in_flight.insert(request_id, request.meta());
                Ok(request_id)
            }
            Err(TrySendError::Full(_)) => Err(LlmRuntimeError::QueueFull),
            Err(TrySendError::Disconnected(_)) => Err(LlmRuntimeError::Unavailable),
        }
    }

    /// Drains all currently available worker responses.
    pub fn drain_responses(&mut self) -> Vec<LlmResponse> {
        let mut responses: Vec<LlmResponse> = Vec::new();
        let Some(channels) = self.channels.as_ref() else {
            return responses;
        };
        while let Ok(response) = channels.response_rx.try_recv() {
            responses.push(response);
        }
        responses
    }

    /// Removes and returns in-flight metadata for one request.
    pub fn take_request_meta(&mut self, request_id: u64) -> Option<LlmRequestMeta> {
        self.in_flight.remove(&request_id)
    }
}

impl Drop for LlmRuntime {
    fn drop(&mut self) {
        self.stop();
    }
}

fn default_config() -> LlmConfig {
    let project_root = project_root();
    LlmConfig {
        enabled_default: config::LLM_ENABLED_DEFAULT,
        host: config::LLM_SERVER_HOST.to_string(),
        port: config::LLM_SERVER_PORT,
        server_binary: project_root.join(config::LLM_SERVER_BINARY),
        model_path: project_root.join(config::LLM_MODEL_PATH),
        prompt_dir: project_root.join(config::LLM_PROMPT_DIR),
        grammar_path: project_root.join(config::LLM_LAYER3_GRAMMAR_PATH),
        layer3_grammar: String::new(),
        context_size: config::LLM_CONTEXT_SIZE,
        max_tokens_l3: config::LLM_MAX_TOKENS_L3,
        max_tokens_l4: config::LLM_MAX_TOKENS_L4,
        threads: config::LLM_THREADS,
        threads_batch: config::LLM_THREADS_BATCH,
        temperature_l3: config::LLM_TEMPERATURE_L3,
        temperature_l4: config::LLM_TEMPERATURE_L4,
        queue_capacity: config::LLM_QUEUE_CAPACITY,
        healthcheck_attempts: config::LLM_HEALTHCHECK_ATTEMPTS,
        healthcheck_interval_ms: config::LLM_HEALTHCHECK_INTERVAL_MS,
        shutdown_grace_ms: config::LLM_SHUTDOWN_GRACE_MS,
        model_id: "qwen3.5-0.8b-q4km".to_string(),
    }
}

fn wait_for_health(config: &LlmConfig, process: &mut LlmServerProcess) -> bool {
    for _ in 0..config.healthcheck_attempts {
        if health_check(config) {
            return true;
        }
        if process.child.try_wait().ok().flatten().is_some() {
            return false;
        }
        thread::sleep(Duration::from_millis(config.healthcheck_interval_ms));
    }
    false
}

fn health_check(config: &LlmConfig) -> bool {
    let url = format!("{}/health", config.base_url());
    ureq::get(url.as_str())
        .call()
        .map(|response| response.status() == 200)
        .unwrap_or(false)
}

fn stop_process(child: &mut Child, shutdown_grace_ms: u64) -> std::io::Result<()> {
    let pid = Pid::from_raw(child.id() as i32);
    let _ = kill(pid, Signal::SIGTERM);
    let start = Instant::now();
    while start.elapsed() < Duration::from_millis(shutdown_grace_ms) {
        if child.try_wait()?.is_some() {
            return Ok(());
        }
        thread::sleep(Duration::from_millis(50));
    }
    let _ = kill(pid, Signal::SIGKILL);
    let _ = child.wait();
    Ok(())
}

fn project_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .and_then(Path::parent)
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .unwrap_or(manifest_dir)
}

#[cfg(test)]
mod tests {
    use super::{default_config, LlmConfig, LlmRuntime};

    #[test]
    fn default_config_uses_project_relative_paths() {
        let config = LlmConfig::load_default().unwrap_or_else(|_| default_config());
        assert!(config.prompt_dir.ends_with("data/llm/prompts"));
        assert!(config.grammar_path.ends_with("data/llm/grammars/layer3_judgment.gbnf"));
    }

    #[test]
    fn status_json_reports_not_running_by_default() {
        let runtime = LlmRuntime::default();
        let status = runtime.status_json();
        assert!(status.contains("\"running\":false"));
        assert!(status.contains("\"queue_depth\":0"));
    }
}
