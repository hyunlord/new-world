"""Synthetic binding tests; external source/build authority is explicitly mocked."""
import json
import tempfile
import unittest
from collections.abc import Generator
from contextlib import contextmanager
from dataclasses import dataclass, replace
from pathlib import Path
from unittest.mock import patch

from generic_bench import WORKLOADS, Product, compare_rounds
from generic_bench_safety import Aggregate, audit_bench_safety, children
from generic_gate import GateContractError, document
from generic_process import ProcessResult, ProcessSpec, environment_for, file_hash
from mux2_gate_inputs import Json, Node
from test_generic_bench import SHA, synthetic_round


@dataclass(frozen=True, slots=True)
class Scenario:
    root: Path
    script: Path
    aggregates: tuple[Aggregate, ...]


def put(path: Path, value: Json) -> None:
    _ = path.write_text(json.dumps(value))


@contextmanager
def scenario() -> Generator[Scenario, None, None]:
    with tempfile.TemporaryDirectory() as temporary:
        home = Path(temporary).resolve()
        root = home / 'evidence'
        root.mkdir()
        script = home / 'bench_script.json'
        _ = script.write_text('[]')
        products: dict[str, Product] = {}
        for arm in ('base', 'candidate'):
            product_root = home / arm
            product_root.mkdir()
            binary, rules, build = (product_root / name for name in ('probe', 'rules.ron', 'build.json'))
            _ = binary.write_text('synthetic, never executable')
            _ = rules.write_text('synthetic input')
            _ = build.write_text('{"synthetic":true}')
            products[arm] = Product(product_root, binary, SHA, file_hash(binary), build,
                                    file_hash(build), (('rules.ron', file_hash(rules)),))
        aggregates: list[Aggregate] = []
        for outer in range(1, 4):
            for arm in ('base', 'candidate'):
                index = len(aggregates)
                product = products[arm]
                out = root / f'{arm}-r{outer}'
                synthetic_round(out, 100 + index * 100)
                sidecars = out / 'gate_observations'
                sidecars.mkdir()
                parent_pid = 1000 + index
                argv = (str(product.binary), 'bench', '--out', str(out), '--commands', str(script),
                        '--repeats', '1', '--commit', SHA)
                spec = ProcessSpec(argv, product.root, product.binary, product.binary_sha256,
                                   product.root, product.input_files, out,
                                   (('WORLDSIM_GATE_OBSERVER', '1'), ('WORLDSIM_GIT_SHA', SHA)),
                                   external_inputs=((script, file_hash(script)),))
                for name in ('stdout.log', 'stderr.log'):
                    _ = (out / name).write_text('')
                process: dict[str, Json] = {
                    'schema_version': 1, 'status': 'PASS', 'native_exit': 0, 'pid': parent_pid, 'errors': [],
                    'argv': list(argv), 'cwd': str(product.root), 'executable': str(product.binary),
                    'input_root': str(product.root), 'output_root': str(out), 'fresh_directories': [],
                    'environment': dict(environment_for(spec)),
                    'environment_policy': 'fixed_defaults_plus_explicit_only_no_inheritance',
                    'expected_executable_sha256': product.binary_sha256,
                    'before_executable_sha256': product.binary_sha256, 'after_executable_sha256': product.binary_sha256,
                    'stdout_sha256': file_hash(out / 'stdout.log'), 'stderr_sha256': file_hash(out / 'stderr.log'),
                }
                for prefix in ('expected', 'before', 'after'):
                    process[prefix + '_inputs'] = dict(spec.inputs)
                    process[prefix + '_external_inputs'] = {str(script): file_hash(script)}
                put(out / 'process.json', process)
                result = ProcessResult(out / 'process.json', 'PASS', 0, parent_pid)
                aggregates.append(Aggregate(arm, outer, product, spec, result, file_hash(result.receipt)))
                journal: list[Json] = []
                for child_index, (kind, scene, repeat, filename) in enumerate(children()):
                    agents, _side, scripted, wolves = WORKLOADS[scene]
                    raw_path = out / 'runs' / filename
                    report = document(raw_path.read_text()).mapping()
                    report.update(alive=agents) if kind == 'rss' else report.update(founders=agents, alive_after_warmup=agents, gifts=1)
                    put(raw_path, report)
                    ticks = 0 if kind == 'rss' else 4320
                    native: dict[str, Json] = {
                        'schema_version': 1, 'initial_tick': 0, 'initial_population': agents,
                        'requested_ticks': ticks, 'requested_terminal_tick': ticks,
                        'terminal_tick': ticks, 'terminal_alive': agents, 'terminal_complete': True,
                        'history_complete': True, 'history_source': 'fresh_world_append_only_chronicle',
                        'knowledge_capability': {'enabled': True, 'source': 'resources.rules.colony.knowledge'},
                        'knowledge_lost': [], 'clock_offset_ticks': 480, 'ticks_per_day': 1440,
                        'deaths': {name: 0 for name in ('starvation', 'dehydration', 'exposure', 'combat', 'old_age')},
                    }
                    if kind == 'timed':
                        native.update(warmup_alive=agents, interventions={'gifts': 1, 'gift_requested': 3 * agents,
                            'gift_applied': 3 * agents, 'wolves_requested': wolves, 'wolves_before': 0, 'wolves_after': wolves})
                    pid = 10000 + index * 100 + child_index
                    put(sidecars / (filename + '.safety.json'), {'schema_version': 1, 'producer_pid': pid,
                        'raw_report_path': str(raw_path), 'raw_report_bytes': raw_path.read_text(), 'observation': native})
                    child_argv = [str(product.binary), 'bench-one']
                    if kind == 'rss': child_argv.extend(['--build-only', str(repeat)])
                    child_argv.extend(['--scene-label', scene, '--out', str(raw_path)])
                    if kind == 'timed' and scripted: child_argv.extend(['--commands', str(script)])
                    for offset, phase in enumerate(('launch', 'spawned', 'completed')):
                        journal.append({'schema': 'bench_child_process_v1', 'sequence': child_index * 3 + offset,
                            'child_index': child_index, 'phase': phase, 'parent_pid': parent_pid,
                            'pid': None if phase == 'launch' else pid, 'argv': list(child_argv),
                            'cwd': str(product.root), 'output': str(raw_path), 'kind': kind, 'scene': scene,
                            'repeat': repeat, 'native_exit': 0 if phase == 'completed' else None,
                            'success': True if phase == 'completed' else None, 'signal': None, 'error': None})
                _ = (out / 'child-processes.jsonl').write_text(''.join(json.dumps(row) + '\n' for row in journal))
        yield Scenario(root, script, tuple(aggregates))


