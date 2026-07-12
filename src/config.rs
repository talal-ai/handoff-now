use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    env, fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default, rename_all = "camelCase")]
pub struct Config {
    pub prepare_above_percentage: f64,
    pub handoff_above_percentage: f64,
    pub hard_stop_above_percentage: f64,
    pub artifact_directory: String,
    pub semantic_provider: SemanticProvider,
    pub api_model: String,
    pub retain_raw_transcript: bool,
    pub redaction_mode: RedactionMode,
    pub include_git_diff: bool,
    pub maximum_semantic_input_bytes: usize,
    pub telemetry: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum SemanticProvider {
    Hybrid,
    Subscription,
    Api,
    Deterministic,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum RedactionMode {
    Standard,
    Strict,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            prepare_above_percentage: 85.0,
            handoff_above_percentage: 90.0,
            hard_stop_above_percentage: 95.0,
            artifact_directory: ".handoff-now".into(),
            semantic_provider: SemanticProvider::Hybrid,
            api_model: "claude-haiku-4-5-20251001".into(),
            retain_raw_transcript: false,
            redaction_mode: RedactionMode::Standard,
            include_git_diff: true,
            maximum_semantic_input_bytes: 120_000,
            telemetry: false,
        }
    }
}

impl Config {
    pub fn validate(&self) -> Result<()> {
        if !(0.0..=100.0).contains(&self.prepare_above_percentage)
            || !(0.0..=100.0).contains(&self.handoff_above_percentage)
            || !(0.0..=100.0).contains(&self.hard_stop_above_percentage)
        {
            bail!("threshold percentages must be between 0 and 100");
        }
        if !(self.prepare_above_percentage < self.handoff_above_percentage
            && self.handoff_above_percentage < self.hard_stop_above_percentage)
        {
            bail!("thresholds must satisfy prepare < handoff < hard stop");
        }
        if self.artifact_directory.is_empty()
            || Path::new(&self.artifact_directory).is_absolute()
            || self
                .artifact_directory
                .split(['/', '\\'])
                .any(|p| p == "..")
        {
            bail!("artifactDirectory must be a safe project-relative path");
        }
        if self.maximum_semantic_input_bytes < 8_192
            || self.maximum_semantic_input_bytes > 2_000_000
        {
            bail!("maximumSemanticInputBytes must be between 8192 and 2000000");
        }
        Ok(())
    }

    pub fn load_or_default(path: &Path, diagnostics: &Path) -> Self {
        match fs::read_to_string(path)
            .with_context(|| format!("read {}", path.display()))
            .and_then(|s| serde_json::from_str::<Self>(&s).context("parse config"))
            .and_then(|c| {
                c.validate()?;
                Ok(c)
            }) {
            Ok(c) => c,
            Err(err) => {
                let _ = fs::create_dir_all(diagnostics.parent().unwrap_or(Path::new(".")));
                let _ = fs::write(
                    diagnostics,
                    format!("Invalid or missing configuration; safe defaults active: {err:#}\n"),
                );
                Self::default()
            }
        }
    }

    pub fn user_root() -> Result<PathBuf> {
        Ok(Self::claude_dir()?.join("handoff-now"))
    }

    pub fn claude_dir() -> Result<PathBuf> {
        if let Some(path) = env::var_os("CLAUDE_CONFIG_DIR").filter(|v| !v.is_empty()) {
            return Ok(PathBuf::from(path));
        }
        let home = dirs::home_dir().context("cannot determine home directory")?;
        Ok(home.join(".claude"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn validates_default() {
        Config::default().validate().unwrap();
    }
    #[test]
    fn rejects_equal_thresholds() {
        let c = Config {
            handoff_above_percentage: 85.0,
            ..Config::default()
        };
        assert!(c.validate().is_err());
    }
    #[test]
    fn rejects_traversal() {
        let c = Config {
            artifact_directory: "../escape".into(),
            ..Config::default()
        };
        assert!(c.validate().is_err());
    }
}
