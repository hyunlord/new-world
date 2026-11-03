"""Runtime input plumbing with tiny processes and synthetic oracles, not perf proof."""
import sys
import unittest
from dataclasses import replace
from pathlib import Path
from typing import final, override
from unittest.mock import patch

import generic_bench as bench
import generic_bench_run as bench_run
import generic_perf_run as perf
import test_generic_bench as bench_tests
import test_generic_bench_safety as safety_tests
import test_generic_perf_run as perf_tests
from generic_gate import document
from generic_process import ProcessSpec, execute, file_hash
from mux2_gate_inputs import Json


def with_runtime(product: bench.Product) -> bench.Product:
    path=product.root/'rust/target/runtime.bin';path.parent.mkdir(parents=True,exist_ok=True)
    _=path.write_bytes(b'runtime dependency')
    return replace(product,runtime_inputs=(('rust/target/runtime.bin',file_hash(path)),))


@final
class RuntimeInputsTests(unittest.TestCase):
    def __init__(self, methodName: str='runTest') -> None:
        super().__init__(methodName)
        self.fixture=bench_tests.RetainedBenchTests()

    @override
    def setUp(self) -> None:
        self.fixture.setUp();self.addCleanup(self.fixture.doCleanups)

    def test_empty_default_preserves_original_inputs_and_matching_overlap(self) -> None:
        product=self.fixture.products[0]
        self.assertIs(bench.process_inputs(product),product.input_files)
        self.assertEqual(bench.process_inputs(replace(product,runtime_inputs=product.input_files)),product.input_files)

    def test_invalid_paths_duplicates_hashes_and_links_reject(self) -> None:
        product=with_runtime(self.fixture.products[0]);name,digest=product.runtime_inputs[0]
        directory=product.root/'directory';directory.mkdir()
        link=product.root/'link';link.symlink_to(product.root/name)
        parent=product.root/'linked';parent.symlink_to(product.root/'rust/target',target_is_directory=True)
        variants=(((name,digest),(name,digest)),((product.input_files[0][0],'0'*64),),
                  (('/absolute',digest),),(('../escape',digest),),(('rust//target/runtime.bin',digest),),
                  ((name,'wrong'),),(('directory',digest),),(('link',digest),),(('linked/runtime.bin',digest),))
        for rows in variants:
            with self.subTest(rows=rows),self.assertRaises((bench.AuditError,ValueError)):
                _=bench.process_inputs(replace(product,runtime_inputs=rows))
        _=(product.root/name).write_bytes(b'drift')
        with self.assertRaises(bench.AuditError):_=bench.process_inputs(product)

    def test_actual_bench_process_binds_source_and_runtime_and_rejects_drift(self) -> None:
        product=with_runtime(self.fixture.products[0]);script=self.fixture.root/'script.json';_=script.write_text('[]')
        output=self.fixture.root/'actual';output.mkdir()
        spec=bench_run.aggregate_spec(product,'base',1,script,file_hash(script),output)
        self.assertEqual(dict(spec.inputs),dict((*product.input_files,*product.runtime_inputs)))
        result=execute(spec)
        self.assertEqual(result.native_exit,73)
        row=document(result.receipt.read_text())
        row.field('expected_inputs').equals(dict(spec.inputs));row.field('before_inputs').equals(dict(spec.inputs))
        _=(product.root/product.runtime_inputs[0][0]).write_bytes(b'drift')
        with self.assertRaises(bench.AuditError):_=bench_run.aggregate_spec(product,'base',2,script,file_hash(script),output)
        self.assertFalse((output/'base-r2').exists())

    def test_bench_runtime_dependency_records_and_retained_authentication(self) -> None:
        self.fixture.products=[with_runtime(product) for product in self.fixture.products]
        script,output=self.fixture.prepare()
        self.assertEqual(self.fixture.authenticate(script,output).status,'COMPONENT_PASS')
        document((output/'base-r1.started.json').read_text()).field('runtime_inputs').equals(dict(self.fixture.products[0].runtime_inputs))
        self.fixture.products=[replace(product,runtime_inputs=()) for product in self.fixture.products]
        self.assertEqual(self.fixture.authenticate(script,output).status,'FAIL')

    def test_bench_late_runtime_drift_is_rejected(self) -> None:
        self.fixture.products=[with_runtime(product) for product in self.fixture.products]
        script,output=self.fixture.prepare();product=self.fixture.products[0]
        _=(product.root/product.runtime_inputs[0][0]).write_bytes(b'drift')
        self.assertEqual(self.fixture.authenticate(script,output).status,'FAIL')

    def test_bench_runtime_changed_during_final_oracle_rejects(self) -> None:
        self.fixture.products=[with_runtime(product) for product in self.fixture.products]
        script,output=self.fixture.prepare();product=self.fixture.products[0]
        def audit(*args: object) -> dict[str,Json]:
            value=self.fixture.audit(*args)
            _=(product.root/product.runtime_inputs[0][0]).write_bytes(b'late oracle drift')
            return value
        with patch.object(bench_run,'audit_bench_safety',side_effect=audit):
            result=bench_run.authenticate_result(self.fixture.products[0],self.fixture.products[1],'publication',script,file_hash(script),output,receipt_sha256=file_hash(output/'result.json'))
        self.assertEqual(result.status,'FAIL')

    def exercise_perf(self, *, omit: bool=False) -> None:
        original_product=perf_tests.product;original_process=perf_tests.synthetic_process;original_auth=perf.authenticate_result
        calls: list[ProcessSpec]=[]
        def product(root: Path) -> bench.Product:return with_runtime(original_product(root))
        def process(spec: ProcessSpec,pid: int) -> None:
            self.assertIn('rust/target/runtime.bin',dict(spec.inputs));calls.append(spec);original_process(spec,pid)
        def auth(fixture: str,base: bench.Product,candidate: bench.Product,output: Path,*,receipt_sha256: str,deadline: float,purpose: str='publication') -> perf.PerfResult:
            if omit:base=replace(base,runtime_inputs=());candidate=replace(candidate,runtime_inputs=())
            return original_auth(fixture,base,candidate,output,receipt_sha256=receipt_sha256,deadline=deadline,purpose=purpose)
        with patch.object(perf_tests,'product',side_effect=product),patch.object(perf_tests,'synthetic_process',side_effect=process),patch.object(perf,'authenticate_result',side_effect=auth):
            result=perf_tests.SyntheticRunTests().exercise('nav.soak',authenticate=True)
        self.assertEqual(len(calls),2)
        self.assertIn('runtime_inputs',result.mapping())

    def test_perf_actual_specs_and_runtime_terminal_bindings(self) -> None:
        # The synthetic exercise cleans its temporary root; inspect retained metadata inside auth.
        original=perf.authenticate_result
        def auth(fixture: str,base: bench.Product,candidate: bench.Product,output: Path,*,receipt_sha256: str,deadline: float,purpose: str='publication') -> perf.PerfResult:
            document((output/'result.json').read_text()).field('runtime_inputs').field('base').equals(dict(base.runtime_inputs))
            return original(fixture,base,candidate,output,receipt_sha256=receipt_sha256,deadline=deadline,purpose=purpose)
        original_product=perf_tests.product
        def product(root: Path) -> bench.Product:return with_runtime(original_product(root))
        with patch.object(perf_tests,'product',side_effect=product),patch.object(perf,'authenticate_result',side_effect=auth):
            result=perf_tests.SyntheticRunTests().exercise('nav.soak',authenticate=True)
        result.field('status').equals('COMPONENT_PASS')

    def test_perf_runtime_omission_cannot_authenticate_old_processes(self) -> None:
        with self.assertRaises(ValueError):self.exercise_perf(omit=True)

    def test_actual_child_runtime_write_retains_failed_process(self) -> None:
        product=with_runtime(self.fixture.products[0]);runtime=product.root/product.runtime_inputs[0][0]
        _=product.binary.write_text('#!'+sys.executable+'\nfrom pathlib import Path\nPath('+repr(str(runtime))+').write_bytes(b"changed by child")\n')
        product=replace(product,binary_sha256=file_hash(product.binary))
        script=self.fixture.root/'script.json';_=script.write_text('[]');output=self.fixture.root/'process-drift';output.mkdir()
        result=execute(bench_run.aggregate_spec(product,'base',1,script,file_hash(script),output))
        self.assertEqual(result.native_exit,0);self.assertEqual(result.status,'FAIL')
        self.assertTrue(document(result.receipt.read_text()).field('errors').array())

    def test_standalone_safety_audit_rejects_missing_runtime_process_binding(self) -> None:
        with safety_tests.scenario() as case:
            products={item.product.root:with_runtime(item.product) for item in case.aggregates}
            changed=replace(case,aggregates=tuple(replace(item,product=products[item.product.root]) for item in case.aggregates))
            with self.assertRaises(ValueError):_=safety_tests.audit(changed)
