use std::fs;
use std::path::{Path, PathBuf};

use minijinja::Environment;
use serde::Serialize;
use thiserror::Error;

use sim_core::config;

/// Request context serialized into the Layer 3/4 prompt templates.
#[derive(Debug, Clone, Serialize)]
pub struct LlmPromptContext {
    /// Entity display name.
    pub entity_name: String,
    /// Narrative role label.
    pub role: String,
    /// Current action identifier from the closed action space.
    pub action_id: u32,
    /// Human-readable action label.
    pub action_label: String,
    /// Current HEXACO axes.
    pub personality_axes: [f64; 6],
    /// Current Plutchik primary emotion values.
    pub emotions: [f64; 8],
    /// Current 13-need values.
    pub needs: [f64; 13],
    /// Current stress level.
    pub stress_level: f64,
    /// Current stress-state bucket.
    pub stress_state: u8,
    /// Optional recent event label.
    pub recent_event_type: Option<String>,
    /// Optional recent event cause description.
    pub recent_event_cause: Option<String>,
    /// Optional recent target/partner name.
    pub recent_target_name: Option<String>,
}

/// Fully rendered prompt pair sent to llama-server.
#[derive(Debug, Clone)]
pub struct RenderedPrompt {
    /// Shared system prompt.
    pub system: String,
    /// Request-specific user prompt.
    pub user: String,
}

/// Loaded prompt templates for the Phase 1 LLM runtime.
pub struct LlmPromptTemplates {
    env: Environment<'static>,
}

/// Errors emitted while loading or rendering prompt templates.
#[derive(Debug, Error)]
pub enum LlmPromptError {
    #[error("prompt template IO failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("prompt template not found: {0}")]
    MissingTemplate(String),
    #[error("prompt render failed: {0}")]
    Render(String),
}

impl LlmPromptTemplates {
    /// Loads all prompt templates from the given directory.
    pub fn load(prompt_dir: &Path) -> Result<Self, LlmPromptError> {
        let mut env = Environment::new();
        for template_name in [
            "system.jinja",
            "layer3_judgment.jinja",
            "layer4_narrative.jinja",
            "layer4_personality.jinja",
        ] {
            let template_path = prompt_dir.join(template_name);
            let source = fs::read_to_string(&template_path)?;
            let leaked_source: &'static str = Box::leak(source.into_boxed_str());
            env.add_template(template_name, leaked_source)
                .map_err(|error| LlmPromptError::Render(error.to_string()))?;
        }
        Ok(Self { env })
    }

    /// Loads templates from the default project prompt directory.
    pub fn load_default() -> Result<Self, LlmPromptError> {
        Self::load(&default_prompt_dir())
    }

    /// Renders the shared system prompt.
    pub fn render_system(&self) -> Result<String, LlmPromptError> {
        self.render("system.jinja", minijinja::context! {})
    }

    /// Renders the Layer 3 judgment prompt pair.
    pub fn render_layer3_judgment(
        &self,
        context: &LlmPromptContext,
    ) -> Result<RenderedPrompt, LlmPromptError> {
        Ok(RenderedPrompt {
            system: self.render_system()?,
            user: self.render("layer3_judgment.jinja", minijinja::context! { entity => context })?,
        })
    }

    /// Renders the Layer 4 event narrative prompt pair.
    pub fn render_layer4_narrative(
        &self,
        context: &LlmPromptContext,
    ) -> Result<RenderedPrompt, LlmPromptError> {
        Ok(RenderedPrompt {
            system: self.render_system()?,
            user: self.render("layer4_narrative.jinja", minijinja::context! { entity => context })?,
        })
    }

    /// Renders the Layer 4 personality-description prompt pair.
    pub fn render_layer4_personality(
        &self,
        context: &LlmPromptContext,
    ) -> Result<RenderedPrompt, LlmPromptError> {
        Ok(RenderedPrompt {
            system: self.render_system()?,
            user: self.render("layer4_personality.jinja", minijinja::context! { entity => context })?,
        })
    }

    fn render<S: Serialize>(
        &self,
        template_name: &str,
        context: S,
    ) -> Result<String, LlmPromptError> {
        let template = self
            .env
            .get_template(template_name)
            .map_err(|_| LlmPromptError::MissingTemplate(template_name.to_string()))?;
        template
            .render(context)
            .map_err(|error| LlmPromptError::Render(error.to_string()))
    }
}

/// Returns the default project prompt directory.
pub fn default_prompt_dir() -> PathBuf {
    project_root().join(config::LLM_PROMPT_DIR)
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
    use super::{default_prompt_dir, LlmPromptContext, LlmPromptTemplates};

    fn sample_context() -> LlmPromptContext {
        LlmPromptContext {
            entity_name: "Kaya".to_string(),
            role: "agent".to_string(),
            action_id: 3,
            action_label: "GatherFood".to_string(),
            personality_axes: [0.5; 6],
            emotions: [0.1; 8],
            needs: [0.8; 13],
            stress_level: 0.2,
            stress_state: 0,
            recent_event_type: Some("action_changed".to_string()),
            recent_event_cause: Some("gather_food".to_string()),
            recent_target_name: None,
        }
    }

    #[test]
    fn default_prompt_dir_points_into_project_data_tree() {
        let dir = default_prompt_dir();
        assert!(dir.ends_with("data/llm/prompts"));
    }

    #[test]
    fn prompt_templates_render_all_layers() {
        let templates = LlmPromptTemplates::load_default().expect("prompt templates should load");
        let context = sample_context();
        let system = templates.render_system().expect("system prompt should render");
        let layer3 = templates
            .render_layer3_judgment(&context)
            .expect("layer3 prompt should render");
        let layer4 = templates
            .render_layer4_narrative(&context)
            .expect("layer4 prompt should render");
        let personality = templates
            .render_layer4_personality(&context)
            .expect("layer4 personality prompt should render");

        assert!(!system.trim().is_empty());
        assert!(layer3.user.contains("Kaya"));
        assert!(layer4.user.contains("Kaya"));
        assert!(personality.user.contains("Kaya"));
    }
}
