# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- ARCHITECTURE.md with component map and design decisions
- CHANGELOG.md following Keep a Changelog format
- RELEASING.md with maintainer release runbook
- CONTRIBUTING.md, SECURITY.md, CODE_OF_CONDUCT.md governance files
- GitHub issue templates (bug report, feature request) and PR template
- CODEOWNERS file for review assignments
- `.editorconfig`, `.dockerignore`, `.cargo/config.toml` configuration
- Dependabot configuration for automated dependency updates
- `rust-toolchain.toml` pinning to Rust 1.94.0
- `clippy.toml` and `rustfmt.toml` for consistent code style
- Release profile with LTO, single codegen unit, panic=abort, strip
- Secret scanning (TruffleHog) in security workflow
- CycloneDX SBOM generation for supply chain transparency
- OSSF Scorecard integration

### Changed

- Upgraded to Rust 2024 edition with MSRV 1.94.0
- Updated all dependencies to latest stable versions
- Pinned all GitHub Actions to commit SHAs for supply-chain security
- Restructured CI with quick-check gate, cargo-hack feature powerset testing
- Added pedantic and nursery clippy lints to CI
- Improved release workflow with proper version validation and multi-platform builds
- Auto-release workflow now uses conventional commits for version determination

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

[Unreleased]: https://github.com/ThreatFlux/ollama_rust_sdk/compare/v0.1.1...HEAD
[0.1.1]: https://github.com/ThreatFlux/ollama_rust_sdk/releases/tag/v0.1.1
