"""Exact provenance exclusions must never hide gameplay or policy changes."""
import copy
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

import core_equivalence as equivalence


class EquivalenceTests(unittest.TestCase):
    def test_only_enumerated_leaf_provenance_is_removed(self):
        original = {'source_sha': 'old', 'binary_sha256': 'old',
                    'input_hashes': {'summary_sha256': 'old', 'receipt_sha256': 'old',
                                     'why_checker_sha256': 'checker'},
                    'policy_sha256': 'policy', 'count': 3, 'passed': True}
        updated = copy.deepcopy(original)
        updated['source_sha'] = updated['binary_sha256'] = 'new'
        updated['input_hashes']['summary_sha256'] = 'new'
        self.assertEqual(equivalence.payload(original, 'why'), equivalence.payload(updated, 'why'))
        for key, value in [('count', 4), ('policy_sha256', 'changed'), ('passed', False)]:
            changed = {**updated, key: value}
            self.assertNotEqual(equivalence.payload(original, 'why'), equivalence.payload(changed, 'why'))
        updated['input_hashes']['why_checker_sha256'] = 'changed'
        self.assertNotEqual(equivalence.payload(original, 'why'), equivalence.payload(updated, 'why'))

    def test_overbroad_or_missing_or_duplicate_pointer_rejected(self):
        for pointers in (['/input_hashes'], ['/policy_sha256'], ['/source_sha', '/source_sha'], []):
            contract = copy.deepcopy(equivalence.CONTRACT)
            contract['artifacts']['why'] = pointers
            with self.assertRaises(equivalence.EquivalenceError):
                equivalence.validate_contract(contract)

    def test_exact_decimal_never_rounds_small_mutations(self):
        left = equivalence.parse('{"x":0.100000000000000000000000000001}')
        right = equivalence.parse('{"x":0.100000000000000000000000000002}')
        self.assertNotEqual(equivalence.canonical(left), equivalence.canonical(right))
        self.assertEqual(equivalence.canonical(equivalence.parse('{"x":1.00}')),
                         equivalence.canonical(equivalence.parse('{"x":1e0}')))

    def test_duplicate_keys_nonfinite_and_missing_pointer_rejected(self):
        for raw in ('{"a":1,"a":2}', '{"a":NaN}', '{"a":Infinity}'):
            with self.assertRaises(equivalence.EquivalenceError):
                equivalence.parse(raw)
        with self.assertRaises(equivalence.EquivalenceError):
            equivalence.payload({}, 'why')

    def test_colony_has_no_exclusions_and_is_literal(self):
        with tempfile.TemporaryDirectory() as directory:
            left, right = Path(directory) / 'a.json', Path(directory) / 'b.json'
            left.write_text('{"count":1}')
            right.write_text('{ "count":1 }')
            result = equivalence.compare(left, right, 'colony_summary')
            self.assertEqual(result['reference_payload_sha256'], result['candidate_payload_sha256'])
            self.assertFalse(result['equal'])

    def test_cli_requires_committed_contract_and_checks_payload(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            repo = root / 'repo'
            repo.mkdir()
            subprocess.run(['git', 'init', '-q', str(repo)], check=True)
            for name, content in ((equivalence.SCRIPT_PATH, Path(equivalence.__file__).read_bytes()),
                                  (equivalence.CONTRACT_PATH, json.dumps(equivalence.CONTRACT).encode())):
                path = repo / name
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_bytes(content)
            subprocess.run(['git', '-C', str(repo), 'add', '.'], check=True)
            subprocess.run(['git', '-C', str(repo), '-c', 'user.name=Test', '-c', 'user.email=test@example.invalid',
                            'commit', '-qm', 'Commit exact test comparison contract'], check=True)
            left, right = root / 'left.json', root / 'right.json'
            left.write_text('{"n":1}')
            right.write_text('{"n":1}')
            command = [sys.executable, str(repo / equivalence.SCRIPT_PATH), '--artifact', 'colony_summary',
                       '--reference', str(left), '--candidate', str(right), '--reference-sha256',
                       hashlib.sha256(left.read_bytes()).hexdigest(), '--output', str(root / 'equal.json')]
            result = subprocess.run(command, capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            right.write_text('{"n":2}')
            command[-1] = str(root / 'different.json')
            self.assertEqual(subprocess.run(command, capture_output=True).returncode, 1)
            (repo / equivalence.CONTRACT_PATH).write_text('{}')
            command[-1] = str(root / 'invalid.json')
            self.assertNotEqual(subprocess.run(command, capture_output=True).returncode, 0)
            self.assertFalse((root / 'invalid.json').exists())


if __name__ == '__main__':
    unittest.main()
