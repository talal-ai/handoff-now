# Handoff Now

<p align="center"><img src="assets/logo.svg" width="112" alt="Handoff Now shield and handoff arrow"></p>

<p align="center"><strong>Never lose a long Claude Code session to the 5-hour usage limit.</strong></p>

<p align="center">
  <a href="https://github.com/talal-ai/handoff-now/actions/workflows/ci.yml"><img alt="CI" src="https://github.com/talal-ai/handoff-now/actions/workflows/ci.yml/badge.svg"></a>
  <a href="LICENSE"><img alt="Apache-2.0" src="https://img.shields.io/badge/license-Apache--2.0-blue.svg"></a>
  <a href="https://github.com/talal-ai/handoff-now/releases"><img alt="Release" src="https://img.shields.io/github/v/release/talal-ai/handoff-now?include_prereleases"></a>
  <img alt="Telemetry off" src="https://img.shields.io/badge/telemetry-off-10b981.svg">
</p>

`handoff-now` is an open-source, quota-aware recovery plugin for Claude Code CLI and Claude Desktop local/SSH sessions. It watches Claude Code's documented five-hour usage signal, checkpoints at 85%, freezes new source mutations at 90%, and creates a redacted, integrity-checked package for the next session.

**The recovery guarantee never depends on another model request.**

## Install from source

This is the supported early-access path until the signed native binaries are published:

```powershell
git clone https://github.com/talal-ai/handoff-now
cd handoff-now
cargo build --release
./target/release/handoff-now.exe setup
claude --plugin-dir .
```

On macOS, replace `handoff-now.exe` with `handoff-now`. Inside Claude, run `/handoff-now:doctor`.

## Marketplace registration

```text
/plugin marketplace add https://github.com/talal-ai/handoff-now
/plugin install handoff-now@handoff-now-marketplace
```

The marketplace installs the Claude plugin bundle. During early access, build and set up the native engine using the source instructions above. The one-command bootstrap will become the default after the release binaries are signed and notarized.

## Why this exists

Claude Code can report official five-hour usage only after an API response. When a long task reaches the limit, the current response can stop before Claude has enough allowance to explain what changed or how to resume. Handoff Now continuously records recoverable facts and intervenes at documented lifecycle boundaries before that cliff.

It does not claim to interrupt an unfinished response or kill an active command. It finishes the current boundary, blocks the next unsafe action, and preserves the task.

## How it works

| Official five-hour usage | Behavior |
| --- | --- |
| `< 85%` | Journal factual events. |
| `>= 85%` and `< 90%` | Create a deterministic checkpoint and attempt bounded semantic preparation. |
| `>= 90%` and `< 95%` | Freeze normal development at the next lifecycle boundary and finalize the handoff. |
| `>= 95%` | Make no recovery-dependent model request; stop with the verified local handoff. |

Every project gets `.handoff-now/<session-id>/` with:

- `HANDOFF.md` — goals, decisions, verified progress, risks, remaining work, and exact resume action.
- `CHAT-HISTORY.redacted.md` — useful conversational history with detected credentials removed.
- `EVENTS.jsonl` — append-only factual journal.
- `working-changes.patch`, `git-status.txt`, `FILES.md`, and `TESTS.md` — repository evidence.
- `integrity.json` — SHA-256 hashes and journal sequence numbers.

## What makes it different

| Capability | Handoff Now | Manual handoff prompt | Transcript exporter |
| --- | --- | --- | --- |
| Uses official five-hour usage signal | Yes | No | No |
| Prepares before exhaustion | Automatic | Only when remembered | Usually after the fact |
| Blocks new risky source mutations | Yes | No | No |
| Works when no model call remains | Yes | No | Sometimes |
| Separates facts from model interpretation | Yes | Unstructured | Varies |
| Local redaction and integrity manifest | Yes | No | Varies |

## Requirements

- Claude Code 2.1.206 or newer is recommended.
- Windows 10/11 x64 or ARM64, or macOS 13+ Intel/Apple Silicon.
- Git is optional but recommended.
- `ANTHROPIC_API_KEY` is optional. It enables an isolated Haiku API summary; never store it in `config.json`.

## Development checkout

```powershell
git clone https://github.com/talal-ai/handoff-now
cd handoff-now
cargo build --release
./target/release/handoff-now.exe setup
claude --plugin-dir .
```

On macOS, replace `handoff-now.exe` with `handoff-now`.

Inside Claude Code or Desktop, run `/handoff-now:doctor`. Restart Claude after the first setup so the status-line trust prompt and hooks load consistently.

The setup command preserves and chains an existing status line, creates a timestamped settings backup, and installs a stable watcher under `~/.claude/handoff-now/`.

## Commands

- `/handoff-now:setup` — install or repair the watcher.
- `/handoff-now:status` — show session state and usage bands.
- `/handoff-now:now` — create a handoff immediately.
- `/handoff-now:resume` — generate a verified resume prompt.
- `/handoff-now:doctor` — check installation health.
- `/handoff-now:configure` — locate and explain configuration.
- `/handoff-now:uninstall` — restore the previous status line without deleting recovery data.

To keep an isolated Haiku API credential in Windows Credential Manager or macOS Keychain, pipe it to the native CLI so it never appears in command history:

```powershell
$env:ANTHROPIC_API_KEY | handoff-now credential store
```

The environment variable takes precedence over the OS credential store.

## Artifacts

Each project receives `.handoff-now/<session-id>/` containing `HANDOFF.md`, redacted history, an append-only event journal, Git state/diff, test evidence, session metadata, and SHA-256 integrity hashes. `.handoff-now/` is added to `.git/info/exclude`, not the tracked `.gitignore`.

Raw transcript copying is off by default. The source transcript remains in Claude Code's own storage unless the user explicitly enables `retainRawTranscript`.

## Configuration

Edit `~/.claude/handoff-now/config.json`. Thresholds must satisfy `prepare < handoff < hard stop`; invalid configuration activates safe defaults and writes a local diagnostic. See [`schemas/config.schema.json`](schemas/config.schema.json).

## What this plugin does not do

- It does not scrape Claude OAuth credentials or private usage endpoints.
- It does not promise token-by-token interruption.
- It does not force-kill active commands.
- It does not upload telemetry.
- It does not guarantee that pattern-based redaction detects every possible secret.

Read [`SECURITY.md`](SECURITY.md) before enabling semantic API summaries.

## Trust, privacy, and limitations

- Local-first; telemetry and raw transcript retention are off by default.
- No OAuth scraping and no undocumented Claude subscription endpoints.
- Optional isolated Haiku summaries require your own API credential; deterministic recovery does not.
- Redaction is best effort. Review artifacts before sharing them.
- Protected-mode path checks reject traversal and symlink/junction escape attempts.

Read the [threat model](docs/threat-model.md), [architecture](docs/architecture.md), and [security policy](SECURITY.md).

## Project status

The plugin is in early public release. Windows x64/ARM64 and macOS Intel/Apple Silicon builds are exercised in CI. Please report reproducible failures using the issue templates—never attach raw transcripts or credentials.

If Handoff Now successfully saves a real task, share the recovery scenario in [Discussions](https://github.com/talal-ai/handoff-now/discussions). Those reports guide the roadmap.

## Development

```text
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
claude plugin validate .
```

Licensed under Apache-2.0.