def audit(case: Scenario) -> dict[str, Json]:
    # Source/build authority is an external dependency, not established by these synthetic files.
    with patch('generic_bench_safety.validate_products'), patch('generic_bench_safety.git_bytes', return_value=b''):
        return audit_bench_safety(case.root, case.aggregates, 'self_proof', case.script, file_hash(case.script))


class BenchSafetyTests(unittest.TestCase):
    def test_exact126_children63_pairs_without_publication_authority(self) -> None:
        with scenario() as case:
            result = audit(case)
            self.assertEqual((result['worlds'], result['pairs']), (126, 63))
            self.assertIs(result['publication_authority'], False)
            self.assertEqual(len(Node(result['safety']).array()), 63)

    def test_missing_extra_or_foreign_sidecars_rejected(self) -> None:
        for mode in ('missing', 'extra', 'foreign_pid', 'raw_bytes'):
            with self.subTest(mode=mode), scenario() as case:
                path = case.root / 'candidate-r3/gate_observations/rss_n200_r5.json.safety.json'
                if mode == 'missing': path.unlink()
                elif mode == 'extra': _ = path.with_name('extra.json').write_text('{}')
                else:
                    data = document(path.read_text()).mapping()
                    data['producer_pid' if mode == 'foreign_pid' else 'raw_report_bytes'] = 7 if mode == 'foreign_pid' else '{}'
                    put(path, data)
                with self.assertRaises(GateContractError): _ = audit(case)

    def test_failed_incomplete_or_foreign_child_process_rejected(self) -> None:
        for key, value in (('native_exit', 1), ('success', False), ('signal', 9), ('parent_pid', 7),
                           ('pid', 7), ('phase', 'wait_failed'), ('repeat', 9), ('sequence', 61)):
            with self.subTest(key=key), scenario() as case:
                path = case.root / 'candidate-r3/child-processes.jsonl'
                rows = [document(line).mapping() for line in path.read_text().splitlines()]
                rows[-1][key] = value
                _ = path.write_text(''.join(json.dumps(row) + '\n' for row in rows))
                with self.assertRaises(GateContractError): _ = audit(case)
        with scenario() as case:
            path = case.root / 'candidate-r3/child-processes.jsonl'
            _ = path.write_text('\n'.join(path.read_text().splitlines()[:-1]))
            with self.assertRaises(GateContractError): _ = audit(case)

    def test_loss_in_one_child_cannot_hide_behind_numerical_pass(self) -> None:
        with scenario() as case:
            path = case.root / 'candidate-r3/gate_observations/n200_r1.json.safety.json'
            data = document(path.read_text()).mapping()
            Node(data['observation']).mapping()['knowledge_lost'] = [{'tick': 100, 'node': 'clay_digging', 'holder': 9}]
            put(path, data)
            with self.assertRaisesRegex(GateContractError, 'KnowledgeLost'): _ = audit(case)

    def test_exact_report_bytes_do_not_normalize_line_endings(self) -> None:
        with scenario() as case:
            raw = case.root / 'candidate-r1/runs/n200_r1.json'
            pretty = json.dumps(document(raw.read_text()).mapping(), indent=2)
            _ = raw.write_bytes(pretty.replace('\n', '\r\n').encode())
            sidecar = case.root / 'candidate-r1/gate_observations/n200_r1.json.safety.json'
            envelope = document(sidecar.read_text()).mapping()
            envelope['raw_report_bytes'] = pretty
            put(sidecar, envelope)
            with self.assertRaisesRegex(GateContractError, 'raw_report_bytes'):
                _ = audit(case)

    def test_mutation_after_numerical_audit_cannot_replace_original_digest(self) -> None:
        with scenario() as case:
            def alter_after_numeric(root: Path, base_sha: str, candidate_sha: str,
                                    justifications: tuple[dict[str, str], ...] = (),
                                    changelog: str = '') -> dict[str, Json]:
                result = compare_rounds(root, base_sha, candidate_sha, justifications, changelog)
                raw = case.root / 'candidate-r1/runs/n200_r1.json'
                report = document(raw.read_text()).mapping()
                report['tick_mean_us'] = 100000
                put(raw, report)
                sidecar = case.root / 'candidate-r1/gate_observations/n200_r1.json.safety.json'
                envelope = document(sidecar.read_text()).mapping()
                envelope['raw_report_bytes'] = raw.read_text()
                put(sidecar, envelope)
                return result
            with patch('generic_bench_safety.compare_rounds', side_effect=alter_after_numeric), \
                    self.assertRaisesRegex(GateContractError, 'previously audited'):
                _ = audit(case)

    def test_actual_aggregate_result_and_receipt_hash_required(self) -> None:
        for mode in ('pid', 'exit', 'hash', 'missing', 'input', 'argv', 'source'):
            with self.subTest(mode=mode), scenario() as case:
                items = list(case.aggregates)
                last = items[-1]
                if mode == 'pid': items[-1] = replace(last, result=replace(last.result, pid=7))
                elif mode == 'exit': items[-1] = replace(last, result=replace(last.result, native_exit=1))
                elif mode == 'hash': items[-1] = replace(last, receipt_sha256='0' * 64)
                elif mode == 'missing': _ = items.pop()
                elif mode == 'argv': items[-1] = replace(last, process=replace(last.process, argv=last.process.argv + ('--extra',)))
                elif mode == 'source': items[-1] = replace(last, product=replace(last.product, source_sha='f' * 40))
                else: _ = (last.product.root / 'rules.ron').write_text('changed')
                with self.assertRaises(GateContractError): _ = audit(replace(case, aggregates=tuple(items)))


if __name__ == '__main__':
    _ = unittest.main()
