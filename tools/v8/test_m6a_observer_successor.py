"""Frozen product bytes cannot be hidden by an observer tooling successor."""
from __future__ import annotations

import hashlib
import json
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

from m6a_evidence import EvidenceError, Node, source_hash, reference_json, write_json
import m6a_observer_successor as successor
from m6a_gate_source import FIELDS, validate_schema
from m6a_observer_successor import ALLOWED, changes, committed_inputs, inventory_hash, verify_live


class ObserverSuccessorTests(unittest.TestCase):
    def test_gate_schema_keeps_legacy_and_requires_explicit_adapter(self) -> None:
        legacy = dict.fromkeys(FIELDS, None)
        legacy.update(schema_version=1, kind='m6a_simulation_successor_v1',
                      base_head='5b0c88812374b87e7854cc9203f144ad0ccd9009')
        validate_schema(Node(legacy))
        successor = {**legacy, 'schema_version': 2, 'kind': 'm6a_observer_successor_v2',
                     'base_observer': {}, 'observer_successor': {}}
        validate_schema(Node(successor))
        for broken in ({**legacy, 'base_observer': {}},
                       {key: value for key, value in successor.items() if key != 'observer_successor'},
                       {**successor, 'schema_version': 1},
                       {**successor, 'allowlist': []}):
            with self.subTest(data=broken), self.assertRaises(EvidenceError):
                validate_schema(Node(broken))

    def fixture(self) -> tuple[dict[str, str], dict[str, str]]:
        old = {'rust/product.rs': 'a' * 64, 'tools/v8/m6a_score.py': 'b' * 64,
               'tools/v8/m6a_gate_source.py': 'c' * 64,
               'tools/v8/m6a_gate_evidence.py': 'd' * 64}
        new = {**old, **dict.fromkeys(ALLOWED, 'e' * 64)}
        return old, new

    def test_exact_seven_tool_changes_preserve_product(self) -> None:
        old, new = self.fixture()  # Given frozen product and exact tool patch
        result = changes(old, new)  # When comparing committed inventories
        self.assertEqual({row['path'] for row in result}, ALLOWED)  # Then only reviewed paths

    def test_product_helper_and_unlisted_tool_changes_fail(self) -> None:
        for path in ('rust/product.rs', 'tools/v8/m6a_score.py', 'tools/new.py',
                     'localization/compiled/ko.json', 'assets/policy.json'):
            old, new = self.fixture()
            new[path] = 'f' * 64
            with self.subTest(path=path), self.assertRaises(EvidenceError):
                changes(old, new)

    def test_deleted_allowed_file_fails(self) -> None:
        old, new = self.fixture()
        del new['tools/v8/m6a_gate_source.py']
        with self.assertRaises(EvidenceError):
            changes(old, new)

    def test_unreviewed_partial_allowlist_fails(self) -> None:
        old, new = self.fixture()
        del new['tools/v8/m6a_base72_entry.rs']
        with self.assertRaises(EvidenceError):
            changes(old, new)

    def test_inventory_hash_binds_names_and_order(self) -> None:
        value = {'rust/z': 'b' * 64, 'rust/a': 'a' * 64}
        expected = hashlib.sha256(b'rust/a\0' + b'a' * 64 + b'\nrust/z\0' + b'b' * 64 + b'\n').hexdigest()
        self.assertEqual(inventory_hash(value), expected)
        self.assertNotEqual(inventory_hash({'rust/renamed': 'a' * 64}), inventory_hash({'rust/a': 'a' * 64}))

    def test_complete_proof_rejects_tampering_without_rewriting_frozen_evidence(self) -> None:
        # Given an actual committed product and separately pinned original evidence.
        with tempfile.TemporaryDirectory() as directory:
            root, evidence = Path(directory)/'repo', Path(directory)/'evidence'
            root.mkdir()
            evidence.mkdir()
            def git(*args: str) -> str:
                return subprocess.check_output(['git', '-C', str(root), *args], text=True,
                                               stderr=subprocess.DEVNULL).strip()
            git('init')
            git('config', 'user.name', 'Fixture')
            git('config', 'user.email', 'fixture@example.invalid')
            for name in ('rust/Cargo.lock', 'rust/product.rs', 'tools/v8/m6a_run.py',
                         'tools/v8/m6a_score.py', 'tools/v8/m6a_gate_source.py',
                         'tools/v8/m6a_gate_evidence.py', 'docs/v8/m6a/M6A_GOAL.md',
                         'docs/v8/m6a/M6A_DESIGN.md'):
                target = root/name
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_text(name)
            git('add', '.')
            git('commit', '-qm', 'original')
            original = git('rev-parse', 'HEAD')
            original_hash = source_hash(root, original)
            probe = evidence/'probe'
            probe.write_bytes(b'fixture binary bytes only')
            write_json(evidence/'build.json', {'source_head': original,
                'probe': reference_json(probe), 'lock': reference_json(root/'rust/Cargo.lock'),
                'lock_sha256': successor.digest(root/'rust/Cargo.lock')})
            cohort = {'source_head': original, 'source_hash': original_hash, 'exit_code': 0,
                'source_unchanged': True, 'observer': reference_json(root/'tools/v8/m6a_run.py'),
                'scorer': reference_json(root/'tools/v8/m6a_score.py')}
            write_json(evidence/'cal4.json', cohort)
            rows = []
            for number in range(1, 5):
                rows.extend([{'event': 'started', 'round': number}, {'event': 'finished',
                    'round': number, 'exit_code': 0, 'receipt': reference_json(evidence/'cal4.json')}])
            (evidence/'register.jsonl').write_text(''.join(json.dumps(row)+'\n' for row in rows))
            write_json(evidence/'freeze.json', {'source_head': original, 'source_hash': original_hash,
                'calibration_register': reference_json(evidence/'register.jsonl'),
                'binary_sha256': successor.digest(probe),
                'scorer_sha256': successor.digest(root/'tools/v8/m6a_score.py')})
            write_json(evidence/'candidate.json', {**cohort, 'binary': reference_json(probe),
                'freeze': reference_json(evidence/'freeze.json')})
            names = {'freeze': 'freeze.json', 'register': 'register.jsonl', 'calibration': 'cal4.json',
                     'candidate': 'candidate.json', 'candidate_build': 'build.json'}
            pins = {key: (name, successor.digest(evidence/name)) for key, name in names.items()}
            for name in ALLOWED:
                (root/name).write_text('reviewed observer correction: '+name)
            git('add', '.')
            git('commit', '-qm', 'tools successor')
            head = git('rev-parse', 'HEAD')
            binding = Node({'deadline_epoch': successor.DEADLINE,
                'calibration': {key: reference_json(evidence/names[key]) for key in ('freeze', 'register')},
                'heldout': {'candidate': reference_json(evidence/'candidate.json')},
                'candidate_build': reference_json(evidence/'build.json')})
            proof_path = evidence/'proof.json'
            # Fixture pins stand in for immutable production pins; no production artifacts are edited.
            with patch.object(successor, 'P9', original), patch.object(successor, 'PINS', pins):
                successor.prepare(root, evidence, proof_path)
                proof = Node.load(proof_path).value
                self.assertEqual(successor.validate(root, proof_path, head, binding), original_hash)
                for field, bad in [('original_source_hash', '0'*64), ('successor_head', original),
                                   ('changes', []), ('extra_allowlist', ['rust/product.rs'])]:
                    with self.subTest(field=field):
                        proof_path.write_text(json.dumps({**proof, field: bad}))
                        with self.assertRaises(EvidenceError):
                            successor.validate(root, proof_path, head, binding)
                proof_path.write_text(json.dumps(proof))
                with self.assertRaises(EvidenceError):
                    successor.validate(root, proof_path, head,
                                       Node({**binding.value, 'deadline_epoch': successor.DEADLINE + 1}))
                (evidence/'freeze.json').write_text('{}')
                with self.assertRaises(EvidenceError):
                    successor.validate(root, proof_path, head, binding)

    def test_historical_hash_matches_real_source_algorithm_and_live_guard(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            def git(*args: str) -> str:
                return subprocess.check_output(['git', '-C', str(root), *args], text=True,
                                               stderr=subprocess.DEVNULL).strip()
            git('init')
            git('config', 'user.name', 'Fixture')
            git('config', 'user.email', 'fixture@example.invalid')
            (root/'rust').mkdir()
            (root/'rust/product.rs').write_text('original')
            git('add', '.')
            git('commit', '-qm', 'fixture')
            head = git('rev-parse', 'HEAD')
            committed = committed_inputs(root, head)
            self.assertEqual(inventory_hash(committed), source_hash(root, head))
            verify_live(root, head, committed)
            (root/'rust/untracked.rs').write_text('extra')
            with self.assertRaises(EvidenceError):
                verify_live(root, head, committed)
            (root/'rust/untracked.rs').unlink()
            (root/'rust/product.rs').write_text('changed')
            with self.assertRaises(EvidenceError):
                verify_live(root, head, committed)


if __name__ == '__main__':
    unittest.main()
