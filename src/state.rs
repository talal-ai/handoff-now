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
    /// Recent `(unix_seconds, usage_percentage)` samples for the deterministic
    /// burn-rate predictor (Phase 3.1). Bounded ring; oldest evicted.
    #[serde(default)]
    pub usage_samples: Vec<(i64, f64)>,
}

/// Maximum retained usage samples for burn-rate estimation.
const MAX_USAGE_SAMPLES: usize = 24;

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
            usage_samples: Vec::new(),
        }
    }

    /// Record a usage sample and return the jump (points) versus the previous
    /// reading, if any. Used by the spike guard.
    pub fn record_sample(&mut self, pct: f64) -> Option<f64> {
        let now = Utc::now().timestamp();
        let jump = self.usage_samples.last().map(|(_, prev)| pct - prev);
        self.usage_samples.push((now, pct));
        if self.usage_samples.len() > MAX_USAGE_SAMPLES {
            let overflow = self.usage_samples.len() - MAX_USAGE_SAMPLES;
            self.usage_samples.drain(0..overflow);
        }
        jump
    }

    /// Deterministic burn rate in usage-points per minute over the retained
    /// samples (least-squares slope). `None` until there is enough signal.
    pub fn burn_rate_per_minute(&self) -> Option<f64> {
        let pts = &self.usage_samples;
        if pts.len() < 3 {
            return None;
        }
        let t0 = pts[0].0 as f64;
        let xs: Vec<f64> = pts.iter().map(|(t, _)| (*t as f64 - t0) / 60.0).collect();
        let ys: Vec<f64> = pts.iter().map(|(_, p)| *p).collect();
        let n = xs.len() as f64;
        let sx: f64 = xs.iter().sum();
        let sy: f64 = ys.iter().sum();
        let sxx: f64 = xs.iter().map(|x| x * x).sum();
        let sxy: f64 = xs.iter().zip(&ys).map(|(x, y)| x * y).sum();
        let denom = n * sxx - sx * sx;
        if denom.abs() < f64::EPSILON {
            return None;
        }
        let slope = (n * sxy - sx * sy) / denom;
        (slope > 0.0).then_some(slope)
    }

    /// Estimated minutes until `target` usage at the current burn rate.
    pub fn minutes_to(&self, target: f64) -> Option<f64> {
        let rate = self.burn_rate_per_minute()?;
        let current = self.usage_percentage?;
        if current >= target {
            return Some(0.0);
        }
        Some((target - current) / rate)
    }

    pub fn observe_usage(&mut self, usage: Option<f64>, reset: Option<i64>, config: &Config) {
        self.updated_at = Utc::now();

        // The status line fires on every render, but `rate_limits` is only
        // present for Pro/Max subscribers after the first API response and
        // may be absent on any individual render. When a render carries no
        // reading, preserve the last known usage/reset instead of clobbering
        // them back to "unknown" (which is what surfaced as
        // "Five-hour usage: unknown" in handoffs).
        let Some(pct) = usage else {
            return;
        };

        let old_reset = self.resets_at;
        let old_usage = self.usage_percentage;
        self.usage_percentage = Some(pct);
        self.record_sample(pct);
        if reset.is_some() {
            self.resets_at = reset;
        }
        if old_reset.is_some()
            && reset.is_some()
            && old_reset != reset
            && old_usage.is_some_and(|old| pct < old)
        {
            self.transition(Phase::Reset);
            self.semantic_attempted = false;
            self.semantic_checkpoint_sequence = None;
            self.preparation_semantic_attempted = false;
            self.emergency_semantic_attempted = false;
            self.final_handoff_path = None;
            self.usage_samples.clear();
            self.record_sample(pct);
            return;
        }
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
        // Bounded acquisition (Phase 2.4). Hooks run under a timeout; a wedged
        // or orphaned holder must not make every tool call hang. Try for a few
        // seconds, then proceed best-effort so the journal/state still advance
        // (atomic writes keep the state file consistent regardless).
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
        let mut held = false;
        loop {
            match lock.try_lock_exclusive() {
                Ok(()) => {
                    held = true;
                    break;
                }
                Err(_) if std::time::Instant::now() < deadline => {
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
                Err(_) => {
                    eprintln!(
                        "handoff-now: session lock busy after timeout; proceeding best-effort"
                    );
                    break;
                }
            }
        }
        let mut state = self
            .load(id)?
            .unwrap_or_else(|| SessionState::new(id.to_owned(), cwd.to_path_buf()));
        let result = f(&mut state);
        if result.is_ok() {
            self.save(&state)?;
        }
        if held {
            FileExt::unlock(&lock)?;
        }
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
    #[test]
    fn absent_render_does_not_change_effective_reading() {
        // Regression for journal inflation: after one real reading, renders
        // without rate_limits must leave the effective usage/reset unchanged
        // so the statusline handler appends no spurious event.
        let c = Config::default();
        let mut s = SessionState::new("x".into(), PathBuf::from("."));
        s.observe_usage(Some(50.0), Some(1), &c);
        let (u, r, p) = (s.usage_percentage, s.resets_at, s.phase);
        for _ in 0..100 {
            s.observe_usage(None, None, &c);
        }
        assert_eq!((s.usage_percentage, s.resets_at, s.phase), (u, r, p));
    }
    #[test]
    fn burn_rate_and_eta_are_positive_when_rising() {
        let mut s = SessionState::new("x".into(), PathBuf::from("."));
        // Synthesize rising samples ~1 point/sample.
        let base = Utc::now().timestamp();
        for i in 0..6 {
            s.usage_samples.push((base + i * 60, 50.0 + i as f64));
        }
        s.usage_percentage = Some(55.0);
        let rate = s.burn_rate_per_minute().expect("rate");
        assert!(rate > 0.0);
        let eta = s.minutes_to(95.0).expect("eta");
        assert!(eta > 0.0 && eta.is_finite());
    }
    #[test]
    fn flat_usage_has_no_burn_rate() {
        let mut s = SessionState::new("x".into(), PathBuf::from("."));
        let base = Utc::now().timestamp();
        for i in 0..6 {
            s.usage_samples.push((base + i * 60, 50.0));
        }
        assert!(s.burn_rate_per_minute().is_none());
    }
    #[test]
    fn absent_reading_preserves_last_known_usage() {
        // Reproduces the reported "Five-hour usage: unknown" bug: a status
        // line render without `rate_limits` must not wipe a known reading.
        let c = Config::default();
        let mut s = SessionState::new("x".into(), PathBuf::from("."));
        s.observe_usage(Some(50.0), Some(1), &c);
        assert_eq!(s.usage_percentage, Some(50.0));
        s.observe_usage(None, None, &c);
        assert_eq!(s.usage_percentage, Some(50.0));
        assert_eq!(s.resets_at, Some(1));
        assert_eq!(s.phase, Phase::Normal);
    }
}
