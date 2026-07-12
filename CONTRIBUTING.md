# Contributing

1. Open an issue describing the behavior or security invariant being changed.
2. Add tests for state transitions, hook output, redaction, and crash consistency as applicable.
3. Run formatting, Clippy with warnings denied, all tests, and Claude plugin validation.
4. Do not commit credentials, raw user transcripts, generated handoffs, signing material, or compiled binaries.
5. Keep deterministic recovery independent from optional model providers.

Security-sensitive changes require review from a maintainer familiar with the threat model.
