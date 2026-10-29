"""BASE72 observation coverage and strict qualification boundaries; no simulation."""
from __future__ import annotations

import copy
import unittest
import tempfile
from pathlib import Path

import m6a_base72 as adapter
from m6a_evidence import EvidenceError, Json, Node
from m6a_base72 import compare_qualification, validate_world


class Base72Tests(unittest.TestCase):
    def summary(self, days: int = 72) -> dict[str, Json]:
        return {'scene': 'colony', 'days': days, 'seeds': [{
            'seed': 0, 'days': days, 'ledger_violation_ticks': 0,
            'reservation_violation_ticks': 0,
            'mux': {'ticks_per_day': 1440, 'stockpile_first_tick': None},
            'mux2': {'start_tick': 0, 'end_tick': days * 1440,
                     'observed_ticks': days * 1440, 'observer_integrity_errors': [],
                     'wolf_deaths': 1, 'wolf_death_events': [{'tick': days * 1440 - 1,
                         'victim': 7, 'attacker': 8, 'kind': 0, 'mechanic': 'predator'}],
                     'inputs': {'steward': True, 'defence': True, 'story': 'on',
                         'neighbours': 'on', 'player_script': False,
                         'policy_sha256': 'policy', 'source_sha': 'base',
                         'defence_enabled_end': True}}}]}

    def test_complete_72_and_late_predator(self) -> None:
        validate_world(Node(self.summary()), 0, 72, 'policy', 'base')

    def test_reject_short_horizon_and_integrity_error(self) -> None:
        for key, value in [('end_tick', 69120), ('observed_ticks', 69120),
                           ('start_tick', 1), ('observer_integrity_errors', ['missing'])]:
            row = self.summary(); row['seeds'][0]['mux2'][key] = value
            with self.assertRaises(EvidenceError):
                validate_world(Node(row), 0, 72, 'policy', 'base')

    def test_reject_wrong_inputs_or_coverage(self) -> None:
        for key, value in [('steward', False), ('defence', False), ('player_script', True),
                           ('story', 'off'), ('neighbours', 'off'), ('policy_sha256', 'wrong')]:
            row = self.summary(); row['seeds'][0]['mux2']['inputs'][key] = value
            with self.assertRaises(EvidenceError):
                validate_world(Node(row), 0, 72, 'policy', 'base')
        for field, value in [('seed', 1), ('days', 48)]:
            row = self.summary(); row['seeds'][0][field] = value
            with self.assertRaises(EvidenceError):
                validate_world(Node(row), 0, 72, 'policy', 'base')

    def test_reject_bad_fatal_events(self) -> None:
        for mutation in ('duplicate', 'future', 'count'):
            row = self.summary(); mux = row['seeds'][0]['mux2']
            if mutation == 'duplicate':
                mux['wolf_death_events'] *= 2; mux['wolf_deaths'] = 2
            elif mutation == 'future': mux['wolf_death_events'][0]['tick'] = 103680
            else: mux['wolf_deaths'] = 0
            with self.assertRaises(EvidenceError):
                validate_world(Node(row), 0, 72, 'policy', 'base')

    def test_72_only_mux2_exclusion(self) -> None:
        adapter = self.summary(); original = copy.deepcopy(adapter)
        del original['seeds'][0]['mux2']
        compare_qualification(Node(original), Node(adapter), 72)
        adapter['seeds'][0]['ledger_violation_ticks'] = 1
        with self.assertRaises(EvidenceError):
            compare_qualification(Node(original), Node(adapter), 72)

    def test_72_rejects_bool_integer_coercion(self) -> None:
        candidate = self.summary(); original = copy.deepcopy(candidate)
        del original['seeds'][0]['mux2']
        original['seeds'][0]['flag'] = False
        candidate['seeds'][0]['flag'] = 0
        with self.assertRaises(EvidenceError):
            compare_qualification(Node(original), Node(candidate), 72)

    def test_48_full_equality_no_exclusion(self) -> None:
        original = self.summary(48); adapter = copy.deepcopy(original)
        compare_qualification(Node(original), Node(adapter), 48)
        adapter['seeds'][0]['mux2']['wolf_deaths'] = 2
        with self.assertRaises(EvidenceError):
            compare_qualification(Node(original), Node(adapter), 48)


    def test_manifest_rejects_unknown_missing_and_protocol_fields(self) -> None:
        base: dict[str, Json] = {key: {} for key in adapter.MANIFEST}
        base.update(schema_version=1, kind='m6a_base72_observer_adapter_v1',
                    base=adapter.BASE, base_library_delta='',
                    entry_delta='validated_observer_horizon_48_to_72')
        variants = []
        extra = copy.deepcopy(base); extra['override'] = True; variants.append(extra)
        missing = copy.deepcopy(base); del missing['build']; variants.append(missing)
        for key, value in [('base', 'foreign'), ('base_library_delta', 'changed'),
                           ('entry_delta', ''), ('schema_version', 2)]:
            row = copy.deepcopy(base); row[key] = value; variants.append(row)
        with tempfile.TemporaryDirectory() as directory:
            import json
            path = Path(directory) / 'manifest.json'
            for row in variants:
                path.write_text(json.dumps(row))
                with self.assertRaises(EvidenceError):
                    adapter.validate_manifest(Path(directory), path)

    def test_reference_tampering_and_internal_artifact_reject(self) -> None:
        from m6a_evidence import reference_json
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory); path = root / 'artifact'; path.write_text('original')
            reference = Node(reference_json(path))
            with self.assertRaises(EvidenceError): adapter.ref(root, reference)
            path.write_text('changed')
            with self.assertRaises(EvidenceError): adapter.ref(root / 'other', reference)

    def test_qualification_rejects_extra_original_observer_or_missing_candidate(self) -> None:
        with self.assertRaises(EvidenceError):
            compare_qualification(Node(self.summary()), Node(self.summary()), 72)
        a = self.summary(); del a['seeds'][0]['mux2']
        with self.assertRaises(EvidenceError):
            compare_qualification(Node(a), Node(copy.deepcopy(a)), 72)

    def test_command_has_exact_declared_observation_inputs(self) -> None:
        argv = adapter.command(Path('/frozen/adapter'), 1296, Path('/evidence/seed-1296'))
        self.assertEqual(argv[argv.index('--days') + 1], '72')
        self.assertEqual(argv[argv.index('--seeds') + 1], '1296..1297')
        self.assertEqual(argv[argv.index('--mux2-policy-sha256') + 1], adapter.POLICY)
        self.assertNotIn('--commands', argv)
        self.assertEqual(argv.count('--days'), 1)


