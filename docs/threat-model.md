# Threat model

## Protected assets

- Source repository integrity
- Claude settings and the user's prior status line
- API credentials and secrets appearing in transcripts or diffs
- Accuracy and availability of the latest valid handoff
- Integrity of release binaries and marketplace metadata

## Adversaries and failure sources

- Prompt injection embedded in source files, terminal output, or transcript text
- Malicious tool inputs attempting shell or path escape
- Symlinks/junctions escaping the artifact directory
- Abrupt process, host, or Desktop termination
- Duplicate/concurrent hooks and stale status-line updates
- Compromised or malformed semantic-model output
- Supply-chain substitution of release binaries

## Controls

- Data/instruction separation in all semantic prompts
- Canonical path confinement and safe Git allowlist
- Atomic replacement, session locks, journal sequences, and integrity hashes
- Bounded payloads and one-shot emergency semantic attempts
- Local redaction and opt-in raw transcript retention
- Signed release support, SHA-256 checksums, SBOM, and GitHub provenance
- No telemetry, OAuth scraping, or undocumented endpoints

## Residual risk

Pattern redaction cannot identify every secret. Lifecycle hooks cannot preempt an unfinished API response or safely undo a running command. A compromised user account or repository signing key remains outside the plugin's protection boundary.
