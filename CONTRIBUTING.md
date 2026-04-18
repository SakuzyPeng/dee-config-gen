# Contributing

Thanks for spending time on `dee-config-gen`.

Start here:
- [README.md](README.md) / [README.en.md](README.en.md) for the user-facing overview
- [docs/open-source-guide.zh.md](docs/open-source-guide.zh.md) / [docs/open-source-guide.en.md](docs/open-source-guide.en.md) for distribution boundaries and reproducibility
- [docs/developer-guide.zh.md](docs/developer-guide.zh.md) / [docs/developer-guide.en.md](docs/developer-guide.en.md) for architecture and maintenance rules
- [SECURITY.md](SECURITY.md) for vulnerability reporting
- [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) for collaboration expectations

## Communication

- Maintainer discussions default to Chinese.
- English issues and pull requests are welcome.
- Keep bug reports concrete and include reproduction steps or the exact input spec when possible.

## Local Setup

Build:

```bash
cargo build
```

Default test suite:

```bash
cargo test --workspace --quiet
```

Recommended pre-PR checks:

```bash
scripts/precommit_checks.sh
```

Quick manual smoke:

```bash
cargo run -- validate -i examples/atmos_ec3_single.streaming.yaml
cargo run -- generate -i examples/pcm_ddp_single.dd.yaml -o job.xml
```

## Test Layers and Runtime Boundaries

- Default `cargo test --workspace --quiet` is the baseline for public contributors. It should pass without a local Dolby runtime.
- Real DEE runtime suites stay `#[ignore]` and are run manually with `-- --ignored --nocapture`.
- Some ignored suites require local `dee`, `ffmpeg`, `DEE_WORKSPACE_ROOT`, AC-4 runtime packages, native mp4 muxing support, or local sample assets such as `testfiles/testADM.wav`.
- Do not commit proprietary runtimes, private audio fixtures, local logs, or generated temp outputs.

See the open-source guide and coverage docs for the exact runtime split:
- [docs/open-source-guide.en.md](docs/open-source-guide.en.md)
- [docs/coverage-and-experiments.en.md](docs/coverage-and-experiments.en.md)

## Change Expectations

- Keep changes focused. Avoid mixing refactors with behavior changes unless the refactor is required for the feature or fix.
- Follow Conventional Commit style when possible, for example `feat: ...`, `fix: ...`, `docs: ...`, `test: ...`.
- If you change template behavior, also update the matching examples, parameter matrix, coverage matrix, and evidence fixtures.
- If you change the stable FFI surfaces, preserve the documented C ABI v1 / UniFFI v1 contracts or bump the documented contract version intentionally.
- If a change is breaking for users, call it out in [CHANGELOG.md](CHANGELOG.md) and the release notes.

## Pull Requests

Before opening a PR, try to include:
- a short problem/solution summary
- verification commands you ran
- example input/output or fixture notes if behavior changed
- docs updates when public behavior, compatibility, or contribution workflow changed

Useful checklist:
- default tests still pass
- ignored runtime tests were left ignored unless there is a strong reason to change the policy
- no proprietary assets, local paths, temp logs, or local helper files were added to tracked content
- new public behavior is reflected in the docs

## Security

Please do not report security issues in a public issue or PR.

Use the process in [SECURITY.md](SECURITY.md). If the repository does not yet expose a private reporting path, open a minimal public issue titled `Security contact requested` without exploit details.
