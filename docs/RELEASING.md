# Releasing

## Automated Release (default)

Releases are driven by [Conventional Commits](https://www.conventionalcommits.org/). When CI and security checks pass on `main`, the `auto-release.yml` workflow:

1. Analyzes commits since the last tag.
2. Determines the version bump (patch / minor / major) from commit prefixes.
3. Creates a new Git tag (`v*`).
4. The tag triggers `release.yml`, which builds, packages, publishes, and creates the GitHub Release.

**No manual steps are required for routine releases.**

## Manual Release

Use this when the automated flow is insufficient (e.g., pre-release versions, hotfixes).

### Pre-flight

1. Ensure `main` is green:
   ```bash
   make ci-local
   ```
2. Update `docs/CHANGELOG.md` - move items from `[Unreleased]` to a new version header.
3. Bump the version in `Cargo.toml`.
4. Commit:
   ```bash
   git add Cargo.toml docs/CHANGELOG.md
   git commit -m "chore: release v1.2.3"
   ```
5. Tag:
   ```bash
   git tag v1.2.3
   git push origin main --tags
   ```

### What Happens Next

The `v*` tag triggers `release.yml`:

| Step | Artifact |
|------|----------|
| Cross-compile | Linux x86_64/musl, macOS aarch64/x86_64, Windows x86_64 |
| Package | `.tar.gz` (Unix) and `.zip` (Windows) with SHA256 checksums |
| Publish | crates.io (if `CRATES_IO_TOKEN` or `CARGO_REGISTRY_TOKEN` secret is set) |
| GitHub Release | Checksums + packaged assets attached |

### Required Permissions

| Secret | Holder | Purpose |
|--------|--------|---------|
| `GITHUB_TOKEN` | Automatic | Release assets |
| `CRATES_IO_TOKEN` | Repo admin | crates.io publish |

### Rollback

If a release is defective:

1. Delete the GitHub Release (draft state or full delete).
2. Delete the Git tag: `git push --delete origin v1.2.3`
3. Yank from crates.io if published: `cargo yank --version 1.2.3`
4. Fix, then re-release with the next patch version.
