# Changelog

All notable changes follow Semantic Versioning.

## [Unreleased]

## [0.2.1] - 2026-07-13

### Fixed
- Transcript auto-discovery: when the client hands the hook a `transcript_path`
  that does not exist on disk (observed in the Claude Code desktop app, where
  the hook's session id can differ from the real transcript filename), the
  engine now locates the actual transcript under
  `~/.claude/projects/<encoded-cwd>/*.jsonl` (preferring a session-id match,
  else the most recent) and records it. Previously this left the handoff with
  "User Goal: Not recoverable from transcript" and empty history.

### Added
- On-demand genuine summary: `/handoff-now:now` now writes the deterministic
  package **and** invokes the `handoff-writer` agent to produce a real narrative
  `SUMMARY.md`, then promotes it. New `handoff-now promote` verb validates and
  installs the agent's candidate.
- The `handoff-writer` agent now reads the full both-sides `CHAT-HISTORY` and is
  pinned to **Haiku 4.5 on the user's subscription** (no API key) so enrichment
  costs the least possible against the five-hour limit it is protecting; the
  deterministic package (zero tokens) remains the recovery guarantee, and no
  model runs at hard-stop.

## [0.2.0] - 2026-07-13

### Fixed
- Long-session recovery: transcript rendering now keeps a head+tail window so
  the *current* user goal (transcript tail) survives the safety cap instead of
  being silently dropped in favour of a stale early message.
- Stopped journal inflation: renders without `rate_limits` no longer append a
  null `UsageObserved` event or inflate `journalSequence`.
- Unified journal-sequence provenance between the hook and API promote paths.

### Added
- Automatic engine updates: on session start the dispatch hook compares the
  installed engine version with the plugin version and, on mismatch, fetches the
  matching release in the background (checksum-verified, rate-limited, offline
  safe). Existing users no longer stay pinned to an old engine — no manual
  command. Binary replacement is Windows-safe (renames a running `.exe` aside).
  `doctor` now reports `engineVersion`.
- Deterministic snapshots on `SessionEnd` and `PreCompact` so graceful exits
  and context compaction always leave a fresh handoff.
- Rolling handoff on every `Stop` (config `rollingHandoff`, default on) — the
  artifact is never more than one turn stale.
- `resume` rebuilds a deterministic handoff from `EVENTS.jsonl` when no handoff
  file exists (e.g. a hard kill before any snapshot).
- Burn-rate predictor: the status line shows an estimated `~Nm to wall`, and a
  spike guard (`spikeSnapshotDelta`, default 15) forces a snapshot on a large
  single-render usage jump.
- Bounded session-lock acquisition so a wedged holder can no longer hang hooks.
- Stop-time reconciliation promotes a valid on-disk semantic candidate even if a
  subagent's `PostToolUse` fired under a different session id.
- Strict redaction mode now active: entropy-based detection of opaque
  mixed-case tokens that pattern matching misses (public model ids, UUIDs, and
  hashes stay readable).
- New commands: `verify` (integrity/tamper check), `export` (portable
  `RESUME.md`), `tail` (recent journal), and `doctor --fix` (auto-repair).

## [0.1.1] - 2026-07-12

- Preserve the last known five-hour usage/reset across status-line renders instead of clobbering it to "unknown" when `rate_limits` is momentarily absent; display the retained value in the status line.
- Extract the real last user message for the handoff goal instead of a synthetic `[Tool result recorded]` tool-result echo.
- Prefer the session in the current working directory for a manual `/handoff-now:now` or `:resume`, so it snapshots the project you're in.
- Auto-bootstrap the native engine on first run so `/plugin marketplace add` + `/plugin install` is a true two-command install.

## [0.1.0] - 2026-07-12

- Initial Claude Code/Desktop plugin architecture.
- Official status-line quota observation and threshold state machine.
- Append-only journal, deterministic handoff artifacts, redacted history, and integrity hashes.
- Protected-mode hook enforcement and `StopFailure` recovery.
- Optional bounded Haiku 4.5 semantic provider.
- GitHub marketplace and cross-platform release workflows.
