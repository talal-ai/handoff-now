use crate::config::Config;
use anyhow::{Context, Result};
use atomicwrites::{AllowOverwrite, AtomicFile};
use chrono::{DateTime, Utc};
use fs2::FileExt;
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Phase {
    Normal,
    Preparing,
    Prepared,
    HandoffRequired,
    SemanticFinalizing,
    Finalized,
    HardStopped,
    Exhausted,
    Reset,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionState {
    pub schema_version: u32,
    pub session_id: String,
    pub cwd: PathBuf,
    pub transcript_path: Option<PathBuf>,
    pub usage_percentage: Option<f64>,
    pub resets_at: Option<i64>,
    pub phase: Phase,
    pub previous_phase: Option<Phase>,
    pub journal_sequence: u64,
    pub last_transcript_offset: u64,
    pub semantic_attempted: bool,
    #[serde(default)]
    pub semantic_checkpoint_sequence: Option<u64>,
    #[serde(default)]
    pub preparation_semantic_attempted: bool,
    #[serde(default)]
    pub emergency_semantic_attempted: bool,
    pub final_handoff_path: Option<PathBuf>,
    pub updated_at: DateTime<Utc>,
}

impl SessionState {
    pub fn new(session_id: String, cwd: PathBuf) -> Self {
        Self {
            schema_version: 1,
            session_id,
            cwd,
            transcript_path: None,
            usage_percentage: None,
            resets_at: None,
            phase: Phase::Normal,
            previous_phase: None,
            journal_sequence: 0,
            last_transcript_offset: 0,
            semantic_attempted: false,
            semantic_checkpoint_sequence: None,
            preparation_semantic_attempted: false,
            emergency_semantic_attempted: false,
            final_handoff_path: None,
            updated_at: Utc::now(),
        }
    }

    pub fn observe_usage(&mut self, usage: Option<f64>, reset: Option<i64>, config: &Config) {
        let old_reset = self.resets_at;
        let old_usage = self.usage_percentage;
        self.usage_percentage = usage;
        self.resets_at = reset;
        self.updated_at = Utc::now();
        if old_reset.is_some() && reset.is_some() && old_reset != reset && usage < old_usage {
            self.transition(Phase::Reset);
            self.semantic_attempted = false;
            self.semantic_checkpoint_sequence = None;
            self.preparation_semantic_attempted = false;
            self.emergency_semantic_attempted = false;
            self.final_handoff_path = None;
            return;
        }
        let Some(pct) = usage else { return };
        let target = if pct >= config.hard_stop_above_percentage {
            Phase::HardStopped
        } else if pct >= config.handoff_above_percentage {
            Phase::HandoffRequired
        } else if pct >= config.prepare_above_percentage {
            Phase::Preparing
        } else {
            Phase::Normal
        };
        if !matches!(self.phase, Phase::Finalized | Phase::Exhausted)
            || target == Phase::HardStopped
        {
            self.transition(target);
        }
    }

    pub fn transition(&mut self, target: Phase) {
        if self.phase != target {
            self.previous_phase = Some(self.phase);
            self.phase = target;
            self.updated_at = Utc::now();
        }
    }
}

pub struct StateStore {
    root: PathBuf,
}

impl StateStore {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }
    fn path(&self, id: &str) -> PathBuf {
        self.root.join("sessions").join(format!("{id}.json"))
    }
    fn lock_path(&self, id: &str) -> PathBuf {
        self.root.join("locks").join(format!("{id}.lock"))
    }

    pub fn with_locked<T>(
        &self,
        id: &str,
        f: impl FnOnce(&mut SessionState) -> Result<T>,
        cwd: &Path,
    ) -> Result<T> {
        fs::create_dir_all(self.root.join("sessions"))?;
        fs::create_dir_all(self.root.join("locks"))?;
        let lock = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(self.lock_path(id))?;
        lock.lock_exclusive().context("acquire session lock")?;
        let mut state = self
            .load(id)?
            .unwrap_or_else(|| SessionState::new(id.to_owned(), cwd.to_path_buf()));
        let result = f(&mut state);
        if result.is_ok() {
            self.save(&state)?;
        }
        FileExt::unlock(&lock)?;
        result
    }

    pub fn load(&self, id: &str) -> Result<Option<SessionState>> {
        let path = self.path(id);
        if !path.exists() {
            return Ok(None);
        }
        Ok(Some(serde_json::from_slice(&fs::read(path)?)?))
    }

    pub fn save(&self, state: &SessionState) -> Result<()> {
        atomic_write_json(&self.path(&state.session_id), state)
    }

    pub fn list(&self) -> Result<Vec<SessionState>> {
        let dir = self.root.join("sessions");
        if !dir.exists() {
            return Ok(vec![]);
        }
        let mut out = vec![];
        for entry in fs::read_dir(dir)? {
            let path = entry?.path();
            if path.extension().and_then(|x| x.to_str()) == Some("json") {
                if let Ok(value) = serde_json::from_slice(&fs::read(path)?) {
                    out.push(value);
                }
            }
        }
        out.sort_by_key(|s: &SessionState| std::cmp::Reverse(s.updated_at));
        Ok(out)
    }
}

pub fn atomic_write(path: &Path, bytes: &[u8]) -> Result<()> {
    let parent = path.parent().context("output path has no parent")?;
    fs::create_dir_all(parent)?;
    AtomicFile::new(path, AllowOverwrite).write(|file| {
        file.write_all(bytes)?;
        file.sync_all()
    })?;
    if let Ok(dir) = File::open(parent) {
        let _ = dir.sync_all();
    }
    Ok(())
}

pub fn atomic_write_json(path: &Path, value: &impl Serialize) -> Result<()> {
    atomic_write(path, &serde_json::to_vec_pretty(value)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn threshold_jumps_are_band_based() {
        let c = Config::default();
        let mut s = SessionState::new("x".into(), PathBuf::from("."));
        s.observe_usage(Some(84.0), Some(1), &c);
        assert_eq!(s.phase, Phase::Normal);
        s.observe_usage(Some(91.0), Some(1), &c);
        assert_eq!(s.phase, Phase::HandoffRequired);
        s.observe_usage(Some(100.0), Some(1), &c);
        assert_eq!(s.phase, Phase::HardStopped);
    }
    #[test]
    fn reset_is_detected() {
        let c = Config::default();
        let mut s = SessionState::new("x".into(), PathBuf::from("."));
        s.observe_usage(Some(92.0), Some(1), &c);
        s.observe_usage(Some(2.0), Some(2), &c);
        assert_eq!(s.phase, Phase::Reset);
    }
}
