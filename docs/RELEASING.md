# Releasing

## Automated Release (default)

Releases are driven by [Conventional Commits](https://www.conventionalcommits.org/). When CI and security checks pass on `main`, the `auto-release.yml` workflow:

1. Analyzes commits since the last tag.
2. Determines the version bump (patch / minor / major) from commit prefixes.
3. Commits the version bump and pushes a new Git tag (`v*`) as the `threatflux-automation`
   GitHub App.
4. The tag push triggers `release.yml`, which builds, packages, publishes, and creates the GitHub
   Release, and `docker.yml`, which publishes the container image. Each runs once per tag: the
   reusable workflow (`ThreatFlux/github_actions` `reusable-auto-release.yml`) does not dispatch
   them again when the App pushed the tag. It only dispatches them if the release falls back to
   `GITHUB_TOKEN`, whose tag push starts no workflows. The release commit's push to `main` runs
   CI as usual, but `docker.yml` skips its branch build: the tag run builds that commit.

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
| SBOM | `ollama-cli-vX.Y.Z.cdx.json` (CycloneDX 1.5, all features) with a SHA256 checksum |
| Publish | crates.io through trusted publishing (skipped when the version is already published) |
| GitHub Release | Checksums, SBOM and packaged assets attached |

The release notes are the `## [X.Y.Z]` section of `docs/CHANGELOG.md` followed by GitHub's
generated list of merged pull requests, so move the `[Unreleased]` entries under the new version
before releasing. auto-release creates the Release first with notes that list only breaking,
feat and fix commits; `release.yml` replaces that generated body (a bare `## Release vX.Y.Z`
heading when there are none) and leaves any other notes alone.

### Required Permissions

| Credential | Holder | Purpose |
|------------|--------|---------|
| `threatflux-automation` GitHub App (org variable `TF_AUTOMATION_APP_ID`, org secret `TF_AUTOMATION_APP_PRIVATE_KEY`) | `auto-release.yml` and `dependencies.yml` | Release commit and tag pushed so they trigger `release.yml`/`docker.yml`; weekly dependency PRs whose CI starts without approval |
| `GITHUB_TOKEN` | Automatic | Release tag (manual dispatch), GitHub Release and assets |
| GitHub OIDC (`id-token: write`) | `publish` job in the `crates-io` environment | Short-lived crates.io token |

crates.io publishing uses [trusted publishing](https://crates.io/docs/trusted-publishing): the
crate's trusted publisher is `ThreatFlux/ollama_rust_sdk`, workflow `release.yml`, environment
`crates-io`. `rust-lang/crates-io-auth-action` exchanges the job's OIDC identity for a short-lived
token and revokes it when the job ends, so no crates.io API token is stored in GitHub. Renaming
`release.yml` or the environment requires updating the trusted publisher on crates.io first. A
failed publish fails the release run.

### Dry Run

Both release workflows can be rehearsed from `main` without tagging, releasing or publishing:

```bash
# Report the version auto-release would cut; no commit, tag, release or dispatch.
# Once the CI and Security gate passes it still mints the GitHub App token, so it
# also checks the App configuration.
gh workflow run auto-release.yml -f version_bump=auto -f dry_run=true

# Build and package every target and run `cargo publish --dry-run --locked`
# without creating the tag or GitHub Release, uploading assets, or publishing.
gh workflow run release.yml -f version=1.2.3 -f dry_run=true
```

A release dry run doesn't use the `crates-io` environment or request a crates.io token.
It doesn't need an existing tag. If `version` is ahead of `Cargo.toml`,
as it is before auto-release commits the bump, it warns and builds the manifest version.
Container images are built by `docker.yml`, which a dry run doesn't dispatch.

### Rollback

If a release is defective:

1. Delete the GitHub Release (draft state or full delete).
2. Delete the Git tag: `git push --delete origin v1.2.3`
3. Yank from crates.io if published: `cargo yank --version 1.2.3`
4. Fix, then re-release with the next patch version.
