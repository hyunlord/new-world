"""Reject shared-source attestations, inherited waivers and incomplete M6a perf."""
from __future__ import annotations
import copy
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import m6a_perf as perf
import m6a_perf_config as config
from core_perf_check import CEILINGS, SCENES, Document, EvidenceError, Json, sha, write, loads, validate_schedule
from m6a_evidence import EvidenceError as ParseError, reference_json
from m6a_perf_validate import monitor_clean


class ProvenanceTests(unittest.TestCase):
    def test_strict_json_rejects_duplicate_and_nonfinite(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'config.json'
            for text in ('{"kind":"a","kind":"b"}', '{"deadline_epoch":NaN}'):
                path.write_text(text)
                with self.assertRaises(ParseError):
                    config.StrictDocument.read(path)

    def test_unequal_arm_trees_are_admitted_only_by_their_own_proof(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            candidate = 'c' * 40
            base_build_head = 'b' * 40
            command = root / 'commands.json'
            command.write_text('[]')
            proof_fields: dict[str, Json] = {}
            for role, head in [('base', base_build_head), ('new', candidate)]:
                binary, probe, log, lock = [root / f'{role}-{kind}' for kind in ('perf', 'probe', 'log', 'lock')]
                binary.write_text(role)
                probe.write_text(role + '-probe')
                log.write_text('build exited 0')
                lock.write_text('untracked lock-' + head)
                proof = root / f'{role}-build.json'
                write(proof, {'schema_version': 1, 'source_head': head,
                    'runtime_sha256': 'digest-' + head, 'lock_sha256': sha(lock),
                    'profile': 'release', 'exit_code': 0, 'command': 'cargo build --release',
                    'perf': reference_json(binary), 'probe': reference_json(probe), 'log': reference_json(log), 'lock': reference_json(lock)})
                proof_fields[role] = reference_json(binary)
                proof_fields[role + '_build'] = reference_json(proof)
            data: dict[str, Json] = {'schema_version': 1, 'kind': config.KIND,
                'base_head': config.BASE, 'candidate_head': candidate,
                'candidate_source_sha256': 'live', 'candidate_runtime_sha256': 'digest-' + candidate,
                'deadline_epoch': 9999999999, 'commands': reference_json(command), **proof_fields}

            def git(*args: str) -> str:
                if args == ('rev-parse', 'HEAD'):
                    return candidate
                if args[0] == 'rev-parse':
                    revision = args[1].split(':')[0]
                    return 'new-tree' if revision == candidate else 'old-tree'
                if args[0] == 'worktree':
                    return 'worktree /not-this-evidence'
                return ''

            def committed(_root: Path, head: str, scope: str) -> dict[str, str]:
                self.assertNotEqual(scope, 'rust/Cargo.lock')
                return {scope: sha(command)}

            original_sha = config.sha
            def checksum(path: Path) -> str:
                if path == config.ROOT / 'rust/Cargo.lock':
                    return original_sha(root / 'new-lock')
                return original_sha(command if path == config.ROOT / 'tools/v8/bench_script.json' else path)

            with patch.dict(config.os.environ, {}, clear=True), patch.object(config, 'git', side_effect=git), \
                 patch.object(config, 'source_hash', return_value='live'), \
                 patch.object(config, 'source_digest', side_effect=lambda _root, head: 'digest-' + head), \
                 patch.object(config, 'sha', side_effect=checksum), \
                 patch.object(config.inventory, 'git', return_value=b''), \
                 patch.object(config.inventory, 'committed', side_effect=committed):
                config.verify_config(Document.parse(data))
                bad = copy.deepcopy(data)
                bad['schema_version'] = True
                with self.assertRaises(EvidenceError):
                    config.verify_config(Document.parse(bad))
                bad = copy.deepcopy(data)
                bad['unexpected_waiver'] = True
                with self.assertRaises(EvidenceError):
                    config.verify_config(Document.parse(bad))
                swapped = copy.deepcopy(data)
                swapped['base'] = swapped['new']
                with self.assertRaises(EvidenceError):
                    config.verify_config(Document.parse(swapped))
                base_proof = root / 'base-build.json'
                proof_data = dict(config.StrictDocument.read(base_proof).values)
                original = copy.deepcopy(proof_data)
                for change in ('missing', 'mismatch'):
                    altered = copy.deepcopy(original)
                    if change == 'missing':
                        del altered['lock']
                    else:
                        altered['lock_sha256'] = '0' * 64
                    write(base_proof, altered)
                    data['base_build'] = reference_json(base_proof)
                    with self.subTest(lock=change), self.assertRaises(EvidenceError):
                        config.verify_config(Document.parse(data))
                proof_data['runtime_sha256'] = 'digest-' + candidate
                write(base_proof, proof_data)
                data['base_build'] = reference_json(base_proof)
                with self.assertRaises(EvidenceError):
                    config.verify_config(Document.parse(data))

    def test_inherited_waiver_rejected(self) -> None:
        with patch.dict(config.os.environ, {'CORE_P7_RUN': '1'}, clear=True):
            with self.assertRaises(EvidenceError):
                config.reject_waivers()


class AcceptanceTests(unittest.TestCase):
    def summaries(self) -> dict[str, Json]:
        return {'scenes': [{'label': scene, 'tick_mean_us': 100, 'tick_p99_us': 200} for scene in SCENES],
                'alloc_per_agent': 100, 'alloc_per_tile': 10,
                'ceilings': {name: 'PASS' for name in CEILINGS}}

    def test_side64_and_allocator_have_no_waiver(self) -> None:
        old = self.summaries()
        new = self.summaries()
        rows = Document.parse(new).rows('scenes')
        altered = [dict(row.values) for row in rows]
        altered[4]['tick_p99_us'] = 221
        new['scenes'] = altered
        new['alloc_per_tile'] = 12
        self.assertEqual(perf.failures(Document.parse(old), Document.parse(new)),
                         ['n200_side64/tick_p99_us>1.1', 'alloc_per_tile>1.1'])

    def test_missing_pairs_and_invalid_load_rejected(self) -> None:
        with self.assertRaises(EvidenceError):
            validate_schedule([])
        for value in ('1 2', 'NaN 1 1', '-1 1 1'):
            with self.assertRaises(EvidenceError):
                loads(value)

    def test_single_mean_just_above_bar_rejected(self) -> None:
        old = self.summaries()
        new = self.summaries()
        rows = [dict(row.values) for row in Document.parse(new).rows('scenes')]
        rows[0]['tick_mean_us'] = 110.01
        new['scenes'] = rows
        self.assertEqual(perf.failures(Document.parse(old), Document.parse(new)), ['n200/tick_mean_us>1.1'])

    def test_only_owned_measurement_pid_is_excluded(self) -> None:
        snapshot = '1 /tmp/worldsim_perf\n2 /tmp/worldsim_probe\n3 /bin/cargo\n'
        self.assertEqual(len(perf.competing_processes(snapshot)), 3)
        self.assertEqual(len(perf.competing_processes(snapshot, 1)), 2)

    def test_monitor_rejects_competing_measurement_and_gap(self) -> None:
        trace: dict[str, Json] = {'pid': 1, 'started': 10, 'ended': 11, 'deadline': 20,
            'deadline_exceeded': False, 'samples': [{'utc': 10, 'snapshot': '1 /tmp/worldsim_perf'}]}
        self.assertTrue(monitor_clean(Document.parse(trace)))
        trace['samples'] = [{'utc': 10, 'snapshot': '1 /tmp/worldsim_perf\n2 /tmp/worldsim_probe'}]
        self.assertFalse(monitor_clean(Document.parse(trace)))
        trace['samples'] = [{'utc': 10, 'snapshot': ''}]
        trace['ended'] = 13
        self.assertFalse(monitor_clean(Document.parse(trace)))

    def test_check_rejects_missing_receipt(self) -> None:
        from m6a_perf_validate import check
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / 'config.json').write_text('{}')
            with patch.object(perf, 'verify_config'), self.assertRaises(EvidenceError):
                check(root, require_receipt=True)

    def test_check_requires_same_config_before_reading_old_receipt(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            config_path = root / 'input.json'
            config_path.write_text('{}')
            output = root / 'out'
            output.mkdir()
            (output / 'config.json').write_text('{"old":true}')
            with self.assertRaises(EvidenceError):
                perf.check_config(config_path, output)


if __name__ == '__main__':
    unittest.main()
