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
make ci-local
```

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
- [ ] Commit messages follow conventions

## Security Issues

Do not open public issues for security vulnerabilities. See [SECURITY.md](SECURITY.md).
