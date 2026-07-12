# handoff-now

`handoff-now` is a quota-aware, crash-safe recovery plugin for Claude Code and Claude Desktop. It journals factual session events, creates deterministic handoff packages before a five-hour allowance is exhausted, and optionally uses Claude Haiku 4.5 to enrich the summary.

The recovery guarantee never depends on another model request.

## Safety model

| Official five-hour usage | Behavior |
| --- | --- |
| `< 85%` | Journal factual events. |
| `>= 85%` and `< 90%` | Create a deterministic checkpoint and attempt bounded semantic preparation. |
| `>= 90%` and `< 95%` | Freeze normal development at the next lifecycle boundary and finalize the handoff. |
| `>= 95%` | Do not start another model request; stop with the verified local handoff. |

Claude Code reports usage only after an API response. No plugin can reliably interrupt an unfinished model response or an already-running tool. `handoff-now` therefore checkpoints early and takes control at documented hook boundaries.

## Requirements

- Claude Code 2.1.206 or newer is recommended.
- Windows 10/11 x64 or ARM64, or macOS 13+ Intel/Apple Silicon.
- Git is optional but recommended.
- `ANTHROPIC_API_KEY` is optional. It enables an isolated Haiku API summary; never store it in `config.json`.

## Local development install

```powershell
git clone https://github.com/YOUR-GITHUB-OWNER/handoff-now
cd handoff-now
cargo build --release
./target/release/handoff-now.exe setup
claude --plugin-dir .
```

On macOS, replace `handoff-now.exe` with `handoff-now`.

Inside Claude Code or Desktop, run `/handoff-now:doctor`. Restart Claude after the first setup so the status-line trust prompt and hooks load consistently.

## Marketplace install

```text
/plugin marketplace add https://github.com/YOUR-GITHUB-OWNER/handoff-now
/plugin install handoff-now@handoff-now-marketplace
/handoff-now:setup
```

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

## Development

```text
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
claude plugin validate .
```

Licensed under Apache-2.0.
