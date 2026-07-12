# Changelog

All notable changes follow Semantic Versioning.

## [Unreleased]

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
