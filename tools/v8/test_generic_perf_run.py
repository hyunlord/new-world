"""Offline contract tests; no native performance or publication claims."""
import json
import tempfile
import unittest
from contextlib import nullcontext
from dataclasses import replace
from pathlib import Path
from typing import final
from unittest.mock import patch

import generic_perf_run
from core_p7_admission import AdmissionError, admit
from generic_bench import AuditError, Product
from generic_gate import document, typed_json
from generic_perf_observations import PerfObservation
from generic_perf_run import plans, run_pair
from generic_process import ProcessResult, ProcessSpec, environment_for, file_hash
from mux2_gate_inputs import Json, Node
from test_generic_observations import receipt


def product(root: Path) -> Product:
    return Product(root, root / 'worldsim_perf', 'a' * 40, 'b' * 64,
                   root / 'build.json', 'c' * 64, (('content.ron', 'd' * 64),))


class PlansTests(unittest.TestCase):
    def test_authenticator_is_source_owned(self) -> None:
        self.assertTrue(callable(getattr(generic_perf_run, "authenticate_result", None)))

    def test_standing_perf_matrix_and_explicit_companion(self) -> None:
        root = Path('/synthetic')
        for arm in ('base', 'candidate'):
            rows = plans('perf.natural_quick', product(root / arm), root / 'out' / arm, arm)
            self.assertEqual(len(rows), 14)
            self.assertEqual([(r.seed, r.ticks) for r in rows[:12]],
                             [(s, 172800) for _ in range(3) for s in (19600,19603,19605,19606)])
            self.assertEqual(rows[0].argv[1:7], ('natural','--scene','game_v26','--seeds','19600..19601','--days'))
            self.assertEqual(rows[12].argv[1:2], ('quick',))
            self.assertEqual(rows[13].argv[1:-2], ('stress','--seeds','15100..15101','--agents','500',
                                                   '--side','64','--days','1','--feed','fed','--rules','game_v26'))
            self.assertEqual([r.ticks for r in rows[12:]], [1440,1440])
            self.assertEqual(rows[13].companion, arm == 'candidate')

    def test_nav_has_explicit_actual_script_and_full_horizon(self) -> None:
        root = Path('/synthetic')
        row, = plans('nav.soak', product(root), root / 'out', 'base')
        self.assertEqual(row.argv[1:], ('nav-bench','--out',str(root/'out/raw/nav_bench.json'),
                                       '--commands',str(root/'tools/v8/soak_script.json')))
        self.assertEqual((row.seed,row.ticks,row.agents), (15100,4320,1000))

    def test_unsupported_contract_rejected(self) -> None:
        with self.assertRaises(ValueError):
            _ = plans('bench.other', product(Path('/synthetic')), Path('/out'), 'base')


