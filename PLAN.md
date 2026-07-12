# Handoff Now — Master Plan

> Mission: make `HANDOFF.md` a recovery guarantee a developer can bet a 5-hour
> session on. Never depend on a model call. Never lose the current goal. Fail
> only in ways that still leave a usable, honest artifact behind.

This plan folds together every confirmed finding from the code review plus a
forward vision for the Claude Code developer community. It is ordered by
**value / risk**, not by ease. Each item lists: what, why, code touch points,
acceptance criteria.

> **Status (v0.2.0):** Phases 1–4 shipped. 1.1–1.5, 2.1–2.4, 3.1–3.2, 4.1–4.4
> all landed with regression tests. Phase 5 partial: `doctor --fix`, portable
> `export`, and `docs/recovery-scenarios.md` done; remaining Phase 5 items
> (handoff templates, `diff`, local success counter) are still open.

---

## 0. Guiding principles (do not violate)

1. **Determinism is the floor.** Every recovery path must work with zero model
   calls. Semantic enrichment is always additive, never load-bearing.
2. **The journal is the source of truth.** `EVENTS.jsonl` is append-only,
   fsync'd, monotonic. Everything else is a rendered view of it.
3. **Newest wins.** When we must drop data, drop the oldest, never the most
   recent goal/state.
4. **Honest artifacts.** Never claim tests passed, work done, or a handoff is
   fresh when it is not. Timestamp and provenance everything.
5. **Local-first, no telemetry.** New features cannot phone home.
6. **Idempotent + crash-safe.** Any step can run twice or die mid-way and the
   next run recovers.

---

## 1. Confirmed bug fixes (Phase 1 — ship first)

### 1.1 🔴 Long-session tail truncation (the critical one)
- **Bug:** `extract_transcript` reads from offset 0 and keeps the first ~480KB
  (`maximum_semantic_input_bytes * 4`), dropping the newest turns. In a real
  5-hour session the transcript far exceeds this, so `latest_real_user_message`
  returns a *stale* goal. Recovery names the wrong task.
- **Files:** `src/artifacts.rs:136-180`, `:304`.
- **Fix:** Read a **head + tail window**. Keep first N/4 (early goal/setup) and
  last 3N/4 (recent work + current goal), with an explicit
  `> [middle elided K bytes]` marker between them. `latest_real_user_message`
  then searches the retained tail.
- **Accept:** New test — synthetic transcript > cap with the real goal in the
  last 10% still surfaces as the User Goal. Head+tail both present in output.

### 1.2 🟠 `last_transcript_offset` dead — wire true incremental journaling
- **Bug:** Field written (`artifacts.rs:178`), read nowhere. The intended
  incremental read was never finished.
- **Fix:** Use it to append only *new* transcript records to a rolling
  `CHAT-HISTORY.redacted.md` between snapshots, and to cheaply refresh the
  tail. Ties into 1.1 and 2.2.
- **Accept:** Second snapshot on an unchanged transcript does no re-read;
  appended turns show up without re-processing old ones.

### 1.3 🟠 Journal inflation on empty renders
- **Bug:** `engine.rs:76` — when a render lacks `rate_limits`, stored
  `Some(pct)` != incoming `None` → a null `UsageObserved` event is appended and
  `journal_sequence` inflated on every such render.
- **Fix:** Skip the event when incoming `usage.is_none() && reset.is_none()`
  and phase unchanged. Compare *effective* (post-observe) values, not raw
  incoming vs stored.
- **Accept:** Test — 100 renders with no `rate_limits` after one real reading
  append zero extra events.

### 1.4 🟡 Sequence provenance unification
- **Bug:** API path uses live `state.journal_sequence` (`engine.rs:492`); hook
  path uses frozen `SESSION.json/journalSequence` (`engine.rs:286`).
- **Fix:** Single helper `expected_sequence(out, state)` used by both paths;
  document which is authoritative (SESSION.json at snapshot time).
- **Accept:** Both promote paths validate against the same value in a test.

### 1.5 🟡 Subagent session_id assumption (verify, then guard)
- **Risk:** Emergency semantic promote assumes the `handoff-writer` subagent's
  `Write` fires the *parent* session's `PostToolUse`. Unverified against Claude
  Code semantics.
- **Fix:** (a) Add an integration/contract test that asserts the wiring; (b)
  regardless of subagent id, make the deterministic handoff already complete
  before the agent runs (it is) so a missed promote only loses *enrichment*,
  never recovery; (c) add a `Stop`-time reconciliation that scans `out/` for a
  valid `SUMMARY.candidate.md` and promotes it even if `PostToolUse` never
  fired for the parent.
- **Accept:** With `PostToolUse` suppressed, a valid candidate on disk is still
  promoted at `Stop`.

---

## 2. Real-time recovery hardening (Phase 2 — closes the "sudden 100%" hole)

### 2.1 Snapshot on `SessionEnd` and `PreCompact` unconditionally
- **Gap:** `hooks.json` wires `SessionEnd`, `PreCompact`, `PostCompact` but
  `handle_hook` has no case → append-only, no snapshot. Graceful end / context
  compaction leave no fresh handoff unless already in a high band.
- **Fix:** In `handle_hook`, snapshot on `SessionEnd` and `PreCompact`
  (deterministic, cheap). `PreCompact` is the ideal pre-context-loss checkpoint.
- **Accept:** A session that never crossed 85% but ends gracefully has a
  current `HANDOFF.md`.

