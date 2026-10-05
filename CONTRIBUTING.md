# Contributing to Ollama Rust SDK

Thank you for your interest in contributing to the Ollama Rust SDK.

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/<you>/ollama_rust_sdk.git`
3. Create a branch: `git checkout -b feat/your-change`
4. Make your changes
5. Run checks: `make ci-local`
6. Open a Pull Request

## Development Setup

```bash
make dev-setup
make hooks-install
make ci-local
```

Documentation changes have a machine-checkable contract. When editing the README, Cargo features,
package metadata, quickstart, or local documentation links, use Python 3.11 or newer and run:

```bash
python3 scripts/check_docs.py
cargo check --locked --example quickstart
```

The README quickstart must match `examples/quickstart.rs`; edit both in the same change. The contract
also keeps Git installation guidance release-safe and validates static JSON payloads in the
supplemental curl reference.

The development toolchain is Rust 1.99.0, while consumers remain supported on Rust 1.97.1.
`make ci-local` checks the actual MSRV, every feature combination, the CI Clippy policy, and
security and dependency policy; failures stop the gate. Repository hooks also check formatting
and the documentation contract, and installation preserves foreign or symlink hooks.

## Commit Guidelines

We use [Conventional Commits](https://www.conventionalcommits.org/):

- `feat`: new feature
- `fix`: bug fix
- `docs`: documentation only
- `refactor`: code refactoring
- `test`: adding or updating tests
- `chore`: maintenance

## Pull Request Process

- Use a conventional-commit title
- Explain what changed and why
- Add tests or validation where applicable
- Ensure all CI checks pass

### PR Checklist

- [ ] Code follows project style (`make fmt`)
- [ ] All tests pass (`make test`)
- [ ] Linting passes (`make lint`)
- [ ] Documentation updated if needed
- [ ] Documentation contract passes (`make docs-contract`)
- [ ] Commit messages follow conventions

## Security Issues

Do not open public issues for security vulnerabilities. See [SECURITY.md](SECURITY.md).
