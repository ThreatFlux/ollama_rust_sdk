"""Regression coverage for tagging a reviewed release proposal."""

import copy
import os
import unittest
from unittest import mock

from check_release_candidate import check_proposal_base, find_proposal, main


class ReleaseCandidateTests(unittest.TestCase):
    repository = "ThreatFlux/ollama_rust_sdk"
    candidate = "a" * 40
    proposal = {
        "merged_at": "2026-11-22T15:00:00Z",
        "merge_commit_sha": "release",
        "base": {"ref": "main"},
        "title": "chore(release): v1.2.4",
        "head": {"sha": "proposal", "repo": {"full_name": repository}},
    }

    def setUp(self):
        self.enterContext(
            mock.patch.dict(
                os.environ,
                {
                    "RELEASE_CANDIDATE": self.candidate,
                    "GITHUB_REPOSITORY": self.repository,
                },
            )
        )

    def test_exact_reviewed_release_can_be_tagged(self):
        self.assertEqual(
            find_proposal([self.proposal], self.repository, "release", "1.2.4"),
            "proposal",
        )
        check_proposal_base("proposal", [{"sha": "reviewed-base"}], "reviewed-base")

    def test_newer_main_cannot_inherit_the_release_version(self):
        with self.assertRaisesRegex(ValueError, "not the merged proposal"):
            find_proposal([self.proposal], self.repository, "later-feature", "1.2.4")

    def test_unmerged_proposal_cannot_authorize_tagging(self):
        unmerged = copy.deepcopy(self.proposal)
        unmerged["merged_at"] = None
        with self.assertRaisesRegex(ValueError, "not the merged proposal"):
            find_proposal([unmerged], self.repository, "release", "1.2.4")

    def test_base_advancing_before_merge_requires_a_new_proposal(self):
        with self.assertRaisesRegex(ValueError, "was not prepared from"):
            check_proposal_base("proposal", [{"sha": "reviewed-base"}], "later-feature")

    def test_refreshed_proposal_can_be_tagged(self):
        # v0.7.15 refreshes keep the previous proposal as a second parent.
        check_proposal_base(
            "proposal",
            [{"sha": "reviewed-base"}, {"sha": "previous-proposal"}],
            "reviewed-base",
        )

    def test_refreshed_proposal_on_stale_base_is_rejected(self):
        with self.assertRaisesRegex(ValueError, "was not prepared from"):
            check_proposal_base(
                "proposal",
                [{"sha": "stale-base"}, {"sha": "reviewed-base"}],
                "reviewed-base",
            )

    def test_refreshed_proposal_with_extra_parents_is_rejected(self):
        with self.assertRaisesRegex(ValueError, "was not prepared from"):
            check_proposal_base(
                "proposal",
                [{"sha": "reviewed-base"}, {"sha": "previous-proposal"}, {"sha": "other"}],
                "reviewed-base",
            )

    def test_refreshed_proposal_repeating_the_base_is_rejected(self):
        with self.assertRaisesRegex(ValueError, "was not prepared from"):
            check_proposal_base(
                "proposal",
                [{"sha": "reviewed-base"}, {"sha": "reviewed-base"}],
                "reviewed-base",
            )

    def test_proposal_without_parents_is_rejected(self):
        with self.assertRaisesRegex(ValueError, "was not prepared from"):
            check_proposal_base("proposal", [], "reviewed-base")

    def test_untagged_refreshed_candidate_can_be_tagged(self):
        proposal = copy.deepcopy(self.proposal)
        proposal["merge_commit_sha"] = self.candidate
        refreshed = {"parents": [{"sha": "reviewed-base"}, {"sha": "previous-proposal"}]}
        squashed = {"parents": [{"sha": "reviewed-base"}]}
        with (
            mock.patch("check_release_candidate.package_version", side_effect=["1.2.4", "1.2.3"]),
            mock.patch("check_release_candidate.github_api", side_effect=[[], refreshed, squashed]),
            mock.patch("check_release_candidate.associated_pull_requests", return_value=[proposal]),
        ):
            main()

    def test_release_like_title_cannot_authorize_carried_forward_version(self):
        proposal = copy.deepcopy(self.proposal)
        proposal["merge_commit_sha"] = self.candidate
        parents = {"parents": [{"sha": "reviewed-base"}]}
        with (
            mock.patch("check_release_candidate.package_version", return_value="1.2.4"),
            mock.patch("check_release_candidate.github_api", side_effect=[[], parents, parents]),
            mock.patch("check_release_candidate.associated_pull_requests", return_value=[proposal]),
            self.assertRaisesRegex(ValueError, "did not introduce"),
        ):
            main()

    def test_already_tagged_version_does_not_require_a_proposal(self):
        with (
            mock.patch("check_release_candidate.package_version", return_value="1.2.4"),
            mock.patch(
                "check_release_candidate.github_api",
                return_value=[{"ref": "refs/tags/v1.2.4"}],
            ) as api,
        ):
            main()
        api.assert_called_once_with(f"/repos/{self.repository}/git/matching-refs/tags/v1.2.4")

    def test_unavailable_tag_lookup_fails_closed(self):
        with (
            mock.patch("check_release_candidate.package_version", return_value="1.2.4"),
            mock.patch("check_release_candidate.github_api", side_effect=ValueError("HTTP 503")),
            self.assertRaisesRegex(ValueError, "HTTP 503"),
        ):
            main()

    def test_untagged_candidate_uses_merge_and_proposal_from_exact_sha(self):
        proposal = copy.deepcopy(self.proposal)
        proposal["merge_commit_sha"] = self.candidate
        parents = {"parents": [{"sha": "reviewed-base"}]}
        repo_path = f"/repos/{self.repository}"
        with (
            mock.patch("check_release_candidate.package_version", side_effect=["1.2.4", "1.2.3"]),
            mock.patch(
                "check_release_candidate.github_api",
                side_effect=[
                    [{"ref": "refs/tags/v1.2.40"}],
                    [self.proposal] * 100,
                    [proposal],
                    parents,
                    parents,
                ],
            ) as api,
        ):
            main()
        self.assertEqual(
            api.call_args_list,
            [
                mock.call(f"{repo_path}/git/matching-refs/tags/v1.2.4"),
                mock.call(f"{repo_path}/commits/{self.candidate}/pulls?per_page=100&page=1"),
                mock.call(f"{repo_path}/commits/{self.candidate}/pulls?per_page=100&page=2"),
                mock.call(f"{repo_path}/git/commits/proposal"),
                mock.call(f"{repo_path}/git/commits/{self.candidate}"),
            ],
        )
