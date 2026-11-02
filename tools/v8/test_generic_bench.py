"""Bench boundaries use portable synthetic receipts, never machine-local fixtures."""
import unittest
from typing import final

from generic_bench import AuditError, justified_ratio


class RatioTests(unittest.TestCase):
    def test_normal_limit(self):
        self.assertEqual(justified_ratio(100, 110, 'n200', 'mean', (), ''), 1.1)

    def test_no_justification_rejects_remainder(self):
        with self.assertRaises(AuditError):
            justified_ratio(100, 111, 'n200', 'mean', (), '')

    def test_exact_justification_and_hard_limit(self):
        entry = {'scene': 'n200', 'metric': 'mean', 'quote': 'n200 mean: extra diagnosis work'}
        self.assertEqual(justified_ratio(100, 115, 'n200', 'mean', (entry,), entry['quote']), 1.15)
        with self.assertRaises(AuditError):
            justified_ratio(100, 115.01, 'n200', 'mean', (entry,), entry['quote'])

    def test_missing_or_wrong_quote_fails(self):
        entry = {'scene': 'n200', 'metric': 'mean', 'quote': 'n200 mean: explanation'}
        with self.assertRaises(AuditError):
            justified_ratio(100, 111, 'n200', 'mean', (entry,), '')
        with self.assertRaises(AuditError):
            justified_ratio(100, 111, 'n200', 'p99', (entry,), entry['quote'])

import json
import tempfile
from collections.abc import Callable
from pathlib import Path

from generic_bench import RSS_SCENES, SCENES, WORKLOADS, compare_rounds, round_values
from generic_gate import document
from mux2_gate_inputs import Json, Node

SHA = 'a' * 40


def synthetic_round(root: Path, time_start: int = 100) -> None:
    runs = root / 'runs'
    runs.mkdir(parents=True)
    rows: list[Json] = []
    raws: dict[str, dict[str, Json]] = {}
    for scene in SCENES:
        agents, side, scripted, wolves = WORKLOADS[scene]
        rss = 10000 + agents + side * side / 64
        alloc = 100000 + agents * 1000 + side * side * 20
        raw: dict[str, Json] = {'label': scene, 'agents': agents, 'side': side, 'script': scripted, 'wolves': wolves,
               'seed': 15100, 'warmup_ticks': 1440, 'measured_ticks': 2880, 'build_profile': 'release',
               'tick_mean_us': 100, 'tick_p99_us': 200, 'tick_max_us': 300,
               'load_before': '1 2 3', 'load_after': '2 3 4', 'alive_end': agents, 'fingerprint': 123,
               'rss_kib': {'before_build': 1000, 'after_build': rss, 'after_run': rss + 10},
               'alloc_bytes': {'after_build': alloc}}
        (runs / f'{scene}_r1.json').write_text(json.dumps(raw))
        raws[scene] = raw
        row = {key: raw[key] for key in ('label', 'agents', 'side', 'script', 'tick_mean_us', 'tick_p99_us', 'tick_max_us')}
        row['repeats'] = 1
        for kind in ('rss_kib', 'alloc_bytes'):
            for field, value in Node(raw[kind]).mapping().items():
                row[kind + '_' + field] = value
        row['per_repeat'] = [{key: raw[value] for key, value in
            (('mean', 'tick_mean_us'), ('p99', 'tick_p99_us'), ('max', 'tick_max_us'),
             ('load_before', 'load_before'), ('load_after', 'load_after'),
             ('alive_end', 'alive_end'), ('fingerprint', 'fingerprint'))}]
        rows.append(row)
    expected = [('timed', scene, 1, f'{scene}_r1.json') for scene in SCENES]
    for repeat in range(1, 6):
        for scene in RSS_SCENES:
            agents, side, _, _ = WORKLOADS[scene]
            raw = {'label': scene, 'agents': agents, 'side': side, 'repeat': repeat, 'build_only': True,
                   'load_before': '1 2 3', 'rss_kib': {'after_build': Node(raws[scene]['rss_kib']).field('after_build').value}}
            filename = f'rss_{scene}_r{repeat}.json'
            (runs / filename).write_text(json.dumps(raw))
            expected.append(('rss', scene, repeat, filename))
    report = {'commit': SHA, 'seed': 15100, 'warmup_ticks': 1440, 'measured_ticks': 2880,
              'scenes': rows, 'alloc_per_agent': 1000, 'alloc_per_tile': 20,
              'bytes_per_agent': 1024, 'bytes_per_tile': 16, 'memory_ceiling_source': 'alloc',
              'rss_outer': {'repeats': 5, 'per_agent': [1024] * 5, 'per_tile': [16] * 5,
                            'per_agent_median': 1024, 'per_tile_median': 16, 'agent_bound': 5120, 'tile_bound': 128},
              'ceilings': {key: 'PASS' for key in ('n1000_mean_le_5ms', 'n2000_mean_le_10ms',
                  'bytes_per_agent_le_4096', 'bytes_per_tile_le_96', 'rss_per_agent_le_5120', 'rss_per_tile_le_128')}}
    (root / 'bench.json').write_text(json.dumps(report))
    journal = [{'schema_version': 1, 'kind': kind, 'scene': scene, 'repeat': repeat,
                'output': str(runs / filename), 'load_raw': '1 2 3', 'load_1_5_15': [1, 2, 3],
                'unix_seconds': time_start + i, 'waited_seconds': 0, 'decision': 'ADMIT'}
               for i, (kind, scene, repeat, filename) in enumerate(expected)]
    (root / 'admission.jsonl').write_text(''.join(json.dumps(row) + '\n' for row in journal))


class RawAuditTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name).resolve()
        synthetic_round(self.root)

    def mutate(self, filename: str, function: Callable[[dict[str, Json]], None]) -> None:
        path = self.root / filename
        data = document(path.read_text()).mapping()
        function(data)
        path.write_text(json.dumps(data))

    def audit(self):
        return round_values(self.root, SHA, {}, require_ceilings=True)

    def test_complete_raw_round(self):
        self.assertEqual(self.audit()['child_count'], 21)

    def test_raw_report_disagreement(self):
        self.mutate('runs/n200_r1.json', lambda row: row.update(tick_mean_us=101))
        with self.assertRaisesRegex(AuditError, 'mismatch'):
            self.audit()

    def test_missing_child(self):
        (self.root / 'runs/rss_n200_r5.json').unlink()
        with self.assertRaises(AuditError):
            self.audit()

    def test_extra_child(self):
        (self.root / 'runs/extra.json').write_text('{}')
        with self.assertRaisesRegex(AuditError, 'coverage'):
            self.audit()

    def test_child_start_load_boundary(self):
        self.mutate('runs/n200_r1.json', lambda row: row.update(load_before='8 1 1'))
        with self.assertRaisesRegex(AuditError, 'load'):
            self.audit()

    def test_missing_admission(self):
        path = self.root / 'admission.jsonl'
        path.write_text('\n'.join(path.read_text().splitlines()[:-1]) + '\n')
        with self.assertRaisesRegex(AuditError, 'admission'):
            self.audit()

    def test_wrong_workload(self):
        self.mutate('runs/n200_r1.json', lambda row: row.update(measured_ticks=1))
        with self.assertRaisesRegex(AuditError, 'measured_ticks'):
            self.audit()

    def test_false_ceiling_claim(self):
        self.mutate('bench.json', lambda row: row.update(alloc_per_agent=1))
        with self.assertRaisesRegex(AuditError, 'alloc_per_agent'):
            self.audit()

    def test_rss_raw_changes_recomputed(self):
        self.mutate('runs/rss_n2000_r1.json', lambda row: Node(row['rss_kib']).mapping().update(after_build=99999))
        with self.assertRaisesRegex(AuditError, 'rss_outer'):
            self.audit()

    def test_consistent_ceiling_failure(self):
        self.mutate('runs/n1000_r1.json', lambda row: row.update(tick_mean_us=6000))
        def change(row: dict[str, Json]) -> None:
            scene = Node(row['scenes']).array()[1]
            scene.mapping()['tick_mean_us'] = 6000
            scene.field('per_repeat').array()[0].mapping()['mean'] = 6000
            Node(row['ceilings']).mapping()['n1000_mean_le_5ms'] = 'MISS'
        self.mutate('bench.json', change)
        with self.assertRaises((ValueError, AuditError)):
            self.audit()


class ComparisonTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name).resolve()
        for repeat in range(1, 4):
            for index, arm in enumerate(('base', 'candidate')):
                synthetic_round(self.root / f'{arm}-r{repeat}', repeat * 100 + index * 30)

    def test_self_comparison_has_126_independent_children(self):
        result = compare_rounds(self.root, SHA, SHA)
        self.assertEqual(result['raw_children'], 126)
        self.assertEqual(len(Node(result['hashes']).mapping()), 138)

    def test_missing_round_is_not_self_proof(self):
        import shutil
        shutil.rmtree(self.root / 'base-r3')
        with self.assertRaisesRegex(AuditError, 'coverage'):
            compare_rounds(self.root, SHA, SHA)

    def test_round_order_cannot_overlap(self):
        path = self.root / 'candidate-r1/admission.jsonl'
        rows = [json.loads(line) for line in path.read_text().splitlines()]
        for row in rows:
            row['unix_seconds'] -= 30
        path.write_text(''.join(json.dumps(row) + '\n' for row in rows))
        with self.assertRaisesRegex(AuditError, 'overlap'):
            compare_rounds(self.root, SHA, SHA)

import subprocess

from generic_bench import Product, parse, sha256, validate_products


class JsonBoundaryTests(unittest.TestCase):
    def test_nested_nonfinite_is_rejected(self):
        for text in ('{"unknown":[1e999]}', '{"x":NaN}', '{"x":1,"x":2}'):
            with self.subTest(text=text), self.assertRaises(AuditError):
                parse(text, 'fixture')

    def test_ratio_rejects_boolean_and_nonpositive(self):
        for base in (True, 0, -1, float('nan')):
            with self.subTest(base=base), self.assertRaises(AuditError):
                justified_ratio(base, 1, 'n200', 'mean', (), '')


class ProductFixture(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name).resolve()
        self.products: list[Product] = []
        for name in ('base', 'candidate'):
            root = self.root / name
            root.mkdir()
            subprocess.run(['git', 'init', '-q', str(root)], check=True)
            (root / 'CHANGELOG.md').write_text('test product')
            subprocess.run(['git', '-C', str(root), 'add', 'CHANGELOG.md'], check=True)
            subprocess.run(['git', '-C', str(root), '-c', 'user.name=Test', '-c', 'user.email=test@example.invalid',
                            'commit', '-qm', 'fixture'], check=True)
            sha = subprocess.check_output(['git', '-C', str(root), 'rev-parse', 'HEAD'], text=True).strip()
            binary = root / 'binary'
            binary.write_bytes(b'#!/bin/sh\necho "synthetic producer failure" >&2\nexit 73\n')
            binary.chmod(0o755)
            receipt = self.root / (name + '-build.json')
            receipt.write_text(json.dumps({'source_sha': sha, 'binary_sha256': sha256(binary),
                                          'profile': 'release', 'exit_code': 0}))
            self.products.append(Product(root, binary, sha, sha256(binary), receipt, sha256(receipt), (('CHANGELOG.md', sha256(root / 'CHANGELOG.md')),)))


class ProductBindingTests(ProductFixture):
    def test_distinct_roots_accepted(self):
        validate_products(self.products[0], self.products[1], 'publication')

    def test_same_root_rejected(self):
        with self.assertRaisesRegex(AuditError, 'independent'):
            validate_products(self.products[0], self.products[0], 'self_proof')

    def test_binary_drift_rejected(self):
        self.products[1].binary.write_bytes(b'changed')
        with self.assertRaisesRegex(AuditError, 'binary changed'):
            validate_products(self.products[0], self.products[1], 'publication')

    def test_build_receipt_drift_rejected(self):
        self.products[1].build_receipt.write_text('{}')
        with self.assertRaisesRegex(AuditError, 'receipt changed'):
            validate_products(self.products[0], self.products[1], 'publication')

from unittest.mock import patch

from generic_bench_run import run_bench


class ExecutionFailureTests(ProductFixture):

    def test_nonzero_child_retains_failure_and_logs(self):
        script = self.root / 'commands.json'
        script.write_text('[]')
        output = self.root / 'evidence'
        with patch('generic_bench_run.wait_for_load', return_value=(1.0, 2.0, 3.0)), \
             self.assertRaisesRegex(AuditError, 'producer status FAIL, exit 73'):
            run_bench(self.products[0], self.products[1], 'publication', script, sha256(script), output)
        record = json.loads((output / 'base-r1.finished.json').read_text())
        self.assertEqual(record['exit_code'], 73)
        self.assertGreater(record['producer_pid'], 0)
        self.assertEqual(record['process_receipt_sha256'], sha256(output / 'base-r1/process.json'))
        self.assertEqual((output / 'base-r1/stderr.log').read_text(), 'synthetic producer failure\n')
        self.assertTrue((output / 'failure.json').is_file())
        self.assertFalse((output / 'result.json').exists())

    def test_existing_output_is_never_reused(self):
        script = self.root / 'commands.json'
        script.write_text('[]')
        output = self.root / 'evidence'
        output.mkdir()
        with self.assertRaises(FileExistsError):
            run_bench(self.products[0], self.products[1], 'publication', script, sha256(script), output)


