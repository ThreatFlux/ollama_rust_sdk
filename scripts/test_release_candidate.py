"""Regression coverage for tagging a reviewed release proposal."""

import copy
import unittest

from check_release_candidate import check_proposal_base, find_proposal


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
