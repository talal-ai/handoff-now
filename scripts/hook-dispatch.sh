#!/bin/sh
set -eu
event="${1:-Unknown}"
claude_dir="${CLAUDE_CONFIG_DIR:-$HOME/.claude}"
base="$claude_dir/handoff-now/bin/handoff-now"
if [ -x "$base" ]; then exec "$base" hook "$event"; fi
if [ -x "${base}.exe" ]; then exec "${base}.exe" hook "$event"; fi
echo "handoff-now stable watcher is missing; run /handoff-now:setup" >&2
exit 127
