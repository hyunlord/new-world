"""Negative controls for the additive runtime successor boundary."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

import core_shadow_freeze as freeze
import core_shadow_successor as successor


class SuccessorTests(unittest.TestCase):
    def test_pending_provenance_cannot_pass(self):
        with self.assertRaisesRegex(freeze.FreezeError, 'pending'):
            successor.comparison_contract({'schema_version': 1, 'mode': 'pending'})

    def test_self_declared_projection_cannot_authorize_itself(self):
        with self.assertRaises(freeze.FreezeError):
            successor.comparison_contract({'schema_version': 1, 'mode': 'exclude_provenance'})

    def test_exact_delta_accepts_only_enumerated_change(self):
        before = {'rust/a.rs': 'a' * 64, successor.POLICY: successor.POLICY_SHA}
        after = {**before, 'rust/a.rs': 'b' * 64}
        successor.check_delta(before, after, {'rust/a.rs': {
            'before_sha256': 'a' * 64, 'after_sha256': 'b' * 64}})
        with self.assertRaises(freeze.FreezeError):
            successor.check_delta(before, after, {})

    def test_policy_change_rejected_even_when_listed(self):
        before = {successor.POLICY: successor.POLICY_SHA}
        after = {successor.POLICY: 'b' * 64}
        with self.assertRaises(freeze.FreezeError):
            successor.check_delta(before, after, {successor.POLICY: {
                'before_sha256': successor.POLICY_SHA, 'after_sha256': 'b' * 64}})

    def test_source_digest_matches_existing_length_prefix_algorithm(self):
        name = b'rust/a.rs'
        digest = hashlib.sha256()
        digest.update(len(name).to_bytes(8, 'little'))
        digest.update(name)
        digest.update(bytes.fromhex('a' * 64))
        self.assertEqual(successor.source_digest({'rust/a.rs': 'a' * 64}), digest.hexdigest())

    def test_path_escape_and_bool_digest_rejected(self):
        for inventory in ({'rust/../outside': 'a' * 64}, {'rust/a': True}):
            with self.assertRaises(freeze.FreezeError):
                successor.source_digest(inventory)

    def test_stale_before_hash_rejected(self):
        with self.assertRaises(freeze.FreezeError):
            successor.check_delta({'rust/a': 'a' * 64}, {'rust/a': 'b' * 64},
                                  {'rust/a': {'before_sha256': 'c' * 64, 'after_sha256': 'b' * 64}})

    def test_raw_tree_addition_or_mutation_invalidates_proof(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            repo, capture = root / 'repo', root / 'capture'
            repo.mkdir()
            capture.mkdir()
            raw = capture / 'summary.json'
            raw.write_text('{}')
            manifest = root / 'inventory.json'
            manifest.write_text(json.dumps({'root': str(capture), 'files': {'summary.json': freeze.digest(raw)}}))
            reference = {'path': str(manifest), 'sha256': freeze.digest(manifest), 'size': manifest.stat().st_size}
            self.assertEqual(successor.pinned_tree(repo, reference), capture)
            raw.write_text('{"changed":true}')
            with self.assertRaises(freeze.FreezeError):
                successor.pinned_tree(repo, reference)
            raw.write_text('{}')
            (capture / 'unexpected.json').write_text('{}')
            with self.assertRaises(freeze.FreezeError):
                successor.pinned_tree(repo, reference)

    def test_missing_or_extra_contract_fields_fail(self):
        for value in ({'mode': 'literal'}, {'schema_version': 1, 'mode': 'literal', 'exclude': ['policy']}):
            with self.assertRaises(freeze.FreezeError):
                successor.comparison_contract(value)

    def test_explicit_documentation_links_preserve_ancestry(self):
        first, second = 'a' * 40, 'b' * 40
        def git(_repo, *args):
            if args[0] == 'show':
                return ((successor.PREDECESSOR if args[-1] == first else first) + '\n').encode()
            if args[0] == 'diff':
                return b'docs/report.md\0'
            return b'unchanged-rust-tree\n'
        with patch.object(successor, 'git', side_effect=git):
            self.assertEqual(successor.documentation_links(Path('.'), [first, second]), second)
            with self.assertRaises(freeze.FreezeError):
                successor.documentation_links(Path('.'), [second])
        with patch.object(successor, 'git', side_effect=[successor.PREDECESSOR.encode(), b'tools/check.py\0']):
            with self.assertRaises(freeze.FreezeError):
                successor.documentation_links(Path('.'), [first])

    def test_source_digest_matches_perf_tracked_and_untracked_semantics(self):
        import core_perf_check as perf
        with tempfile.TemporaryDirectory() as directory:
            repo = Path(directory).resolve()
            subprocess.run(['git', 'init', '-q', str(repo)], check=True)
            (repo / 'rust').mkdir()
            (repo / 'rust/.gitignore').write_text('ignored.rs\n')
            (repo / 'rust/tracked.rs').write_text('tracked')
            subprocess.run(['git', '-C', str(repo), 'add', 'rust'], check=True)
            (repo / 'rust/untracked.rs').write_text('untracked')
            (repo / 'rust/ignored.rs').write_text('ignored')
            names = subprocess.check_output(['git', '-C', str(repo), 'ls-files', '-z', '--cached',
                                             '--others', '--exclude-standard', '--', 'rust']).split(b'\0')
            inventory = {os.fsdecode(name): freeze.digest(repo / os.fsdecode(name)) for name in names if name}
            self.assertIn('rust/untracked.rs', inventory)
            self.assertNotIn('rust/ignored.rs', inventory)
            with patch.object(perf, 'ROOT', repo):
                self.assertEqual(successor.source_digest(inventory), perf.source_hash())

    def test_evidence_links_reject_arbitrary_non_rust_changes(self):
        commit = 'a' * 40
        with patch.object(successor, 'git', side_effect=[successor.PREDECESSOR.encode(), b'tools/unrelated.py\0']):
            with self.assertRaises(freeze.FreezeError):
                successor.evidence_links(Path('.'), successor.PREDECESSOR,
                                         [{'commit': commit, 'changes': {'tools/unrelated.py': {}}}])

    def test_authorized_payload_mode_requires_explicit_pins(self):
        with self.assertRaises(freeze.FreezeError):
            successor.comparison_contract({'schema_version': 1, 'mode': 'canonical_payload_v1'})


if __name__ == '__main__':
    unittest.main()
