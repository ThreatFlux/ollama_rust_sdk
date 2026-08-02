#!/usr/bin/env python3
"""Validate the repository's dependency-free documentation contract."""

from __future__ import annotations

import argparse
import re
import sys
import tomllib
from pathlib import Path
from urllib.parse import unquote, urlsplit


ROOT = Path(__file__).resolve().parents[1]
QUICKSTART_BEGIN = "<!-- BEGIN QUICKSTART -->"
QUICKSTART_END = "<!-- END QUICKSTART -->"
LINK_RE = re.compile(r"!?\[[^\]\n]*\]\(\s*(<[^>\n]+>|[^\s)]+)")
FEATURE_ROW_RE = re.compile(r"^\|\s*`([^`]+)`\s*\|", re.MULTILINE)
BANNED_CLAIMS = (
    "all Ollama API endpoints",
    "OpenAI-compatible endpoints support",
    "comprehensive model management",
)
REQUIRED_FILES = (
    "README.md",
    "CONTRIBUTING.md",
    "SECURITY.md",
    ".markdownlint.jsonc",
    "docs/api-coverage.md",
    "docs/configuration.md",
    "docs/README.md",
    "docs/ollama-api-curl-reference.md",
    "examples/quickstart.rs",
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--print-msrv",
        action="store_true",
        help="print package.rust-version and skip validation",
    )
    return parser.parse_args()


def load_manifest() -> dict:
    with (ROOT / "Cargo.toml").open("rb") as cargo_file:
        return tomllib.load(cargo_file)


def cargo_features(manifest: dict) -> set[str]:
    features = manifest.get("features", {})
    return set(features) if isinstance(features, dict) else set()


def markdown_files() -> list[Path]:
    return [
        path
        for path in ROOT.rglob("*.md")
        if ".git" not in path.parts and "target" not in path.parts
    ]


def local_link_target(document: Path, raw_target: str) -> Path | None:
    target = raw_target.removeprefix("<").removesuffix(">")
    parsed = urlsplit(target)
    if parsed.scheme or parsed.netloc or not parsed.path:
        return None
    return (document.parent / unquote(parsed.path)).resolve()


def check_local_links(problems: list[str]) -> None:
    for document in markdown_files():
        text = document.read_text(encoding="utf-8")
        for match in LINK_RE.finditer(text):
            target = local_link_target(document, match.group(1))
            if target is None:
                continue
            if not target.is_relative_to(ROOT) or not target.exists():
                line = text.count("\n", 0, match.start()) + 1
                display = document.relative_to(ROOT)
                problems.append(f"{display}:{line}: missing local link target {match.group(1)}")


def check_quickstart(readme: str, problems: list[str]) -> None:
    if readme.count(QUICKSTART_BEGIN) != 1 or readme.count(QUICKSTART_END) != 1:
        problems.append("README.md: quickstart markers must each appear exactly once")
        return

    region = readme.split(QUICKSTART_BEGIN, 1)[1].split(QUICKSTART_END, 1)[0].strip()
    match = re.fullmatch(r"```rust\n(.*)\n```", region, re.DOTALL)
    if match is None:
        problems.append("README.md: quickstart markers must contain one Rust code block")
        return

    example = (ROOT / "examples/quickstart.rs").read_text(encoding="utf-8").rstrip("\n")
    if match.group(1) != example:
        problems.append("README.md: quickstart must match examples/quickstart.rs exactly")


def check_features(readme: str, manifest: dict, problems: list[str]) -> None:
    heading = "## Cargo features"
    if readme.count(heading) != 1:
        problems.append("README.md: expected exactly one Cargo features section")
        return

    section = readme.split(heading, 1)[1].split("\n## ", 1)[0]
    documented = set(FEATURE_ROW_RE.findall(section))
    expected = cargo_features(manifest)
    if documented != expected:
        problems.append(
            "README.md: feature table mismatch "
            f"(documented={sorted(documented)}, Cargo.toml={sorted(expected)})"
        )


def check_metadata(manifest: dict, readme: str, problems: list[str]) -> None:
    package = manifest.get("package", {})
    msrv = package.get("rust-version")
    if not isinstance(msrv, str) or f"Rust **{msrv}**" not in readme:
        problems.append("README.md: MSRV must match package.rust-version")

    expected_repo = "https://github.com/ThreatFlux/ollama_rust_sdk"
    if package.get("repository") != expected_repo:
        problems.append("Cargo.toml: package.repository must point to ThreatFlux/ollama_rust_sdk")

    cargo_text = (ROOT / "Cargo.toml").read_text(encoding="utf-8")
    if "yourusername" in cargo_text.lower() or "your.email" in cargo_text.lower():
        problems.append("Cargo.toml: placeholder package metadata is not allowed")


def check_claims(problems: list[str]) -> None:
    for relative in ("README.md", "src/lib.rs", "docs/ARCHITECTURE.md"):
        text = (ROOT / relative).read_text(encoding="utf-8")
        for claim in BANNED_CLAIMS:
            if claim.casefold() in text.casefold():
                problems.append(f"{relative}: unsupported absolute claim: {claim!r}")


def validate(manifest: dict) -> list[str]:
    problems = [
        f"{path}: required documentation file is missing"
        for path in REQUIRED_FILES
        if not (ROOT / path).is_file()
    ]
    if problems:
        return problems

    readme = (ROOT / "README.md").read_text(encoding="utf-8")
    check_metadata(manifest, readme, problems)
    check_features(readme, manifest, problems)
    check_quickstart(readme, problems)
    check_claims(problems)
    check_local_links(problems)
    return problems


def main() -> int:
    args = parse_args()
    manifest = load_manifest()
    if args.print_msrv:
        print(manifest["package"]["rust-version"])
        return 0

    problems = validate(manifest)
    if not problems:
        print("Documentation contract passed.")
        return 0

    for problem in problems:
        print(f"error: {problem}", file=sys.stderr)
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
