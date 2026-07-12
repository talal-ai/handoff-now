# Anthropic plugin directory submission

## Listing copy

**Name:** Handoff Now

**Summary:** Quota-aware, crash-safe recovery for Claude Code and Desktop.

**Description:** Handoff Now monitors Claude Code's documented five-hour usage signal, creates deterministic checkpoints before exhaustion, prevents new source mutations in the emergency band, and writes a redacted, integrity-checked package for the next session. It works without an API key; an optional isolated Haiku request can improve narrative quality but is never required for recovery.

**Category:** Developer tools / Productivity

**Repository:** https://github.com/talal-ai/handoff-now

**Support:** https://github.com/talal-ai/handoff-now/issues

**Privacy:** Local-first, telemetry disabled, raw transcript retention disabled by default.

## Review notes

- Reads the documented `rate_limits.five_hour` status-line field.
- Uses documented lifecycle hooks and does not scrape OAuth tokens or private endpoints.
- Cannot interrupt an in-flight response or already-running command; intervention occurs at lifecycle boundaries.
- Deterministic artifacts are generated before any optional model request.
- Protected mode restricts writes to the canonical handoff directory.
- Apache-2.0 licensed; security policy and threat model are public.

Before submission, replace the draft release with a public release and attach the repository social preview image.
