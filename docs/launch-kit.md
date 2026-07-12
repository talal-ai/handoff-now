# Handoff Now launch kit

Use one clear promise everywhere: **Never lose a long Claude Code session to the 5-hour usage limit.**

## One-line description

Handoff Now is an open-source Claude Code safety plugin that watches the official 5-hour usage signal, freezes risky work before exhaustion, and creates a verified local resume package.

## Short launch post

Claude Code once hit its usage limit in the middle of a long implementation and left no clean way to continue. I built Handoff Now: an open-source, local-first plugin that watches Claude Code's official 5-hour usage signal, checkpoints at 85%, freezes new source mutations at 90%, and produces a verified handoff package that a fresh session can resume. Recovery does not depend on another model call.

GitHub: https://github.com/talal-ai/handoff-now

## Show HN title

Show HN: Handoff Now – recover Claude Code tasks before the 5-hour limit

## Reddit title

I built a Claude Code safety net that creates a verified handoff before the 5-hour limit

## X / Bluesky post

Never lose a long Claude Code session to the 5-hour limit.

Handoff Now watches the official usage signal, checkpoints at 85%, freezes risky edits at 90%, and leaves a verified local resume package—even without another model call.

Open source: https://github.com/talal-ai/handoff-now

## Demo script

Record a 60–90 second terminal video:

1. Show `/handoff-now:status` below the preparation threshold.
2. Replay a safe test event at 86%; show the deterministic checkpoint.
3. Replay 91%; attempt a source edit and show it being denied.
4. Open `HANDOFF.md`, `working-changes.patch`, and `integrity.json`.
5. Start a fresh session and run `/handoff-now:resume`.
6. End on the source install command. Switch the recording to the one-command bootstrap only after signed binaries are public.

Never simulate a real testimonial or imply that a mocked percentage consumed subscription quota. Label threshold replays as a demo.

## Launch channels

- Anthropic plugin directory submission
- GitHub release and repository social preview
- Hacker News Show HN
- r/ClaudeCode, r/ClaudeAI, r/codex, and relevant Discord communities
- Product Hunt after at least five independent users successfully recover a task

Answer questions in the first 24 hours. Ask users for reproducible failures and successful recovery stories; do not ask for stars before delivering value.

Do not begin the broad launch while the release is still a private draft. The landing page and repository may go live for early testers, but the main announcement waits for a clean-host install of signed public binaries.