class SyntheticRunTests(unittest.TestCase):
    def exercise(self, fixture: str, defect: str = '', *, authenticate: bool=False, mutation: str='') -> Node:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            products = [product(root / arm) for arm in ('base','candidate')]
            if authenticate:
                products=[replace(p,input_files=((*p.input_files,('tools/v8/soak_script.json','e'*64)))) for p in products]
            output = root / 'out'
            matrix = {arm: plans(fixture, prod, output / arm, arm)
                      for arm,prod in zip(('base','candidate'),products)}
            lookup = {run.argv: run for rows in matrix.values() for run in rows}
            calls: list[str] = []
            processes: list[Path] = []

            def child(spec: ProcessSpec) -> ProcessResult:
                run = lookup[spec.argv]
                calls.append(run.name)
                spec.output_root.mkdir()
                process = spec.output_root / 'process.json'
                _ = process.write_text('{}')
                processes.append(process)
                if len(processes) == 2 and defect in {'receipt_rewrite', 'stdout_rewrite'}:
                    victim = processes[0] if defect == 'receipt_rewrite' else processes[0].parent/'stdout.log'
                    _ = victim.write_text('rewritten')
                if defect == 'exit':
                    return ProcessResult(process, 'FAIL', 2, 42)
                raw = Path(run.argv[3]) if run.kind == 'nav' else run.raw / (run.name + '.json')
                _ = (spec.output_root/'stdout.log').write_text('bad' if defect == 'stdout' else 'nav_bench: synthetic')
                native = receipt()
                for key in ('scene','seed','occurrence','checkpoints'):
                    del native[key]
                native.update(requested_ticks=run.ticks, requested_terminal_tick=run.ticks,
                              terminal_tick=run.ticks, clock_offset_ticks=480, ticks_per_day=1440)
                report = document(run.workload).mapping()
                report.update(build_profile='release',seed=run.seed,alive_end=8)
                if run.kind == 'perf':
                    report.update(git_sha='a'*40,contended=False,agents_requested=run.agents,
                                  founders=10,ticks=run.ticks,ticks_per_day=1440,clock_offset_ticks=480,
                                  warmup={'ticks':60},tick_summary={'ticks':run.ticks-60,'mean_ms':1.,
                                  'p99_ms':2.,'p999_ms':3.,'max_ms':4.},
                                  steady_tick_ms_total=float(run.ticks-60),fingerprint='same')
                    _ = raw.with_suffix('.tick_us.bin').write_bytes(bytes(run.ticks*4))
                    index = run.raw/'runs.json'
                    entries: list[Json] = [node.value for node in document(index.read_text()).field('runs').array()] if index.exists() else []
                    if defect == 'index_rewrite' and entries:
                        entry = Node(entries[0]).mapping()
                        entry['seed'] = -1
                    entries.append({'file':raw.name,'seed':run.seed,'status':'ok'})
                    _ = index.write_text(json.dumps({'runs':entries}))
                else:
                    report.update(agents=1000,warmup_ticks=1440,measured_ticks=2880,
                                  movement_us=3.,budget_us=2.,within_budget=False)
                    native.update(warmup_alive=9,interventions={'gifts':1,'gift_requested':1,'gift_applied':1})
                if defect == 'horizon':
                    native['terminal_tick'] = run.ticks - 1
                if defect == 'loss' and 'candidate' in raw.parts:
                    native['knowledge_lost'] = [{'tick':100,'node':'clay_digging','holder':9}]
                text = json.dumps(report, indent=2)
                _ = raw.write_bytes(text.replace('\n', '\r\n').encode() if defect == 'crlf' else text.encode())
                side = run.raw/'gate_observations'
                side.mkdir(exist_ok=True)
                envelope = {'schema_version':1,'producer_pid':43 if defect == 'pid' else 42,
                            'raw_report_path':str(raw),'raw_report_bytes':text,'observation':native}
                _ = (side/(raw.name+'.safety.json')).write_text(json.dumps(envelope))
                if defect == 'extra':
                    _ = (side/'extra.safety.json').write_text('{}')
                if defect == 'extra_raw' and len(calls) == 5:
                    _ = (output/'base/raw/natural_r1/unregistered.json').write_text(text)
                if defect == 'symlink':
                    (spec.output_root/'forbidden').symlink_to(raw)
                if defect == 'admission_rewrite':
                    _ = (output/'admission.jsonl').write_text('rewritten')
                if authenticate:synthetic_process(spec,42)
                return ProcessResult(process, 'PASS', 0, 42)

            def admission(log: Path, label: str, *, deadline: float) -> tuple[float,float,float]:
                if defect in {'late_admission', 'failed_admission_rewrite'}:
                    if defect == 'failed_admission_rewrite' and calls:
                        _ = log.write_text('rewritten\n')
                    return admit(log, label, deadline=deadline, sample=lambda: (1.,1.,1.),
                                 clock=lambda: 50. if not calls else 100.)
                if defect=='bad_sample' and calls:
                    return admit(log,label,deadline=deadline,sample=lambda:(-1.,1.,1.),clock=lambda:50.)
                if defect == 'admission':
                    raise AdmissionError('load not admitted')
                with log.open('a') as stream:
                    _ = stream.write(json.dumps({'label':label,'utc':50.,'window':0,'loads_1_5_15':[1.,1.,1.],'status':'ADMITTED'} if authenticate else {'label':label, 'deadline':deadline})+'\n')
                return (1.,1.,1.)

            with patch('generic_perf_run.prepare', return_value=matrix), \
                 patch('generic_perf_run.lane', return_value=nullcontext()), \
                 patch('generic_perf_run.admit', side_effect=admission), \
                 patch('generic_perf_run.verify_product', side_effect=AuditError('postflight changed') if defect=='postflight' else None), \
                 patch('generic_perf_run.validate_products'), \
                 patch('generic_perf_run.inventory', side_effect=synthetic_inventory), \
                 patch('generic_perf_run.execute', side_effect=child):
                target = run_pair(fixture, products[0], products[1], output, deadline=100.)
                if authenticate:
                    if mutation and mutation!='late_log':mutate(output,mutation)
                    original_audit=generic_perf_run.audit
                    audit_count=0
                    def checked_audit(run: generic_perf_run.Run, prod: Product, pid: int, raw: Path) -> tuple[PerfObservation,dict[Path,str]]:
                        nonlocal audit_count
                        result=original_audit(run,prod,pid,raw);audit_count+=1
                        if mutation=='late_log' and audit_count==len(lookup):
                            _=(processes[0].parent/'stdout.log').write_text('changed after checked')
                        return result
                    with patch('generic_perf_run.audit',side_effect=checked_audit):
                        result=generic_perf_run.authenticate_result(fixture,products[0],products[1],output,
                            receipt_sha256=file_hash(target),deadline=100.)
                    self.assertEqual(result.status,document(target.read_text()).field('status').text())
            result = document(target.read_text())
            result.field('publication_authorized').equals(False)
            self.assertIn('output_sha256', result.mapping())
            if not defect:
                self.assertEqual(len(calls), 28 if fixture == 'perf.natural_quick' else 2)
            return result

    def test_perf_full_matrix_and_report_only_original_reducer(self) -> None:
        result = self.exercise('perf.natural_quick')
        self.assertEqual(result.field('status').text(), 'COMPONENT_PASS', result.field('errors').array())
        self.assertEqual(len(result.field('safety').array()), 14)
        self.assertEqual(sum(row.field('companion').value is True for row in result.field('worlds').array()), 1)
        self.assertIn('quick (baseline stress)', result.field('standing_timing_comparison').text())

    def test_nav_over_budget_remains_numeric_report_only(self) -> None:
        result = self.exercise('nav.soak')
        self.assertEqual(result.field('status').text(), 'COMPONENT_PASS', result.field('errors').array())
        self.assertEqual(len(result.field('safety').array()), 1)

    def test_bad_native_evidence_never_passes(self) -> None:
        for defect in ('exit','pid','horizon','loss','extra','stdout','crlf'):
            with self.subTest(defect=defect):
                result = self.exercise('nav.soak', defect)
                self.assertEqual(result.field('status').text(), 'FAIL', result)
                self.assertTrue(result.field('errors').array())

    def test_later_children_cannot_rewrite_completed_evidence(self) -> None:
        for defect in ('receipt_rewrite','stdout_rewrite','admission_rewrite'):
            with self.subTest(defect=defect):
                result = self.exercise('nav.soak', defect)
                self.assertEqual(result.field('status').text(), 'FAIL')

    def test_shared_index_and_reducer_coverage_are_exact(self) -> None:
        for defect in ('index_rewrite','extra_raw'):
            with self.subTest(defect=defect):
                result = self.exercise('perf.natural_quick', defect)
                self.assertEqual(result.field('status').text(), 'FAIL')

    def test_postflight_and_forbidden_output_retain_failure(self) -> None:
        for defect in ('postflight','symlink'):
            with self.subTest(defect=defect):
                result = self.exercise('nav.soak', defect)
                self.assertEqual(result.field('status').text(), 'FAIL')
                self.assertTrue(result.field('errors').array())
                if defect == 'symlink':
                    result.field('output_inventory_complete').equals(False)
                else:
                    self.assertIn('postflight changed', str(result.field('errors').value))

    def test_failed_admission_cannot_rewrite_prior_journal(self) -> None:
        result = self.exercise('nav.soak','failed_admission_rewrite')
        result.field('status').equals('FAIL')
        self.assertIn('rewrote journal prefix', str(result.field('errors').value))

    def test_real_late_admission_preserves_pending_and_prior_world(self) -> None:
        result = self.exercise('nav.soak','late_admission')
        result.field('status').equals('PENDING')
        self.assertEqual(len(result.field('worlds').array()), 1)
        result.field('output_inventory_complete').equals(True)
        self.assertIn('owner hard stop reached', str(result.field('errors').value))

    def test_unadmitted_run_remains_pending(self) -> None:
        result = self.exercise('nav.soak','admission')
        self.assertEqual(result.field('status').text(), 'PENDING')
        self.assertEqual(result.field('worlds').array(), [])


