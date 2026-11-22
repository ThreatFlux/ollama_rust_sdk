"""Bind an untagged version on main to the merged release proposal."""

import json
import os
import subprocess
import tomllib


def command(*args):
    return subprocess.check_output(args, text=True).strip()


def find_proposal(pull_requests, repository, candidate, version):
    """Return the reviewed proposal head associated with this merge commit."""
    for pr in pull_requests:
        if (
            pr["merged_at"]
            and pr["merge_commit_sha"] == candidate
            and pr["base"]["ref"] == "main"
            and pr["title"] == f"chore: release v{version}"
            and (pr["head"]["repo"] or {}).get("full_name") == repository
        ):
            return pr["head"]["sha"]
    raise ValueError(f"Commit {candidate} is not the merged proposal for v{version}")


def check_proposal_base(pr_head, proposal_parents, candidate_base):
    if [parent["sha"] for parent in proposal_parents] != [candidate_base]:
        raise ValueError(f"Release proposal {pr_head} was not prepared from {candidate_base}")


def main():
    candidate = command("git", "rev-parse", "HEAD")
    version = tomllib.loads(command("git", "show", "HEAD:Cargo.toml"))["package"]["version"]
    tag_check = subprocess.run(
        ["git", "show-ref", "--verify", "--quiet", f"refs/tags/v{version}"], check=False
    )
    if tag_check.returncode == 0:
        return
    if tag_check.returncode != 1:
        tag_check.check_returncode()

    repository = os.environ["GITHUB_REPOSITORY"]
    pages = json.loads(
        command("gh", "api", "--paginate", "--slurp", f"repos/{repository}/commits/{candidate}/pulls")
    )
    pr_head = find_proposal(
        (pr for page in pages for pr in page), repository, candidate, version
    )
    proposal_commit = json.loads(command("gh", "api", f"repos/{repository}/git/commits/{pr_head}"))
    check_proposal_base(pr_head, proposal_commit["parents"], command("git", "rev-parse", "HEAD^1"))


if __name__ == "__main__":
    try:
        main()
    except (ValueError, subprocess.CalledProcessError) as error:
        raise SystemExit(f"Release candidate verification failed: {error}") from error
