# Security policy

## Reporting

Do not open a public issue for a credential disclosure, path escape, command-injection, signature, or redaction bypass. Use GitHub's private security advisory flow for this repository.

## Trust boundaries

`handoff-now` treats transcripts, repository files, Git output, hook payloads, and model output as untrusted data. Deterministic state and journals are authoritative; semantic summaries are advisory and validated before promotion.

The plugin runs with the user's privileges because Claude Code hooks do. It canonicalizes paths, confines emergency writes to the session artifact directory, uses a Git command allowlist, rejects mixed shell commands, writes atomically, and disables telemetry by default.

## Secrets

Default external semantic payloads are minimized and redacted. Raw transcript retention is opt-in. Redaction is defense-in-depth, not a mathematical guarantee. Projects with exceptionally sensitive material should select the deterministic provider and disable Git diff capture.

API credentials must be supplied through `ANTHROPIC_API_KEY` or a future OS credential-store integration. They must never be placed in plugin configuration, project files, transcripts, or handoff artifacts.

## Signing

Release workflows support Authenticode and Apple Developer ID signing/notarization when repository secrets are provisioned. Unsigned development builds must not be represented as trusted production releases.