def synthetic_inventory(_root: Path, names: list[str]) -> dict[str,str]:
    return {name:('e' if name.endswith('soak_script.json') else 'd')*64 for name in names}


def synthetic_process(spec: ProcessSpec, pid: int) -> None:
    """Native-shaped frozen receipts, explicitly synthetic process evidence."""
    start: dict[str,Json]={'schema_version':1,'argv':list(spec.argv),'cwd':str(spec.cwd),'executable':str(spec.executable),
        'expected_executable_sha256':spec.executable_sha256,'input_root':str(spec.input_root),'expected_inputs':dict(spec.inputs),
        'output_root':str(spec.output_root),'environment':{k:v for k,v in environment_for(spec).items()},'expected_external_inputs':{},
        'fresh_directories':[],'environment_policy':'fixed_defaults_plus_explicit_only_no_inheritance','build_provenance_verified':False,
        'before_executable_sha256':spec.executable_sha256,'before_inputs':dict(spec.inputs),'before_external_inputs':{},'after_external_inputs':{},
        'started_at':'1970-01-01T00:01:00+00:00','pid':None,'native_exit':None,'status':'FAIL','errors':[]}
    _=(spec.output_root/'process-start.json').write_text(json.dumps(start))
    _=(spec.output_root/'process-spawn.json').write_text(json.dumps({'pid':pid,'spawned_at':'1970-01-01T00:01:01+00:00'}))
    _=(spec.output_root/'stderr.log').write_text('')
    terminal=dict(start);terminal.update(pid=pid,native_exit=0,status='PASS',ended_at='1970-01-01T00:01:02+00:00',duration_ns=2_000_000_000,
        after_executable_sha256=spec.executable_sha256,after_inputs=dict(spec.inputs),stdout_sha256=file_hash(spec.output_root/'stdout.log'),
        stderr_sha256=file_hash(spec.output_root/'stderr.log'),limit='Endpoint executable/input binding only; no build, descendants, output schema, or whole-gate authority.')
    _=(spec.output_root/'process.json').write_text(json.dumps(terminal))


