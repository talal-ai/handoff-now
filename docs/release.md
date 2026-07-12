# Release runbook

1. Confirm CI passes on Windows and macOS and `claude plugin validate .` succeeds.
2. Update versions in `Cargo.toml`, `.claude-plugin/plugin.json`, and `.claude-plugin/marketplace.json` together.
3. Update `CHANGELOG.md` and commit the generated `Cargo.lock`.
4. Provision protected GitHub environments and signing secrets:
   - `WINDOWS_CERTIFICATE`, `WINDOWS_CERTIFICATE_PASSWORD`
   - `APPLE_CERTIFICATE`, `APPLE_CERTIFICATE_PASSWORD`, `APPLE_SIGNING_IDENTITY`
   - `APPLE_ID`, `APPLE_TEAM_ID`, `APPLE_APP_PASSWORD` for notarization
5. Create a signed annotated `vX.Y.Z` tag.
6. Inspect the draft release: signatures, checksums, SBOM, provenance, and all four binaries.
7. Test each binary from the draft release on a clean host.
8. Publish the release only after signatures and clean-host tests pass.
9. Verify the marketplace install and `/handoff-now:doctor` from Claude Desktop and CLI.

The release workflow intentionally creates drafts. Missing signing secrets must never silently produce a release described as signed.
