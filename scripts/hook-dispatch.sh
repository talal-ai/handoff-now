#!/usr/bin/env bash
# hooks/hooks.json always invokes this via `bash ...`, so bash features
# (disown) below are safe to use.
set -eu
event="${1:-Unknown}"
claude_dir="${CLAUDE_CONFIG_DIR:-$HOME/.claude}"
base="$claude_dir/handoff-now/bin/handoff-now"
if [ -x "$base" ]; then exec "$base" hook "$event"; fi
if [ -x "${base}.exe" ]; then exec "${base}.exe" hook "$event"; fi

# Native engine not installed yet. Only SessionStart tries to fetch it, and
# it does so in the background so a slow or absent network never blocks the
# hook timeout. Every other event stays a silent no-op until the engine
# lands, instead of erroring on every tool call during the brief install
# window.
[ "$event" = "SessionStart" ] || exit 0

lock="$claude_dir/handoff-now/.bootstrap-lock"
mkdir -p "$claude_dir/handoff-now"
now="$(date +%s)"
if [ -f "$lock" ]; then
  last="$(cat "$lock" 2>/dev/null || echo 0)"
  # Don't retry more than once every 10 minutes (covers offline machines,
  # airplane mode, flaky first attempts) so a broken network doesn't spam
  # download attempts on every session start.
  [ $(( now - last )) -ge 600 ] || exit 0
fi
echo "$now" > "$lock"

echo "handoff-now: installing its background engine for the first time..." >&2
# shellcheck disable=SC1007
script_dir="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
nohup sh "$script_dir/bootstrap.sh" >"$claude_dir/handoff-now/bootstrap.log" 2>&1 &
disown 2>/dev/null || true
exit 0