def mutate(output: Path, mutation: str) -> None:
    target=output/'result.json';report=document(target.read_text()).mapping()
    first=Node(report['worlds']).array()[0]
    process=Path(first.field('process').text())
    payload=document(process.read_text()).mapping()
    match mutation:
        case 'pid':payload['pid']=True
        case 'argv':payload['argv']=[]
        case 'duration_zero':payload['duration_ns']=0
        case 'duration_negative':payload['duration_ns']=-1
        case 'ended':payload['ended_at']='1970-01-01T00:00:01+00:00'
        case 'start':
            start=process.parent/'process-start.json';row=document(start.read_text()).mapping();row['pid']=42
            _=start.write_text(json.dumps(row))
        case 'safety':report['safety']=[]
        case 'source':first.mapping()['source_sha']='f'*40
        case 'terminal_extra':report['schema_version']=1
        case 'extra_process':_=(process.parent/'extra').write_text('extra')
        case 'omit_pending':
            report.update(status='PENDING',worlds=[],safety=[],errors=['load not admitted'])
            (output/'admission.jsonl').unlink()
        case 'extra_output':_=(output/'unregistered.txt').write_text('extra')
        case 'missing_admission':(output/'admission.jsonl').unlink()
        case 'admission':
            path=output/'admission.jsonl';rows=[document(line).mapping() for line in path.read_text().splitlines()];rows[0]['utc']=101.
            _=path.write_text(''.join(json.dumps(row)+'\n' for row in rows))
        case 'index':
            index=Path(first.field('raw').text()).parent/'runs.json';row=document(index.read_text()).mapping();row['runs']=[entry.value for entry in reversed(Node(row['runs']).array())]
            _=index.write_text(json.dumps(row))
        case _:raise AssertionError(mutation)
    _=process.write_text(json.dumps(payload))
    frozen=generic_perf_run.freeze(output);_=frozen.pop('result.json');report['output_sha256']=typed_json(frozen)
    _=target.write_text(json.dumps(report))


@final
class AuthenticationTests(unittest.TestCase):
    exercise=SyntheticRunTests.exercise
    def test_both_complete_producers_reauthenticate(self) -> None:
        for fixture in ('nav.soak','perf.natural_quick'):
            self.exercise(fixture,authenticate=True).field('status').equals('COMPONENT_PASS')

    def test_resealed_false_success_rejected(self) -> None:
        for mutation in ('pid','argv','duration_zero','duration_negative','ended','start','safety','source','terminal_extra','extra_process','admission','late_log','extra_output','missing_admission'):
            with self.subTest(mutation=mutation),self.assertRaises(ValueError):
                _=self.exercise('nav.soak',authenticate=True,mutation=mutation)
        with self.assertRaises(ValueError):_=self.exercise('perf.natural_quick',authenticate=True,mutation='index')

    def test_missing_terminal_is_not_an_authenticated_failure(self) -> None:
        with tempfile.TemporaryDirectory() as directory,patch('generic_perf_run.validate_products'):
            root=Path(directory).resolve()
            with self.assertRaises(OSError):
                _=generic_perf_run.authenticate_result('nav.soak',product(root/'base'),product(root/'candidate'),root,
                    receipt_sha256='a'*64,deadline=100.)

    def test_pending_cannot_omit_retained_completed_children(self) -> None:
        with self.assertRaises(ValueError):
            _=self.exercise('nav.soak',authenticate=True,mutation='omit_pending')

    def test_inconsistent_partial_pending_is_rejected(self) -> None:
        with self.assertRaises(ValueError):
            _=self.exercise('nav.soak','late_admission',authenticate=True,mutation='admission')

    def test_partial_pending_and_failure_retained(self) -> None:
        for defect,status in (('admission','PENDING'),('late_admission','PENDING'),('bad_sample','PENDING'),('exit','FAIL')):
            self.exercise('nav.soak',defect,authenticate=True).field('status').equals(status)


if __name__ == '__main__':
    _ = unittest.main()
