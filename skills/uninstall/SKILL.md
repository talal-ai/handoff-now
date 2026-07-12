---
name: uninstall
description: Restore the status line that existed before handoff-now setup while retaining recovery data.
allowed-tools: Bash
---

Run `handoff-now uninstall`. If it refuses because the status line was changed later, do not overwrite settings manually; explain the conflict. Recovery data remains under `~/.claude/handoff-now/` and project `.handoff-now/` directories.
