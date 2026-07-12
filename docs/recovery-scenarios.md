# Recovery Scenarios

Concrete "it died — now what" walkthroughs. Every path below is deterministic
and needs zero remaining model allowance.

## 1. Session ended gracefully below 85%

You closed the window or the context compacted. `SessionEnd` / `PreCompact`
now write a fresh `HANDOFF.md` regardless of usage band.

```text
handoff-now resume
```

Prints the verified handoff directory. Open `HANDOFF.md`, run the "Exact First
Action", continue.

## 2. Usage climbed through the bands (85 → 90 → 95)

Normal path. At 85% a checkpoint is written; at 90% source writes freeze and the
`handoff-writer` agent enriches `SUMMARY.md`; at 95% work stops with a verified
handoff. Resume as above. If the semantic summary is missing, the deterministic
files (`EVENTS.jsonl`, `FILES.md`, `working-changes.patch`) are authoritative.

## 3. Usage teleported to ~100% in one response

The status line is one response behind, so the freeze cannot stop the response
that caused the jump. Two safety nets catch it:

- **Spike guard** — the next render sees the jump and forces a snapshot.
- **Burn-rate** — a steep slope shows `~Nm to wall` earlier so you can stop
  sooner.

After the jump, `resume` names the snapshot taken at the post-jump render.

## 4. Hard kill before any snapshot ran

Process died, no `HANDOFF.md` exists — but `EVENTS.jsonl` is append-only and
fsync'd per event, so it survived.

```text
handoff-now resume
```

`resume` detects the missing handoff and rebuilds one deterministically from the
journal, then prints it.

## 5. You want to move the task to another machine or agent

```text
handoff-now export
```

Writes a single self-contained `RESUME.md` (goal, summary, git status, journal
tail, full handoff) that pastes into any fresh session with no dependency on the
`.handoff-now` directory.

## 6. You want to trust the artifact before resuming

```text
handoff-now verify
```

Recomputes every file's SHA-256 against `integrity.json` and checks the journal
sequence. Non-zero exit and a `mismatched` list on any tamper or drift.

## 7. Installation looks broken

```text
handoff-now doctor --fix
```

Re-runs setup (restores the chained status line, rewrites settings) then reports
health.

---

Never attach raw transcripts or credentials when reporting a failure. Redaction
is best effort — review artifacts before sharing.
