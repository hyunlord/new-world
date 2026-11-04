"""M6a gate cannot inherit UI-only equality, P7 exceptions or incomplete gates."""
from __future__ import annotations

from pathlib import Path
import json
import tempfile
import unittest
from unittest.mock import patch

from m6a_evidence import EvidenceError, Node, reference_json
from m6a_gate_source import CONFLICTING, Gate, environment, keys
from m6a_gate_evidence import gate_steps
from m6a_gate_art import structural_command, validate as validate_plant_art
from m6a_gate_source import SCOPES

ROOT = Path(__file__).resolve().parents[2]


class GateContractTests(unittest.TestCase):
    def test_plant_art_uses_real_renderer_without_capture_arguments(self) -> None:
        self.assertEqual(structural_command('/owned/godot'), [
            'bash', 'tools/v8/run_bounded.sh', '180', '/owned/godot',
            '--rendering-driver', 'opengl3', '--path', '.',
            '--script', 'res://tools/v8/m6a_plant_art_test.gd'])

    def test_plant_art_structural_checks_fail_closed(self) -> None:
        labels = ['data-only plant maps admitted', 'fixture content installs',
                  'duplicate selected maps rejected', 'species ripe phase wins',
                  'unbound category retains placeholder', 'malformed plant art rejected: []']
        for phase in range(4):
            labels.extend(f'{label} {phase}' for label in
                          ('one actual batch phase', 'native texture phase', 'native anchor phase', 'upright UV phase',
                           'unknown species uses category phase'))
        text = ''.join('M6A_PLANT_ART_PASS ' + label + '\n' for label in labels)
        text += f'M6A_PLANT_ART_DONE checks={len(labels)} failures=0\n'
        self.assertEqual(validate_plant_art(text), len(labels))
        for broken in (text.replace('failures=0', 'failures=1'),
                       text.replace('native anchor phase 3', 'missing phase'),
                       text.replace('upright UV phase 3', 'missing UV phase'),
                       text + 'SCRIPT ERROR: invalid property\n',
                       text + 'M6A_PLANT_ART_FAIL hidden failure\n',
                       text.replace(f'checks={len(labels)}', 'checks=0'),
                       text.split('M6A_PLANT_ART_DONE')[0]):
            with self.subTest(output=broken[-100:]), self.assertRaises(EvidenceError):
                validate_plant_art(broken)

    def test_plant_inputs_and_food_metadata_are_source_bound(self) -> None:
        for path in ('rust/crates/sim-core/data/colony/items.ron',
                     'tools/v8/m6a_plant_art_test.gd', 'tools/v8/fixtures/m6a_plant_art.json',
                     'scripts/world/colony/plant_layer.gd', 'assets/v8art/manifest.json'):
            self.assertTrue(any(path == scope or path.startswith(scope + '/') for scope in SCOPES), path)

    def test_mode_requires_independent_pin(self) -> None:
        with self.assertRaises(EvidenceError):
            environment({'M6A_GATE_CONFIG': '/tmp/config'})

    def test_rejects_every_inherited_mode_or_waiver(self) -> None:
        for name in (*CONFLICTING, 'CORE_P7_UNKNOWN_OVERRIDE'):
            with self.subTest(name=name), self.assertRaises(EvidenceError):
                environment({'M6A_GATE_CONFIG': '/tmp/config',
                             'M6A_GATE_CONFIG_SHA256': 'a' * 64, name: '1'})

    def test_m6a_does_not_activate_p7_parent(self) -> None:
        values = {'M6A_GATE_CONFIG': '/tmp/config', 'M6A_GATE_CONFIG_SHA256': 'a' * 64}
        environment(values)
        self.assertNotIn('CORE_P7_RUN', values)

    def test_strict_json_and_fields(self) -> None:
        for text in ('{"a":1,"a":2}', '{"a":NaN}', '{"a":Infinity}'):
            with self.subTest(text=text), self.assertRaises(EvidenceError):
                Node.parse(text)
        with self.assertRaises(EvidenceError):
            keys(Node.parse('{"a":1,"extra":2}'), {'a'})
        with self.assertRaises(EvidenceError):
            Node.parse('true').integer()

    def text(self) -> str:
        names = ['step' + str(index) for index in range(1, 49)]
        names[11], names[26] = 'perf', 'nav_bench'
        return 'head ' + 'a' * 40 + '\ndirty 0\n' + ''.join(
            f'{index} {name} exit 0 1s started 00:00:00\n'
            for index, name in enumerate(names, 1)) + 'finished now status 0\n'

    def test_complete_gate_and_report_only_semantics(self) -> None:
        self.assertEqual(len(gate_steps(self.text(), 'a' * 40)), 48)
        self.assertEqual(len(gate_steps(self.text().replace('perf exit 0', 'perf exit 1'), 'a' * 40)), 48)

    def test_no_stall_waiver_missing_steps_or_wrong_head(self) -> None:
        for text in (self.text().replace('41 step41 exit 0', '41 step41 exit 1'),
                     self.text().replace('48 step48 exit 0 1s started 00:00:00\n', ''),
                     self.text() + '48 step48 exit 0 1s started 00:00:00\n',
                     self.text().replace('finished now status 0', 'unfinished'),
                     self.text().replace('head ' + 'a' * 40, 'head ' + 'b' * 40),
                     self.text().replace('dirty 0', 'dirty 1')):
            with self.subTest(text=text[-100:]), self.assertRaises(EvidenceError):
                gate_steps(text, 'a' * 40)

    def test_build_binds_own_source_and_rejects_wrong_compiled_arm(self) -> None:
        proof = Node({'schema_version': 1, 'source_head': 'old-build',
            'runtime_sha256': 'own-digest', 'lock_sha256': 'lock', 'profile': 'release',
            'exit_code': 0, 'command': 'cargo build --release',
            'probe': {}, 'perf': {}, 'log': {}, 'lock': {}})
        gate = Gate(ROOT, Path('/tmp/config'), 'a' * 64,
                    Node({'base_build': {}}), 'candidate', Path('/tmp/gate'))
        def committed(root: Path, revision: str, scope: str) -> dict[str, str]:
            self.assertNotEqual(scope, 'rust/Cargo.lock')
            return {scope: 'same-tree'}
        with patch('m6a_gate_source.freeze.digest', return_value='lock'), \
             patch('m6a_gate_source.ref', return_value=Path('/tmp/proof')), \
             patch('m6a_gate_source.Node.load', return_value=proof), \
             patch('m6a_gate_source.source_digest', return_value='own-digest'), \
             patch('m6a_gate_source.inventory.git', return_value=b'worktree /owned\n'), \
             patch('m6a_gate_source.inventory.committed', side_effect=committed):
            self.assertEqual(gate.build('base_build', 'published-base'), proof)
            with patch('m6a_gate_source.source_digest', return_value='changed-digest'), self.assertRaises(EvidenceError):
                gate.build('base_build', 'published-base')
            with patch('m6a_gate_source.inventory.committed', side_effect=[{'x': 'a'}, {'x': 'b'}]), self.assertRaises(EvidenceError):
                gate.build('base_build', 'published-base')

    def test_frozen_untracked_lock_is_required_and_candidate_must_match(self) -> None:
        from core_shadow_freeze import digest
        with tempfile.TemporaryDirectory() as directory:
            external = Path(directory).resolve()
            repo = external / 'repo'
            (repo / 'rust').mkdir(parents=True)
            live = repo / 'rust/Cargo.lock'
            live.write_text('untracked cargo lock')
            frozen = external / 'Cargo.lock'
            frozen.write_bytes(live.read_bytes())
            artifact = external / 'artifact'
            artifact.write_text('preserved build artifact')
            proof = external / 'build.json'
            fields = {'schema_version': 1, 'source_head': 'build-head',
                'runtime_sha256': 'own-digest', 'lock_sha256': digest(frozen),
                'profile': 'release', 'exit_code': 0, 'command': 'cargo build --release',
                'probe': reference_json(artifact), 'perf': reference_json(artifact),
                'log': reference_json(artifact), 'lock': reference_json(frozen)}

            def build(name: str = 'base_build') -> Node:
                proof.write_text(json.dumps(fields))
                gate = Gate(repo, external / 'config', 'a' * 64,
                            Node({name: reference_json(proof)}), 'candidate', external / 'gate')
                return gate.build(name, 'expected-head')

            def committed(_root: Path, _head: str, scope: str) -> dict[str, str]:
                self.assertNotEqual(scope, 'rust/Cargo.lock')
                return {scope: 'same-tree'}

            with patch('m6a_gate_source.source_digest', return_value='own-digest'), \
                 patch('m6a_gate_source.inventory.committed', side_effect=committed), \
                 patch('m6a_gate_source.inventory.git', return_value=f'worktree {repo}\n'.encode()):
                build()
                build('candidate_build')
                lock_ref = fields.pop('lock')
                with self.assertRaises(EvidenceError):
                    build()
                fields['lock'] = lock_ref
                fields['lock_sha256'] = '0' * 64
                with self.assertRaises(EvidenceError):
                    build()
                fields['lock_sha256'] = digest(frozen)
                live.write_text('different live candidate lock')
                build()  # Historical BASE does not claim the candidate's lock.
                with self.assertRaises(EvidenceError):
                    build('candidate_build')
                frozen.write_text('tampered frozen bytes')
                with self.assertRaises(EvidenceError):
                    build()

    def test_dispatch_preserves_legacy_steps(self) -> None:
        text = (ROOT / 'tools/v8/gate.sh').read_text()
        for index in range(1, 49):
            self.assertRegex(text, rf'(?m)^\s*step {index} ')
        self.assertIn('M6A_GATE_CONFIG', text)
        helper = (ROOT / 'tools/v8/core_gate_checks.sh').read_text()
        self.assertIn('m6a_gate_checks.sh', helper)
        self.assertIn("calibration policy does not match frozen/current policy", helper)


if __name__ == '__main__':
    unittest.main()
