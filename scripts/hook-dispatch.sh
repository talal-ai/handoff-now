#!/usr/bin/env bash
# hooks/hooks.json always invokes this via `bash ...`, so bash features
# (disown) below are safe to use.
set -eu
event="${1:-Unknown}"
claude_dir="${CLAUDE_CONFIG_DIR:-$HOME/.claude}"
base="$claude_dir/handoff-now/bin/handoff-now"

# Resolve the installed engine (with or without a .exe suffix).
engine=""
if [ -x "$base" ]; then
  engine="$base"
elif [ -x "${base}.exe" ]; then
  engine="${base}.exe"
fi

# --- Automatic engine updates -------------------------------------------------
# The native engine lives outside the plugin directory, so a plugin update alone
# never refreshes it. Existing users would stay pinned to whatever engine they
# first installed. On SessionStart we compare the installed engine version with
# the plugin's declared version and, on mismatch (or first run), fetch the
# matching release in the background. No manual command, no cache clear: the new
# engine is simply live on the next session. The fetch is backgrounded and
# rate-limited so a slow or offline network never blocks the hook.
if [ "$event" = "SessionStart" ]; then
  expected=""
  if [ -n "${CLAUDE_PLUGIN_ROOT:-}" ] && [ -f "${CLAUDE_PLUGIN_ROOT}/.claude-plugin/plugin.json" ]; then
    expected="$(sed -n 's/.*"version"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' \
      "${CLAUDE_PLUGIN_ROOT}/.claude-plugin/plugin.json" 2>/dev/null | head -n1 || true)"
  fi
  installed=""
  if [ -n "$engine" ]; then
    installed="$("$engine" version 2>/dev/null | awk '{print $2}' || true)"
  fi

  needs_update=0
  if [ -z "$engine" ]; then
    needs_update=1
  elif [ -n "$expected" ] && [ "$installed" != "$expected" ]; then
    needs_update=1
  fi

  if [ "$needs_update" = 1 ]; then
    lock="$claude_dir/handoff-now/.bootstrap-lock"
    mkdir -p "$claude_dir/handoff-now"
    now="$(date +%s)"
    do_fetch=1
    if [ -f "$lock" ]; then
      last="$(cat "$lock" 2>/dev/null || echo 0)"
      # Don't retry more than once every 10 minutes (covers offline machines,
      # airplane mode, flaky first attempts) so a broken network doesn't spam
      # download attempts on every session start.
      [ $(( now - last )) -ge 600 ] || do_fetch=0
    fi
    if [ "$do_fetch" = 1 ]; then
      echo "$now" > "$lock"
      if [ -n "$engine" ]; then
        echo "handoff-now: updating engine ${installed:-unknown} -> ${expected:-latest} in the background..." >&2
      else
        echo "handoff-now: installing its background engine for the first time..." >&2
      fi
      # shellcheck disable=SC1007
      script_dir="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
      # Pin the download to the plugin's version (tagged vX.Y.Z) so the engine
      # and plugin stay in lockstep. Empty falls back to `latest` in bootstrap.
      HANDOFF_NOW_VERSION="${expected:+v$expected}" \
        nohup sh "$script_dir/bootstrap.sh" >"$claude_dir/handoff-now/bootstrap.log" 2>&1 &
      disown 2>/dev/null || true
    fi
  fi
fi
# -----------------------------------------------------------------------------

# Run the currently-installed engine for this event. Even if an update was just
# triggered above, this session keeps using the present binary (the new one goes
# live next session). No-op until the engine first lands.
if [ -n "$engine" ]; then exec "$engine" hook "$event"; fi
exit 0
