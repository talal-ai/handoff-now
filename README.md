# Handoff Now

<p align="center"><img src="assets/logo.svg" width="112" alt="Handoff Now shield and handoff arrow"></p>

<p align="center"><strong>🛟 Never lose a long Claude Code session to the 5-hour usage limit.</strong></p>

<p align="center">
  <a href="https://github.com/talal-ai/handoff-now/actions/workflows/ci.yml"><img alt="CI" src="https://github.com/talal-ai/handoff-now/actions/workflows/ci.yml/badge.svg"></a>
  <a href="LICENSE"><img alt="Apache-2.0" src="https://img.shields.io/badge/license-Apache--2.0-blue.svg"></a>
  <a href="https://github.com/talal-ai/handoff-now/releases"><img alt="Release" src="https://img.shields.io/github/v/release/talal-ai/handoff-now?include_prereleases"></a>
  <img alt="Telemetry off" src="https://img.shields.io/badge/telemetry-off-10b981.svg">
</p>

`handoff-now` is an open-source, quota-aware recovery plugin for Claude Code CLI and Claude Desktop local/SSH sessions. It watches Claude Code's documented five-hour usage signal, checkpoints at 85%, freezes new source mutations at 90%, and creates a redacted, integrity-checked package for the next session.

**🧭 The recovery guarantee never depends on another model request.**

## ✨ Features

- 📉 **Reads the real quota, not a guess** — uses Claude Code's documented five-hour usage signal directly; no token-counting heuristics.
- 🧾 **Checkpoints before you hit the wall** — journals events continuously, then writes a deterministic checkpoint at 85% usage.
- 🚧 **Freezes risky writes automatically** — blocks new source mutations at 90% instead of letting a task get cut off mid-edit.
- 🧩 **Always names a working resume point** — even at ≥95% usage, with zero model calls left, it hands you a verified local handoff.
- 🕵️ **Redacts before it saves** — detected credentials are stripped from conversational history before anything touches disk.
- 🔒 **Integrity-checked artifacts** — every handoff ships SHA-256 hashes and journal sequence numbers so you can trust what you're resuming from.
- 🙈 **No telemetry, ever** — local-first by design; raw transcript retention is opt-in and off by default.
- ⚡ **Zero-friction install** — two `/plugin` commands and the native engine fetches, verifies, and installs itself automatically.

## 🚀 Install

```text
/plugin marketplace add https://github.com/talal-ai/handoff-now
/plugin install handoff-now@handoff-now-marketplace
```

That's it. The first time Claude Code starts with the plugin installed, it
automatically downloads the correct native engine for your OS/CPU, verifies
its SHA-256 checksum against the published `SHA256SUMS`, and installs it in
the background — no Rust, no manual build, no separate download step.
Restart Claude Code once after the first install so the status-line trust
prompt and hooks load consistently, then run `/handoff-now:doctor` to
confirm it's healthy.

The release binaries are not code-signed (that costs money and isn't
required to run them). On first execution, macOS or Windows may show a
one-time "unknown publisher" warning — this is normal for unsigned
open-source tools and only appears once. Click through it
("More info → Run anyway" on Windows) and it won't reappear. If you'd rather
verify the binary yourself first, build from source — see **Development**
near the bottom of this README.

## 🤔 Why this exists

Claude Code can report official five-hour usage only after an API response. When a long task reaches the limit, the current response can stop before Claude has enough allowance to explain what changed or how to resume. Handoff Now continuously records recoverable facts and intervenes at documented lifecycle boundaries before that cliff.

It does not claim to interrupt an unfinished response or kill an active command. It finishes the current boundary, blocks the next unsafe action, and preserves the task.

## 🧠 How it works

| Official five-hour usage | Behavior |
| --- | --- |
| `< 85%` | 🗒️ Journal factual events. |
| `>= 85%` and `< 90%` | 📍 Create a deterministic checkpoint and attempt bounded semantic preparation. |
| `>= 90%` and `< 95%` | 🚧 Freeze normal development at the next lifecycle boundary and finalize the handoff. |
| `>= 95%` | 🛑 Make no recovery-dependent model request; stop with the verified local handoff. |

## ⚖️ What makes it different

| Capability | Handoff Now | Manual handoff prompt | Transcript exporter |
| --- | --- | --- | --- |
| Uses official five-hour usage signal | ✅ Yes | ❌ No | ❌ No |
| Prepares before exhaustion | ✅ Automatic | ⚠️ Only when remembered | ⚠️ Usually after the fact |
| Blocks new risky source mutations | ✅ Yes | ❌ No | ❌ No |
| Works when no model call remains | ✅ Yes | ❌ No | ⚠️ Sometimes |
| Separates facts from model interpretation | ✅ Yes | ❌ Unstructured | ⚠️ Varies |
| Local redaction and integrity manifest | ✅ Yes | ❌ No | ⚠️ Varies |