class BenchmarkSafetyWiringTests(ProductFixture):
    """Real tiny aggregate children; mocked safety oracle is not benchmark evidence."""

    def test_six_actual_aggregates_must_reach_safety_oracle(self):
        from dataclasses import replace

        from generic_gate import GateContractError

        products = []
        for product in self.products:
            product.binary.write_bytes(b'#!/bin/sh\nexit 0\n')
            receipt = json.loads(product.build_receipt.read_text())
            receipt['binary_sha256'] = sha256(product.binary)
            product.build_receipt.write_text(json.dumps(receipt))
            products.append(replace(product, binary_sha256=sha256(product.binary),
                                    build_receipt_sha256=sha256(product.build_receipt)))
        script=self.root/'commands.json';script.write_text('[]')
        output=self.root/'safety-required'
        rejected=GateContractError('TEST_REJECT','/safety','synthetic safety rejection')
        with patch('generic_bench_run.wait_for_load',return_value=(1.0,2.0,3.0)), \
             patch('generic_bench_run.audit_bench_safety',side_effect=rejected) as audit, \
             self.assertRaisesRegex(GateContractError,'synthetic safety rejection'):
            run_bench(products[0],products[1],'publication',script,sha256(script),output)
        aggregates=audit.call_args.args[1]
        self.assertEqual([(a.arm,a.round_index) for a in aggregates],
                         [('base',1),('candidate',1),('base',2),('candidate',2),('base',3),('candidate',3)])
        self.assertEqual(len({a.result.pid for a in aggregates}),6)
        for item in aggregates:
            self.assertEqual(item.result.native_exit,0)
            self.assertEqual(item.receipt_sha256,sha256(item.result.receipt))
            self.assertEqual(item.process.external_inputs,((script,sha256(script)),))
        self.assertTrue((output/'failure.json').is_file())
        self.assertFalse((output/'result.json').exists())


