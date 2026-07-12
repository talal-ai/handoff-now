# Architecture

## Data flow

```text
Claude API response
       |
       v
official statusLine JSON ----> usage state (locked, atomic)
       |                              |
       |                              +--> threshold snapshot worker
       v
existing status line output

Claude lifecycle hook --------> append-only EVENTS.jsonl
       |                              |
       |                              +--> deterministic artifacts
       v
protected-mode decision  -----> optional semantic candidate
                                      |
                                      v
                                validated SUMMARY.md
```

The status line is the only quota source. It supplies the official five-hour percentage and reset timestamp. Hooks intentionally do not estimate usage from tokens or read private authentication state.

## Correctness invariants

1. Deterministic recovery runs before semantic enrichment in emergency mode.
2. A journal record is append-only and receives a monotonically increasing sequence under a session lock.
3. Model output never changes the event journal or session state.
4. Protected-mode writes resolve beneath the canonical session artifact directory.
5. A hard-stop response always names an existing deterministic handoff.
6. Duplicate hooks are safe; semantic finalization has a durable single-attempt marker.
7. Resetting quota preserves old artifacts and requires explicit user resumption.

## Concurrency

Each session has a separate advisory lock and state document under `~/.claude/handoff-now/`. Desktop worktrees remain distinct through their session IDs and canonical working directories. Atomic replacement protects state and promoted Markdown from partial writes. Event journal appends are flushed before the hook returns.

## Provider boundary

The deterministic provider owns factual artifacts. The API provider receives only capped, redacted journal/history excerpts. The native `handoff-writer` agent is advisory and shares subscription allowance. Semantic candidates must satisfy structural, sequence, secret, and size validation before promotion.