## ✅ Requirements

- Claude Code 2.1.206 or newer is recommended.
- Windows 10/11 x64 or ARM64, or macOS 13+ Intel/Apple Silicon.
- Internet access on first run, to download the native engine once.
- Git is optional but recommended.
- `ANTHROPIC_API_KEY` is optional. It enables an isolated Haiku API summary; never store it in `config.json`.

The `setup` step preserves and chains an existing status line, creates a timestamped settings backup, and installs a stable watcher under `~/.claude/handoff-now/`.

## 🛠️ Commands

| Command | What it does |
| --- | --- |
| `/handoff-now:setup` | 🔧 Install or repair the watcher (fetches the engine automatically if missing). |
| `/handoff-now:status` | 📊 Show session state and usage bands. |
| `/handoff-now:now` | 📸 Create a handoff immediately. |
| `/handoff-now:resume` | ▶️ Generate a verified resume prompt (rebuilds from the journal if no handoff exists). |
| `/handoff-now:doctor` | 🩺 Check installation health (`doctor --fix` auto-repairs). |
| `/handoff-now:configure` | ⚙️ Locate and explain configuration. |
| `/handoff-now:uninstall` | 🧹 Restore the previous status line without deleting recovery data. |

Additional native CLI verbs (also usable directly): `handoff-now verify`
(integrity/tamper check against `integrity.json`), `handoff-now export`
(write a portable self-contained `RESUME.md`), and `handoff-now tail` (recent
journal events). See [`docs/recovery-scenarios.md`](docs/recovery-scenarios.md)
for step-by-step "it died — now what" walkthroughs.

To keep an isolated Haiku API credential in Windows Credential Manager or macOS Keychain, pipe it to the native CLI so it never appears in command history:

```powershell
$env:ANTHROPIC_API_KEY | handoff-now credential store
```

The environment variable takes precedence over the OS credential store.

## 📦 Artifacts

Every project gets `.handoff-now/<session-id>/` with:

- `HANDOFF.md` — goals, decisions, verified progress, risks, remaining work, and exact resume action.
- `CHAT-HISTORY.redacted.md` — useful conversational history with detected credentials removed.
- `EVENTS.jsonl` — append-only factual journal.
- `working-changes.patch`, `git-status.txt`, `FILES.md`, and `TESTS.md` — repository evidence.
- `integrity.json` — SHA-256 hashes and journal sequence numbers.

`.handoff-now/` is added to `.git/info/exclude`, not the tracked `.gitignore`. Raw transcript copying is off by default — the source transcript stays in Claude Code's own storage unless you explicitly enable `retainRawTranscript`.

## ⚙️ Configuration

Edit `~/.claude/handoff-now/config.json`. Thresholds must satisfy `prepare < handoff < hard stop`; invalid configuration activates safe defaults and writes a local diagnostic. See [`schemas/config.schema.json`](schemas/config.schema.json).

## 🚫 What this plugin does not do

- It does not scrape Claude OAuth credentials or private usage endpoints.
- It does not promise token-by-token interruption.
- It does not force-kill active commands.
- It does not upload telemetry.
- It does not guarantee that pattern-based redaction detects every possible secret.

Read [`SECURITY.md`](SECURITY.md) before enabling semantic API summaries.

## 🛡️ Trust, privacy, and limitations

- Local-first; telemetry and raw transcript retention are off by default.
- No OAuth scraping and no undocumented Claude subscription endpoints.
- Optional isolated Haiku summaries require your own API credential; deterministic recovery does not.
- Redaction is best effort. Review artifacts before sharing them.
- Protected-mode path checks reject traversal and symlink/junction escape attempts.

Read the [threat model](docs/threat-model.md), [architecture](docs/architecture.md), and [security policy](SECURITY.md).

## 📊 Project status

The plugin is in early public release. Windows x64/ARM64 and macOS Intel/Apple Silicon builds are exercised in CI. Please report reproducible failures using the issue templates—never attach raw transcripts or credentials.

If Handoff Now successfully saves a real task, share the recovery scenario in [Discussions](https://github.com/talal-ai/handoff-now/discussions). Those reports guide the roadmap.

## 🧪 Development

To build and run from source instead of the marketplace install — useful for
contributors, or if you'd rather verify the binary yourself before trusting
it:

```powershell
git clone https://github.com/talal-ai/handoff-now
cd handoff-now
cargo build --release
./target/release/handoff-now.exe setup
claude --plugin-dir .
```

On macOS, replace `handoff-now.exe` with `handoff-now`. Inside Claude, run `/handoff-now:doctor`.

Before submitting changes:

```text
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
claude plugin validate .
```

See [`CONTRIBUTING.md`](CONTRIBUTING.md) for the full contribution checklist and [`SECURITY.md`](SECURITY.md) to report a vulnerability privately.

## 📄 License

Licensed under [Apache-2.0](LICENSE).