class InvocationTests(unittest.TestCase):
    def test_actual_consumed_inputs_and_tampering(self) -> None:
        from m6a_evidence import Reference, digest
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory); (root / 'deps').mkdir(); (root / 'generated').mkdir()
            for name in ('compiler', 'lib.rs', 'deps/libdep.rlib', 'generated/output.rs'):
                (root / name).write_text(name)
            inputs = {(Reference.capture(p).sha256, p.stat().st_size)
                      for p in [root/'deps/libdep.rlib', root/'generated/output.rs']}
            source = {str((root/'lib.rs').resolve()): digest(root/'lib.rs')}
            row = {'argv': [str(root/'compiler'), '--crate-name', 'sample', 'lib.rs',
                           '--extern', 'dep=deps/libdep.rlib', '-Ldependency=deps'],
                   'cwd': str(root), 'exit_code': 0, 'environment': {'OUT_DIR': 'generated'}}
            adapter.validate_invocation(Node(row), digest(root/'compiler'), source, inputs)
            for label, mutate in [
                ('compiler', lambda x: x['argv'].__setitem__(0, str(root/'lib.rs'))),
                ('cwd', lambda x: x.__setitem__('cwd', str(root/'deps'))),
                ('extern', lambda x: x['argv'].__setitem__(5, 'dep=lib.rs')),
                ('outdir', lambda x: x['environment'].__setitem__('OUT_DIR', 'missing')),
            ]:
                wrong = copy.deepcopy(row); mutate(wrong)
                with self.subTest(label=label), self.assertRaises((EvidenceError, OSError)):
                    adapter.validate_invocation(Node(wrong), digest(root/'compiler'), source, inputs)
            (root/'generated/output.rs').write_text('tampered')
            with self.assertRaises(EvidenceError):
                adapter.validate_invocation(Node(row), digest(root/'compiler'), source, inputs)

    def test_only_authenticated_proc_macro_bare_extern(self) -> None:
        from m6a_evidence import Reference, digest
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory); compiler = root/'compiler'; compiler.write_text('rustc')
            source = root/'lib.rs'; source.write_text('source')
            builtin = root/'libproc_macro-example.rlib'; builtin.write_text('builtin')
            inputs = {(Reference.capture(builtin).sha256, builtin.stat().st_size)}
            row = Node({'argv': [str(compiler), str(source), '--extern', 'proc_macro'],
                        'cwd': str(root), 'exit_code': 0})
            sources = {str(source.resolve()): digest(source)}
            adapter.validate_invocation(row, digest(compiler), sources, inputs,
                                        builtin_externs={'proc_macro': builtin})
            with self.assertRaises(EvidenceError):
                adapter.validate_invocation(row, digest(compiler), sources, inputs)
            with self.assertRaises(EvidenceError):
                adapter.validate_invocation(row, digest(compiler), sources, set(),
                                            builtin_externs={'proc_macro': builtin})
            wrong = copy.deepcopy(row.value); wrong['argv'][-1] = 'unknown'
            with self.assertRaises(EvidenceError):
                adapter.validate_invocation(Node(wrong), digest(compiler), sources, inputs,
                                            builtin_externs={'unknown': builtin})

    def test_registry_source_and_probe_scope(self) -> None:
        from m6a_evidence import Reference, digest
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory); compiler = root/'compiler'; compiler.write_text('rustc')
            source = root/'lib.rs'; source.write_text('source')
            reference = Reference.capture(source)
            inputs = {(reference.sha256, reference.size)}
            row = {'argv': [str(compiler), str(source)], 'cwd': str(root), 'exit_code': 0}
            adapter.validate_invocation(Node(row), digest(compiler), {}, inputs)
            with self.assertRaises(EvidenceError):
                adapter.validate_invocation(Node(row), digest(compiler), {}, set())
            row['argv'] = [str(compiler), '-vV']
            adapter.validate_invocation(Node(row), digest(compiler), {}, set())
            row['argv'] += ['-o', 'unexpected']
            with self.assertRaises(EvidenceError):
                adapter.validate_invocation(Node(row), digest(compiler), {}, set())


if __name__ == '__main__':
    unittest.main()
