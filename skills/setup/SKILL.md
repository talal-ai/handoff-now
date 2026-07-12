---
name: setup
description: Install or repair the handoff-now quota watcher and preserve the user's existing Claude status line.
allowed-tools: Bash
---

Run `handoff-now setup`, then `handoff-now doctor`. Show the user the doctor result. If the command reports that the native binary is missing, direct the user to the platform bootstrap instructions in the plugin README. Do not modify Claude settings by hand; the setup command performs a backed-up, idempotent update.
