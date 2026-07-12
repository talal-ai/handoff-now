---
name: setup
description: Install or repair the handoff-now quota watcher and preserve the user's existing Claude status line.
allowed-tools: Bash
---

If the `handoff-now` binary is already installed, run `handoff-now setup`, then `handoff-now doctor`. If it is not installed yet (or `doctor` reports it missing), run `sh "${CLAUDE_PLUGIN_ROOT}/scripts/bootstrap.sh"` instead — it downloads the correct release for the user's OS/architecture, verifies its SHA-256 checksum against the published `SHA256SUMS`, installs it, and runs `setup` and `doctor` itself. Either way, show the user the final `doctor` result. Do not modify Claude settings by hand; `setup` performs a backed-up, idempotent update. If bootstrap fails (no network, unsupported platform), show the exact error and point to the "Development" section of the README for building from source as a fallback.
