"""Bind an untagged version on main to the merged release proposal."""

import base64
import http.client
import json
import os
import re
import tomllib
from urllib.parse import quote


def github_api(path):
    connection = http.client.HTTPSConnection("api.github.com", timeout=30)
    try:
        connection.request(
            "GET",
            path,
            headers={
                "Accept": "application/vnd.github+json",
                "Authorization": f"Bearer {os.environ['GH_TOKEN']}",
                "User-Agent": "ollama-rust-sdk-release-policy",
            },
        )
        response = connection.getresponse()
        if response.status != 200:
            raise ValueError(f"GitHub API returned HTTP {response.status} for {path}")
        return json.load(response)
    finally:
        connection.close()


def associated_pull_requests(repo_path, candidate):
    page_number = 1
    while True:
        page = github_api(f"{repo_path}/commits/{candidate}/pulls?per_page=100&page={page_number}")
        yield from page
        if len(page) < 100:
            return
        page_number += 1


def package_version(repo_path, commit):
    manifest = github_api(f"{repo_path}/contents/Cargo.toml?ref={commit}")
    return tomllib.loads(base64.b64decode(manifest["content"]).decode())["package"]["version"]


def find_proposal(pull_requests, repository, candidate, version):
    """Return the reviewed proposal head associated with this merge commit."""
    release_titles = {f"chore(release): v{version}", f"chore: release v{version}"}
    for pr in pull_requests:
        if (
            pr["merged_at"]
            and pr["merge_commit_sha"] == candidate
            and pr["base"]["ref"] == "main"
            and pr["title"] in release_titles
            and (pr["head"]["repo"] or {}).get("full_name") == repository
        ):
            return pr["head"]["sha"]
    raise ValueError(f"Commit {candidate} is not the merged proposal for v{version}")


def check_proposal_base(pr_head, proposal_parents, candidate_base):
    """Accept a first proposal [base] or a refreshed proposal [base, previous].

    Since github_actions v0.7.15 the release runtime keeps the previous proposal
    as a second parent when it refreshes automation/release, so the branch update
    is a fast-forward. The tree is still built from the tested base alone.
    """
    parents = [parent["sha"] for parent in proposal_parents]
    refreshed = len(parents) == 2 and parents[1] != candidate_base
    if not parents or parents[0] != candidate_base or not (len(parents) == 1 or refreshed):
        raise ValueError(f"Release proposal {pr_head} was not prepared from {candidate_base}")


def main():
    candidate = os.environ["RELEASE_CANDIDATE"]
    if not re.fullmatch(r"[0-9a-f]{40}", candidate):
        raise ValueError("Expected a full release candidate commit SHA")
    repository = os.environ["GITHUB_REPOSITORY"]
    repo_path = f"/repos/{repository}"
    version = package_version(repo_path, candidate)
    tags = github_api(f"{repo_path}/git/matching-refs/tags/{quote(f'v{version}', safe='')}")
    if any(tag["ref"] == f"refs/tags/v{version}" for tag in tags):
        return

    pr_head = find_proposal(
        associated_pull_requests(repo_path, candidate), repository, candidate, version
    )
    proposal = github_api(f"{repo_path}/git/commits/{pr_head}")
    target = github_api(f"{repo_path}/git/commits/{candidate}")
    candidate_base = target["parents"][0]["sha"]
    check_proposal_base(pr_head, proposal["parents"], candidate_base)
    if package_version(repo_path, candidate_base) == version:
        raise ValueError(f"Release proposal {pr_head} did not introduce v{version}")


if __name__ == "__main__":
    try:
        main()
    except (ValueError, OSError, http.client.HTTPException) as error:
        raise SystemExit(f"Release candidate verification failed: {error}") from error
