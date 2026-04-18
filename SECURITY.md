# Security Policy

## Supported Versions

`dee-config-gen` does not currently maintain long-term support branches.

| Version | Status |
| --- | --- |
| Latest `main` and the newest `0.1.x` release/tag | Supported on a best-effort basis |
| Older commits, forks, and unpublished snapshots | Not supported |

The explicitly stable cross-language contracts are:
- C ABI v1
- UniFFI v1

The CLI, Rust library API, and template semantics are still in the `0.x` phase and may evolve between releases. Breaking changes must be called out in [CHANGELOG.md](CHANGELOG.md) and release notes.

## Reporting a Vulnerability

Please do not open a public issue with exploit details.

Preferred path:
1. Use GitHub private vulnerability reporting or a GitHub Security Advisory for this repository.
2. Include the affected version or commit, impact, reproduction steps, and any required runtime assumptions.

Fallback path if private reporting is not available yet:
1. Open a minimal public issue titled `Security contact requested`.
2. Do not include exploit details, private assets, secrets, or proof-of-concept payloads in that public issue.
3. Ask the maintainers to move the report to a private channel.

## Response Expectations

- Maintainers aim to acknowledge new reports within 7 business days.
- Please give maintainers reasonable time to validate and prepare a fix before public disclosure.
- If the issue depends on proprietary Dolby tooling or local fixtures, say so clearly. Do not attach assets you do not have permission to redistribute.
