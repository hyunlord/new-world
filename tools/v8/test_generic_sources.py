"""Real disposable Git repositories exercise source and ancestry admission."""
import os
import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import generic_sources
from generic_gate import GateContractError
from generic_sources import freeze_binding, freeze_source, inventory, published_binding


class SourceTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name) / "repo"
        self.root.mkdir()
        self.git("init", "-q")
        (self.root / ".gitignore").write_text("generated/\n")
        (self.root / "source.txt").write_text("original\n")
        self.git("add", ".gitignore", "source.txt")
        self.git("-c", "user.name=Test", "-c", "user.email=test@example.invalid", "commit", "-qm", "base")
        self.base = self.git("rev-parse", "HEAD").strip()
        self.git("update-ref", "refs/remotes/origin/v8/main", self.base)

    def git(self, *args: str) -> str:
        return subprocess.run(["git", "-C", str(self.root), *args], check=True,
                              capture_output=True, text=True).stdout

    def test_committed_tree_admitted(self) -> None:
        result = freeze_source(self.root, self.base)
        self.assertEqual(result["source_sha"], self.base)
        self.assertEqual(result["tracked_count"], 2)

    def origin(self) -> Path:
        remote = Path(self.temp.name) / "remote.git"
        subprocess.run(["git", "clone", "--bare", str(self.root), str(remote)],
                       check=True, capture_output=True)
        subprocess.run(["git", "--git-dir", str(remote), "update-ref",
                        "refs/heads/v8/main", self.base], check=True, capture_output=True)
        self.git("remote", "add", "origin", str(remote))
        return remote

    def test_fresh_remote_binding_fetches_actual_published_base(self) -> None:
        self.origin()
        self.git("update-ref", "-d", "refs/remotes/origin/v8/main")
        result = published_binding(self.root, self.base, "self_proof")
        self.assertEqual(result["remote_before_sha"], self.base)
        self.assertEqual(result["remote_after_sha"], self.base)
        self.assertEqual(result["base_product_sha"], self.base)
        self.assertFalse(result["publication_authority"])
        self.assertEqual(self.git("rev-parse", "origin/v8/main").strip(), self.base)

    def test_remote_movement_during_fetch_is_rejected(self) -> None:
        remote = self.origin()
        (self.root / "source.txt").write_text("candidate\n")
        self.git("add", "source.txt")
        self.git("-c", "user.name=Test", "-c", "user.email=test@example.invalid", "commit", "-qm", "candidate")
        candidate = self.git("rev-parse", "HEAD").strip()
        self.git("push", "origin", "HEAD:refs/heads/candidate")
        actual_git = generic_sources.git

        def advance_after_fetch(root, *args):
            result = actual_git(root, *args)
            if args[0] == "fetch":
                subprocess.run(["git", "--git-dir", str(remote), "update-ref",
                                "refs/heads/v8/main", candidate], check=True, capture_output=True)
            return result

        with (patch.object(generic_sources, "git", side_effect=advance_after_fetch),
              self.assertRaisesRegex(GateContractError, "moved during fetch")):
            published_binding(self.root, candidate, "publication")

    def test_dirty_harness_rejects_before_any_remote_operation(self) -> None:
        (self.root / "source.txt").write_text("dirty\n")
        with self.assertRaisesRegex(GateContractError, "dirty source"):
            published_binding(self.root, self.base, "self_proof")

    def test_wrong_head_rejected(self) -> None:
        with self.assertRaises(GateContractError):
            freeze_source(self.root, "f" * 40)

    def test_untracked_source_rejected(self) -> None:
        (self.root / "extra.txt").write_text("unexpected")
        with self.assertRaises(GateContractError):
            freeze_source(self.root, self.base)

    def test_hidden_tracked_mutations_rejected(self) -> None:
        for flag in ("--assume-unchanged", "--skip-worktree"):
            with self.subTest(flag=flag):
                self.git("update-index", flag, "source.txt")
                (self.root / "source.txt").write_text("changed\n")
                with self.assertRaises(GateContractError):
                    freeze_source(self.root, self.base)
                (self.root / "source.txt").write_text("original\n")
                self.git("update-index", "--no-assume-unchanged", "--no-skip-worktree", "source.txt")

    def test_staged_change_rejected(self) -> None:
        (self.root / "source.txt").write_text("changed\n")
        self.git("add", "source.txt")
        with self.assertRaises(GateContractError):
            freeze_source(self.root, self.base)

    def test_hidden_mutation_at_final_git_query_rejected(self) -> None:
        self.git("update-index", "--assume-unchanged", "source.txt")
        real_git = generic_sources.git
        head_queries = 0

        def mutate_at_end(root, *args):
            nonlocal head_queries
            result = real_git(root, *args)
            if args == ("rev-parse", "HEAD"):
                head_queries += 1
                if head_queries == 2:
                    (self.root / "source.txt").write_text("hidden end mutation\n")
            return result

        with (patch.object(generic_sources, "git", side_effect=mutate_at_end),
              self.assertRaisesRegex(GateContractError, "tracked bytes changed")):
            freeze_source(self.root, self.base)

    def test_executable_mode_change_rejected_even_if_git_ignores_it(self) -> None:
        self.git("config", "core.filemode", "false")
        (self.root / "source.txt").chmod(0o755)
        with self.assertRaises(GateContractError):
            freeze_source(self.root, self.base)

    def test_generated_inputs_need_independent_inventory(self) -> None:
        generated = self.root / "generated"
        generated.mkdir()
        (generated / "bridge").write_bytes(b"one")
        freeze_source(self.root, self.base)
        before = inventory(self.root, ["generated"])
        (generated / "bridge").write_bytes(b"two")
        self.assertNotEqual(before, inventory(self.root, ["generated"]))
        (generated / "extra").write_bytes(b"three")
        self.assertEqual(len(inventory(self.root, ["generated"])), 2)

    def test_missing_or_symlinked_input_rejected(self) -> None:
        for name in ("missing", "link"):
            if name == "link":
                os.symlink(self.root / "source.txt", self.root / name)
            with self.assertRaises(GateContractError):
                inventory(self.root, [name])

    def test_inventory_escape_rejected(self) -> None:
        for name in ("../outside", str(self.root / "source.txt")):
            with self.assertRaises(GateContractError):
                inventory(self.root, [name])

    def test_publication_binding_uses_unique_published_merge_base(self) -> None:
        (self.root / "source.txt").write_text("candidate\n")
        self.git("add", "source.txt")
        self.git("-c", "user.name=Test", "-c", "user.email=test@example.invalid", "commit", "-qm", "candidate")
        candidate = self.git("rev-parse", "HEAD").strip()
        result = freeze_binding(self.root, candidate, self.base, "publication")
        self.assertEqual(result["base_product_sha"], self.base)
        self.assertEqual(result["candidate_product_sha"], candidate)
        self.assertEqual(result["harness_sha"], candidate)

    def test_stale_published_main_rejected(self) -> None:
        with self.assertRaises(GateContractError):
            freeze_binding(self.root, self.base, "f" * 40, "publication")

    def test_self_proof_keeps_harness_and_products_distinct(self) -> None:
        result = freeze_binding(self.root, self.base, self.base, "self_proof")
        self.assertEqual(result["base_product_sha"], self.base)
        self.assertEqual(result["candidate_product_sha"], self.base)
        self.assertFalse(result["publication_authority"])

    def test_defect_proof_separates_harness_and_explicit_product(self) -> None:
        (self.root / "source.txt").write_text("deliberate defect\n")
        self.git("add", "source.txt")
        self.git("-c", "user.name=Test", "-c", "user.email=test@example.invalid", "commit", "-qm", "defect")
        defect = self.git("rev-parse", "HEAD").strip()
        result = freeze_binding(self.root, self.base, self.base, "defect_proof", candidate_sha=defect)
        self.assertEqual(result["harness_sha"], self.base)
        self.assertEqual(result["candidate_product_sha"], defect)
        with self.assertRaises(GateContractError):
            freeze_binding(self.root, self.base, self.base, "defect_proof")
        with self.assertRaises(GateContractError):
            freeze_binding(self.root, self.base, self.base, "publication", candidate_sha=defect)


if __name__ == "__main__":
    unittest.main()
