#!/bin/sh
set -eu
repository="${HANDOFF_NOW_REPOSITORY:-YOUR-GITHUB-OWNER/handoff-now}"
version="${HANDOFF_NOW_VERSION:-latest}"
[ "$repository" != "YOUR-GITHUB-OWNER/handoff-now" ] || { echo "Set HANDOFF_NOW_REPOSITORY=OWNER/handoff-now after publishing." >&2; exit 2; }
case "$(uname -m)" in
  arm64|aarch64) target="aarch64-apple-darwin" ;;
  x86_64) target="x86_64-apple-darwin" ;;
  *) echo "Unsupported macOS architecture" >&2; exit 2 ;;
esac
[ "$(uname -s)" = "Darwin" ] || { echo "v1 supports macOS and Windows only" >&2; exit 2; }
name="handoff-now-${target}"
if [ "$version" = latest ]; then base="https://github.com/${repository}/releases/latest/download"; else base="https://github.com/${repository}/releases/download/${version}"; fi
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT INT TERM
curl --fail --location --proto '=https' --tlsv1.2 "$base/$name" -o "$tmp/$name"
curl --fail --location --proto '=https' --tlsv1.2 "$base/SHA256SUMS" -o "$tmp/SHA256SUMS"
expected="$(awk -v n="$name" '$2 == n {print $1}' "$tmp/SHA256SUMS")"
[ -n "$expected" ] || { echo "Checksum entry not found" >&2; exit 1; }
actual="$(shasum -a 256 "$tmp/$name" | awk '{print $1}')"
[ "$expected" = "$actual" ] || { echo "Checksum mismatch" >&2; exit 1; }
chmod 700 "$tmp/$name"
"$tmp/$name" setup
"$HOME/.claude/handoff-now/bin/handoff-now" doctor
