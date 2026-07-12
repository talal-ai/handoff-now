#!/bin/sh
# Detect OS/arch, fetch the matching release binary, verify its SHA-256,
# install it, then run setup. Used both by hook-dispatch.sh (automatic,
# first SessionStart) and by /handoff-now:setup (manual repair).
set -eu
repository="${HANDOFF_NOW_REPOSITORY:-talal-ai/handoff-now}"
version="${HANDOFF_NOW_VERSION:-latest}"

case "$(uname -m)" in
  arm64|aarch64) arch="aarch64" ;;
  x86_64) arch="x86_64" ;;
  *) echo "handoff-now: unsupported CPU architecture $(uname -m)" >&2; exit 2 ;;
esac

case "$(uname -s)" in
  Darwin) name="handoff-now-${arch}-apple-darwin" ;;
  MINGW*|MSYS*|CYGWIN*) name="handoff-now-${arch}-pc-windows-msvc.exe" ;;
  *) echo "handoff-now: unsupported OS $(uname -s); macOS and Windows (via Git Bash) are supported" >&2; exit 2 ;;
esac

if [ "$version" = latest ]; then
  base="https://github.com/${repository}/releases/latest/download"
else
  base="https://github.com/${repository}/releases/download/${version}"
fi

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT INT TERM

curl --fail --location --proto '=https' --tlsv1.2 "$base/$name" -o "$tmp/$name"
curl --fail --location --proto '=https' --tlsv1.2 "$base/SHA256SUMS" -o "$tmp/SHA256SUMS"

expected="$(awk -v n="$name" '$2 == n {print $1}' "$tmp/SHA256SUMS")"
[ -n "$expected" ] || { echo "handoff-now: checksum entry not found for $name" >&2; exit 1; }

if command -v sha256sum >/dev/null 2>&1; then
  actual="$(sha256sum "$tmp/$name" | awk '{print $1}')"
else
  actual="$(shasum -a 256 "$tmp/$name" | awk '{print $1}')"
fi
[ "$expected" = "$actual" ] || { echo "handoff-now: checksum mismatch for $name" >&2; exit 1; }

chmod 700 "$tmp/$name"
"$tmp/$name" setup
engine="$HOME/.claude/handoff-now/bin/handoff-now"
[ -x "$engine" ] || engine="${engine}.exe"
"$engine" doctor
