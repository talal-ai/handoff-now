# Windows local install

This document supports testing before the GitHub release exists.

1. Extract the release bundle.
2. Run `release\windows-x64\handoff-now.exe setup` in PowerShell.
3. Add the extracted directory as a local Claude Code marketplace:

   ```powershell
   claude plugin marketplace add .\handoff-now
   claude plugin install handoff-now@handoff-now-marketplace --scope user
   ```

4. Restart Claude Code or run `/reload-plugins`, then use `/handoff-now:doctor`.

The setup command preserves the prior status line and places the stable watcher under `~/.claude/handoff-now/`.
