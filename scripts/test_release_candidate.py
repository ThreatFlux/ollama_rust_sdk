"""Regression coverage for tagging a reviewed release proposal."""

import copy
import os
import unittest
from unittest import mock

from check_release_candidate import check_proposal_base, find_proposal, main


class ReleaseCandidateTests(unittest.TestCase):
    repository = "ThreatFlux/ollama_rust_sdk"
    proposal = {
        "merged_at": "2026-11-22T15:00:00Z",
        "merge_commit_sha": "release",
        "base": {"ref": "main"},
        "title": "chore: release v1.2.4",
        "head": {"sha": "proposal", "repo": {"full_name": repository}},
    }

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

    def test_release_like_title_cannot_authorize_carried_forward_version(self):
        candidate = "a" * 40
        proposal = copy.deepcopy(self.proposal)
        proposal["merge_commit_sha"] = candidate
        parents = {"parents": [{"sha": "reviewed-base"}]}
        with (
            mock.patch.dict(
                os.environ,
                {"RELEASE_CANDIDATE": candidate, "GITHUB_REPOSITORY": self.repository},
            ),
            mock.patch("check_release_candidate.package_version", return_value="1.2.4"),
            mock.patch("check_release_candidate.github_api", side_effect=[[], parents, parents]),
            mock.patch("check_release_candidate.associated_pull_requests", return_value=[proposal]),
            self.assertRaisesRegex(ValueError, "did not introduce"),
        ):
            main()