@final
class RetainedBenchTests(ProductFixture):
    """Tiny processes plus mocked safety: receipt plumbing, never benchmark proof."""

    def prepare(self) -> tuple[Path, Path]:
        from dataclasses import replace
        for index, product in enumerate(self.products):
            _ = product.binary.write_bytes(b'#!/bin/sh\nexit 0\n')
            row = document(product.build_receipt.read_text()).mapping()
            row['binary_sha256'] = sha256(product.binary)
            _ = product.build_receipt.write_text(json.dumps(row))
            self.products[index] = replace(product, binary_sha256=sha256(product.binary),
                                           build_receipt_sha256=sha256(product.build_receipt))
        script = self.root/'commands.json'; _ = script.write_text('[]')
        output = self.root/'retained'
        with patch('generic_bench_run.wait_for_load', return_value=(1.0,2.0,3.0)), \
             patch('generic_bench_run.audit_bench_safety', side_effect=self.audit):
            _ = run_bench(self.products[0],self.products[1], 'publication', script, sha256(script), output)
        return script, output

    @staticmethod
    def audit(*_args: object) -> dict[str, Json]:
        return {'status':'BENCH_NUMERICAL_AND_SAFETY_PASS','publication_authority':False,
                'worlds':126,'pairs':63,'hashes':{},'numerical':{},'safety':[],
                'authority_dependency':'synthetic test oracle only'}

    def authenticate(self, script: Path, output: Path):
        import generic_bench_run as runner
        with patch.object(runner,'audit_bench_safety',side_effect=self.audit):
            return runner.authenticate_result(self.products[0],self.products[1],'publication',script,sha256(script),output,
                receipt_sha256=sha256(output/'result.json'))

    def test_actual_six_retained_processes_reaudited(self):
        script,output=self.prepare()
        result=self.authenticate(script,output)
        self.assertEqual(result.status,'COMPONENT_PASS')
        self.assertEqual(result.receipt,output/'result.json')

    def test_completed_process_lifecycle_cannot_be_resealed_away(self):
        for key,value in (('duration_ns',0),('duration_ns',True),('ended_at','2020-01-01T00:00:00+00:00')):
            with self.subTest(key=key,value=value):
                script,output=self.prepare()
                process=output/'base-r1/process.json'
                row=document(process.read_text()).mapping();row[key]=value;_ = process.write_text(json.dumps(row))
                self.reseal(output)
                result=self.authenticate(script,output)
                self.assertEqual(result.status,'FAIL')
                # Fresh evidence path for next synthetic subtest; never delete prior evidence.
                self.root=self.root/'next';self.root.mkdir()

    def test_finished_and_result_disagreement_rejected(self):
        script,output=self.prepare()
        finished=output/'base-r1.finished.json'
        row=document(finished.read_text()).mapping();row['producer_pid']=1;_ = finished.write_text(json.dumps(row))
        self.assertEqual(self.authenticate(script,output).status,'FAIL')

    def test_result_and_failure_cannot_coexist(self):
        script,output=self.prepare();_ = (output/'failure.json').write_text('{}')
        self.assertEqual(self.authenticate(script,output).status,'FAIL')

    def test_no_terminal_is_pending_without_manufactured_file(self):
        import generic_bench_run as runner
        output=self.root/'pending';output.mkdir()
        script=self.root/'commands.json';_ = script.write_text('[]')
        result=runner.authenticate_result(self.products[0],self.products[1],'publication',script,sha256(script),output,receipt_sha256=None)
        self.assertEqual(result.status,'PENDING');self.assertIsNone(result.receipt)
        self.assertEqual(list(output.iterdir()),[])

    def test_original_failure_retained_without_success_shape(self):
        import generic_bench_run as runner
        script=self.root/'commands.json';_ = script.write_text('[]');output=self.root/'failed'
        with patch.object(runner,'wait_for_load',return_value=(1.0,2.0,3.0)),self.assertRaises(AuditError):
            _ = run_bench(self.products[0],self.products[1],'publication',script,sha256(script),output)
        result=runner.authenticate_result(self.products[0],self.products[1],'publication',script,sha256(script),output,
            receipt_sha256=sha256(output/'failure.json'))
        self.assertEqual(result.status,'FAIL');self.assertEqual(result.receipt,output/'failure.json')
        self.assertFalse((output/'result.json').exists())

    @staticmethod
    def reseal(output: Path, label: str = 'base-r1') -> None:
        process=output/label/'process.json'
        finished=output/(label+'.finished.json')
        row=document(finished.read_text()).mapping();row['process_receipt_sha256']=sha256(process)
        _ = finished.write_text(json.dumps(row))
        terminal=output/'result.json';result=document(terminal.read_text()).mapping()
        records=Node(result['invocations']).array()
        result['invocations']=[row if item.field('arm').text()+'-r'+str(item.field('repeat').value)==label else item.value for item in records]
        _ = terminal.write_text(json.dumps(result))

    def test_resealed_spawn_and_start_drift_reject(self):
        cases: tuple[tuple[str,str,Json], ...] = (('process-spawn.json','pid',0),('process-spawn.json','spawned_at','2099-01-01T00:00:00+00:00'),
                               ('process-start.json','expected_inputs',{}))
        for name,key,value in cases:
            script,output=self.prepare();path=output/'base-r1'/name
            row=document(path.read_text()).mapping();row[key]=value;_ = path.write_text(json.dumps(row))
            self.assertEqual(self.authenticate(script,output).status,'FAIL')
            self.root=self.root/'next';self.root.mkdir()

    def test_extra_file_or_empty_directory_rejected(self):
        for directory in (False,True):
            script,output=self.prepare();path=output/'extra'
            if directory:path.mkdir()
            else:_ = path.write_text('unowned')
            self.assertEqual(self.authenticate(script,output).status,'FAIL')
            self.root=self.root/'next';self.root.mkdir()

    def test_late_mutation_rejected_by_full_snapshot(self):
        import generic_bench_run as runner
        script,output=self.prepare()
        def mutate(*args: object) -> dict[str,Json]:
            _ = (output/'base-r1/process-start.json').write_text('{}')
            return self.audit(*args)
        with patch.object(runner,'audit_bench_safety',side_effect=mutate):
            result=runner.authenticate_result(self.products[0],self.products[1],'publication',script,sha256(script),output,
                receipt_sha256=sha256(output/'result.json'))
        self.assertEqual(result.status,'FAIL');self.assertTrue(any('final:' in error for error in result.errors))

    def test_real_safety_rejection_is_not_numeric_success(self):
        import generic_bench_run as runner
        script,output=self.prepare()
        # Tiny aggregate logs have no bench children; real audit must reject them.
        result=runner.authenticate_result(self.products[0],self.products[1],'publication',script,sha256(script),output,
            receipt_sha256=sha256(output/'result.json'))
        self.assertEqual(result.status,'FAIL')
