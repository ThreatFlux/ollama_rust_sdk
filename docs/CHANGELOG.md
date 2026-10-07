# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed

- `release.yml` writes the Windows archive's `.sha256` file with an LF line ending, like the Unix
  archives' files. The `ollama-cli-windows-amd64.zip.sha256` assets of 0.1.5 and 0.1.6 end in CRLF,
  so `shasum -a 256 -c` and macOS `sha256sum -c` report the archive as missing; check one with
  `tr -d '\r' < ollama-cli-windows-amd64.zip.sha256 | shasum -a 256 -c` (the hash itself is correct).

## [0.1.6] - 2026-10-06

### Documentation

- Audited current official Ollama APIs and recorded a prioritized feature and correctness plan
- Reworked the README around verified SDK capabilities, a compile-checked quickstart, and explicit
  reliability and support guidance
- Added source-derived API coverage and configuration references
- Moved the supplemental raw curl examples under `docs/` and added a documentation index
- Added a documentation contract that checks metadata, feature flags, quickstart drift, unsupported
  claims, and local links in CI

### Changed

- Pinned development, fixed CI jobs, release builds and Docker to stable Rust 1.99.0 while retaining
  and testing the consumer MSRV of 1.97.1
- Upgraded twelve direct/development crates and refreshed the stable, non-yanked dependency graph,
  including rustls 0.23.45 for RUSTSEC-2026-0285
- Refreshed immutable GitHub Action pins and pinned development/CI command-line tools
- Added worktree-aware repository hook installation; aligned local feature, lint and MSRV checks
  with CI, and made security and dependency gate failures propagate
- Updated the Docker certificate pin to match Bookworm and prevent the existing downgrade failure
- Moved the container image off Debian 12: the builder is `rust:1.99.0-trixie` and the runtime
  is `gcr.io/distroless/cc-debian13:nonroot` (both digest-pinned), running as the distroless
  `nonroot` user (UID 65532) with `tini` as PID 1; CI now smoke-tests `--version`, `--help`,
  the non-root user and the embedded SBOM
- Publish to crates.io through trusted publishing (GitHub OIDC in the `crates-io` environment)
  instead of a stored API token; a failed publish fails the release, and a re-run skips a
  version that is already on crates.io
- Cut releases and open weekly dependency PRs as the `threatflux-automation` GitHub App, so the
  release tag starts `release.yml` and `docker.yml` once each and dependency PR CI runs without
  approval; the release automation is pinned to `ThreatFlux/github_actions` v0.7.7, and
  `docker.yml` no longer builds a `chore: release vX.Y.Z` commit on `main` as well as on its tag
- Replaced placeholder Cargo package metadata with the canonical ThreatFlux repository and
  documentation URLs
- Each GitHub Release now carries a CycloneDX 1.5 SBOM of the crate covering every target platform
  (`ollama-cli-vX.Y.Z.cdx.json`) and its SHA-256 checksum next to the CLI archives
- Refreshed `Cargo.lock`: h2 0.4.20, hyper 1.12.0, jiff 0.2.38 and want 0.3.2

### Fixed

- GitHub Release notes were only a `## Release vX.Y.Z` heading: `release.yml` looked for a root
  `CHANGELOG.md` that does not exist and its section extraction stopped at the heading line. The
  notes are now this file's version section followed by GitHub's generated pull request list,
  and they replace the heading-only body auto-release creates without overwriting edited notes

## [0.1.5] - 2026-08-01

### Added

- ARCHITECTURE.md with component map and design decisions
- CHANGELOG.md following Keep a Changelog format
- RELEASING.md with maintainer release runbook
- CONTRIBUTING.md, SECURITY.md, CODE_OF_CONDUCT.md governance files
- GitHub issue templates (bug report, feature request) and PR template
- CODEOWNERS file for review assignments
- `.editorconfig`, `.dockerignore`, `.cargo/config.toml` configuration
- Dependabot configuration for automated dependency updates
- `rust-toolchain.toml` pinning to Rust 1.97.1
- `clippy.toml` and `rustfmt.toml` for consistent code style
- Release profile with LTO, single codegen unit, panic=abort, strip
- Secret scanning (TruffleHog) in security workflow
- CycloneDX SBOM generation for supply chain transparency
- OSSF Scorecard integration

### Changed

- Upgraded to Rust 2024 edition with MSRV 1.97.1
- Updated all dependencies to latest stable versions
- Pinned all GitHub Actions to commit SHAs for supply-chain security
- Restructured CI with quick-check gate, cargo-hack feature powerset testing
- Added pedantic and nursery clippy lints to CI
- Improved release workflow with proper version validation and multi-platform builds
- Auto-release workflow now uses conventional commits for version determination

### Maintenance

- Refreshed Rust dependencies and Cargo.lock to the latest compatible stable releases
- Updated Docker and CI toolchains to Rust 1.97.1
- Updated pinned GitHub Actions and security tooling

## [0.1.1] - 2025-03-24

### Added

- Initial public release
- Complete Ollama API coverage (generate, chat, embed, models, blobs)
- Streaming support for chat and generation
- Builder pattern for request construction
- CLI tool for command-line usage
- Comprehensive test suite (130+ tests)
- Multi-platform CI (Linux, macOS, Windows)
- Code coverage with cargo-llvm-cov
- Security auditing with cargo-audit and cargo-deny

[Unreleased]: https://github.com/ThreatFlux/ollama_rust_sdk/compare/v0.1.6...HEAD
[0.1.6]: https://github.com/ThreatFlux/ollama_rust_sdk/compare/v0.1.5...v0.1.6
[0.1.5]: https://github.com/ThreatFlux/ollama_rust_sdk/compare/v0.1.4...v0.1.5
[0.1.1]: https://github.com/ThreatFlux/ollama_rust_sdk/releases/tag/v0.1.1
