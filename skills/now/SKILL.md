---
name: now
description: Create an immediate recovery package AND a genuine model-written summary for the current or latest session.
allowed-tools: Bash, Task
---

When the user asks for an immediate handoff, produce BOTH the factual package and
a genuine narrative summary — do not stop after the deterministic step.

1. Run `handoff-now now`. Capture the absolute artifact directory it prints (its
   final path component is the session id).

2. Launch the `handoff-now:handoff-writer` subagent (Task tool) to write the
   semantic summary. Pass it the artifact directory and instruct it to read the
   full package — including `CHAT-HISTORY.redacted.md` (the complete both-sides
   conversation), `EVENTS.jsonl`, `HANDOFF.md`, `FILES.md`, `TESTS.md`,
   `git-status.txt`, and `SESSION.json` — and to write `SUMMARY.candidate.md`
   with the required headings and the `Journal Sequence: N` value from
   `SESSION.json`. The agent must only write `SUMMARY.candidate.md`.

3. After the agent finishes, run `handoff-now promote <SESSION_ID>` to validate
   and promote the candidate into the authoritative `SUMMARY.md`. If promotion
   reports an error, relay it and leave the deterministic package in place.

4. Report to the user: the artifact directory, whether the semantic summary was
   promoted (or the validation error), and the recovered user goal from
   `HANDOFF.md`. Do not expose raw transcript content.

Do not continue unrelated implementation work after an explicit handoff request.