### 2.2 Rolling cheap HANDOFF.md on every `Stop`
- **Idea:** Render a deterministic `HANDOFF.md` on every `Stop` (not only band
  crossings), from `EVENTS.jsonl` + git + tail transcript. Keeps the artifact
  ≤ one turn stale at all times, so even a later hard kill leaves a fresh one.
- **Cost:** One deterministic render per turn (no model). Gate behind a
  `rollingHandoff` config (default on).
- **Accept:** After any normal turn, `HANDOFF.md` mtime is current.

### 2.3 `resume()` fallback to the journal
- **Gap:** `resume()` (`main.rs:104`) requires `final_handoff_path`; if a hard
  kill happened before any snapshot, `EVENTS.jsonl` exists but resume errors.
- **Fix:** When no handoff file, reconstruct a minimal deterministic
  `HANDOFF.md` from `EVENTS.jsonl` on the fly, then resume from it.
- **Accept:** Delete `HANDOFF.md`, keep `EVENTS.jsonl` → `resume` still yields a
  usable prompt.

### 2.4 Stale-lock recovery
- **Risk:** A crashed process holding a `.lock` (advisory) — on some platforms a
  dead holder frees it, but orphaned detached `snapshot-session` children could
  wedge. Add lock-age detection + steal-with-warning after a timeout.
- **Accept:** A stale lock older than N seconds is reclaimed with a diagnostic.

---

## 3. Proactive intelligence (Phase 3 — see the wall coming)

### 3.1 Burn-rate predictor (deterministic, no model)
- **Idea:** Sample `(usage%, timestamp)` on each status-line render. Compute a
  short rolling slope → **estimated time until 95%**. Surface in the status
  line: `handoff-now 78% ~14m to wall`.
- **Why:** The core weakness is that status-line usage is one-response-stale and
  can jump. A velocity estimate lets us checkpoint *before* the spike, not
  after. Purely local math over samples already in state.
- **Extend:** Adaptive prep — if burn rate is steep, trigger the 85% checkpoint
  early (effective threshold lowers with velocity).
- **Accept:** Given a rising synthetic sample series, predicted-wall time is
  within tolerance and an early checkpoint fires.

### 3.2 Spike guard
- **Idea:** If a single render jumps usage by > X points, immediately force a
  deterministic snapshot regardless of band (a big jump is the exact danger
  case the user raised).
- **Accept:** A 40→96 single-step render produces a snapshot.

---

## 4. Trust, verification, portability (Phase 4 — make it auditable)

### 4.1 `handoff-now verify [SESSION_ID]`
- Recompute SHA-256 of every file, compare against `integrity.json`; check
  `journal_sequence` monotonicity and `SUMMARY.validated`. Report tamper /
  drift. Exit non-zero on mismatch.

### 4.2 `handoff-now export [SESSION_ID]`
- Emit a single self-contained `RESUME.md` (goal, state, diff summary, first
  action, redaction notes) that pastes into *any* fresh Claude Code session —
  or another agent — with no dependency on the `.handoff-now` dir. Portability =
  community adoption.

### 4.3 `handoff-now tail` / `list --rich`
- Human view of the live journal and all sessions across projects (phase,
  usage, last handoff, freshness). A local dashboard, zero telemetry.

### 4.4 Wire `RedactionMode::Strict`
- **Gap:** `config.redaction_mode` exists but `Redactor` ignores it. Strict
  should add entropy-based high-risk token detection and an aggressive
  key/value blanket. Standard stays pattern-based.
- **Accept:** Strict redacts a high-entropy 40-char token that Standard misses;
  both leave an allowlist (public model ids, obvious placeholders) intact.

---

## 5. Community features (Phase 5 — the "helper hand" vision)

1. **Zero-config good defaults + `/handoff-now:doctor --fix`** — auto-repair a
   broken status line, missing binary, or stale config.
2. **Editor-agnostic resume** — `RESUME.md` (4.2) plus a `--clipboard` option.
3. **Git-aware handoffs** — record branch, ahead/behind, stash presence, and
   whether the working tree is dirty; warn if resuming on a different branch.
4. **Handoff templates** — let teams define a project `HANDOFF.template.md`
   (checklist, deploy steps) merged into the generated handoff.
5. **"What changed since last handoff"** — `handoff-now diff` between two
   snapshots (files, tests, decisions), so a resumed session sees the delta.
6. **Multi-window safety** — the `latest_session_id` cwd-match logic is good;
   extend to a project-scoped `handoff-now here` that always targets the
   current repo.
7. **Failure cookbook** — `docs/recovery-scenarios.md` with real
   "session died at 97%, here's exactly what to run" walkthroughs.
8. **Opt-in anonymized success counter (local only)** — a local tally of
   "handoffs that saved a session," shown in `status`, never uploaded — fuels
   the roadmap without breaking the no-telemetry promise.

---

## 6. Quality gate (applies to every phase)

- `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test --all`,
  `claude plugin validate .` all green.
- Every fix ships with a regression test that fails before, passes after.
- No new network calls except the existing opt-in Haiku path.
- Update `README.md`, `docs/architecture.md`, `CHANGELOG.md` per phase.

---

## 7. Execution order

| Phase | Scope | Risk | Ships |
|-------|-------|------|-------|
| 1 | Confirmed bug fixes (1.1–1.5) | Low, bounded | First PR |
| 2 | Real-time hardening (2.1–2.4) | Low–med | Second PR |
| 3 | Burn-rate + spike guard | Med | Third PR |
| 4 | verify / export / strict redaction | Med | Fourth PR |
| 5 | Community features | Incremental | Ongoing |

Start now with **Phase 1** — smallest, highest-confidence, unblocks trust in
the artifact before adding intelligence on top.
