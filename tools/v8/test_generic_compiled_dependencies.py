"""Tiny schema3 policy/process tests; no compiler provenance or native product claim."""
import json
import unittest
from dataclasses import replace
from pathlib import Path
from tempfile import TemporaryDirectory
from typing import override
from unittest.mock import patch

import gate_rustdoc_exec as wrapper
import generic_cargo_child as child
import generic_compiled_dependencies as compiled
import generic_workspace_run as workspace
import test_gate_rustdoc_exec as wrapper_tests
import test_generic_workspace_capture as capture_tests
import test_generic_workspace_cases as cases_tests
import test_generic_workspace_run as workspace_tests
from generic_gate import GateContractError
from generic_process import file_hash
from generic_workspace_capture import read, reconcile_capture
from mux2_gate_inputs import Json, Node

ORIGINAL_V2 = wrapper_tests.v2_fixture
ORIGINAL_CAPTURE = capture_tests.policy_fixture
ORIGINAL_PARENT = capture_tests.Fixture.parent


def upgrade(data: dict[str, Json], root: Path, *, count: int = 0) -> None:
    """Fixture membership only; provenance deliberately confers no build authority."""
    policy = Node(data, '/manifest').field('doctest_policy').mapping()
    inputs = Node(data, '/manifest').field('inputs').mapping()
    evidence = root/'compiled-provenance.json'
    _ = evidence.write_text('{"build_authority":"NOT_TESTED"}\n')
    for path in (evidence, Path(compiled.__file__).resolve()): inputs[str(path)] = file_hash(path)
    table: list[Json] = []
    if count:
        search = Node(policy['dependency_searches'], '/search').array()[0].mapping()
        rows = list(Node(search['files'], '/files').array())
        members: list[Json] = [row.value for row in rows]
        for index in range(count):
            name = f'compiled-{index:02d}'
            dependency = root/'deps'/name
            _ = dependency.write_bytes(('non-test compiled bytes '+str(index)).encode()); dependency.chmod(0o755)
            checksum = file_hash(dependency); inputs[str(dependency)] = checksum
            table.append({'id': name, 'path': str(dependency), 'sha256': checksum})
            members.insert(index, {'relative_path': name, 'binding_kind': 'compiled_dependency', 'binding_id': name, 'sha256': checksum})
        search['files'] = members; policy['dependency_searches'] = [search]
    policy.update({'schema_version': 3, 'compiled_dependencies': table,
                   'compiled_dependency_provenance': {'path': str(evidence), 'sha256': file_hash(evidence)}})
    data['inputs'] = inputs; data['doctest_policy'] = policy


def fixture(test: unittest.TestCase, *, count: int = 1) -> wrapper_tests.Fixture:
    result = ORIGINAL_V2(test)
    upgrade(result.data, result.root, count=count)
    result.policy = Node(result.data['doctest_policy'], '/policy').mapping()
    result.save()
    return result


def parent_fixture(result: capture_tests.Fixture, code: int) -> None:
    inputs = read(result.manifest).field('inputs').mapping()
    bound = dict(result.spec.external_inputs)
    bound.update({result.spec.input_root/name: digest for name, digest in result.spec.inputs})
    bound.update({Path(name): Node(value, name).text() for name, value in inputs.items()})
    source = tuple((str(path.relative_to(result.spec.input_root)), value) for path, value in bound.items() if path.is_relative_to(result.spec.input_root))
    external = tuple((path, value) for path, value in bound.items() if not path.is_relative_to(result.spec.input_root))
    result.spec = replace(result.spec, inputs=source, external_inputs=external)
    ORIGINAL_PARENT(result, code)
    for prefix in ('expected_', 'before_', 'after_'):
        result.parent_data[prefix+'inputs'] = dict(result.spec.inputs)
    for name in ('process.json', 'process-start.json'):
        capture_tests.write(result.output/name, result.parent_data)


def capture_fixture(test: unittest.TestCase) -> capture_tests.Fixture:
    result = ORIGINAL_CAPTURE(test)
    upgrade(result.manifest_data, result.base)
    capture_tests.write(result.manifest, result.manifest_data)
    inputs = Node(result.manifest_data['inputs'], '/inputs').mapping()
    bound = {path: value for path, value in result.spec.external_inputs}
    bound.update({Path(name): Node(value, name).text() for name, value in inputs.items()})
    bound[result.manifest] = file_hash(result.manifest)
    source = tuple((str(path.relative_to(result.spec.input_root)), value) for path, value in bound.items() if path.is_relative_to(result.spec.input_root))
    external = tuple((path, value) for path, value in bound.items() if not path.is_relative_to(result.spec.input_root))
    result.spec = replace(result.spec, inputs=source, external_inputs=external,
        environment=tuple((key, file_hash(result.manifest) if key == 'CARGO_CAPTURE_MANIFEST_SHA256' else value) for key, value in result.spec.environment))
    result.parent(0)
    return result


class Schema3Tests(unittest.TestCase):
    def test_nine_dependencies_never_inflate_154_test_artifacts(self) -> None:
        f = fixture(self, count=9)
        artifacts = [row.value for row in f.node().field('artifacts').array()]
        for index in range(153):
            path = f.root/f'test-artifact-{index}'; _ = path.write_bytes(str(index).encode())
            artifacts.append({'id': f'test-{index}', 'path': str(path), 'sha256': file_hash(path)})
        f.data['artifacts'] = artifacts; f.save()
        policy = wrapper.policy_for(f.node())
        self.assertEqual(len(compiled.read_compiled_dependencies(f.node(), policy)), 9)
        self.assertEqual(len(f.node().field('artifacts').array()), 154)
        wrapper.match_invocation(f.node(), policy, f.args, f.root).field('id').equals('alpha')
        with self.assertRaises(GateContractError): _ = wrapper.match_invocation(f.node(), policy, f.args, f.cwd)

    def test_bad_table_or_missing_binding_rejects(self) -> None:
        for defect in ('duplicate_id', 'duplicate_path', 'native_id', 'native_path', 'unbound', 'wrong_hash', 'symlink', 'unused', 'ordinary_kind', 'unknown_kind', 'provenance', 'missing_reader', 'extra_key', 'unsorted'):
            with self.subTest(defect=defect):
                f = fixture(self, count=2)
                rows = [row.mapping() for row in Node(f.policy['compiled_dependencies'], '/compiled').array()]
                inputs = Node(f.data['inputs'], '/inputs').mapping()
                if defect == 'duplicate_id': rows[1]['id'] = rows[0]['id']
                if defect == 'duplicate_path': rows[1]['path'] = rows[0]['path']
                if defect == 'native_id': rows[0]['id'] = f.node().field('artifacts').array()[0].field('id').value
                if defect == 'native_path': rows[0]['path'] = str(f.root/'deps/native')
                if defect == 'unbound': del inputs[str(f.root/'deps/compiled-00')]
                if defect == 'wrong_hash': rows[0]['sha256'] = '0'*64
                if defect == 'symlink':
                    path = f.root/'deps/compiled-00'; path.unlink(); path.symlink_to(f.source)
                if defect == 'unused': f.policy['dependency_searches'] = []
                if defect in ('ordinary_kind', 'unknown_kind'):
                    search = Node(f.policy['dependency_searches'], '/search').array()[0].mapping()
                    files = [row.mapping() for row in Node(search['files'], '/files').array()]
                    files[0]['binding_kind'] = 'input' if defect == 'ordinary_kind' else 'unreviewed'
                    files[0]['binding_id'] = str(f.root/'deps/compiled-00')
                    search['files'] = [row for row in files]; f.policy['dependency_searches'] = [search]
                if defect == 'provenance': _ = (f.root/'compiled-provenance.json').write_text('changed')
                if defect == 'missing_reader': del inputs[str(Path(compiled.__file__).resolve())]
                if defect == 'extra_key': rows[0]['build_verified'] = True
                if defect == 'unsorted': rows.reverse()
                f.policy['compiled_dependencies'] = [row for row in rows]; f.data['inputs'] = inputs; f.save()
                with self.assertRaises((GateContractError, OSError)): _ = wrapper.policy_for(f.node())

    def test_compiled_only_file_cannot_be_launched(self) -> None:
        f = fixture(self)
        with patch.object(Path, 'cwd', return_value=f.cwd):
            self.assertNotEqual(child.execute(f.manifest, file_hash(f.manifest), (str(f.root/'deps/compiled-00'),)), 0)
        receipt = read(f.capture/'000001/receipt.json')
        self.assertIsNone(receipt.field('child_pid').value)
        self.assertNotEqual(receipt.field('capture_status').text(), 'COMPLETE')

    def test_static_entry_and_terminal_recheck_all_compiled_inputs(self) -> None:
        f = fixture(self)
        with patch.object(Path, 'cwd', return_value=f.cwd):
            self.assertEqual(child.execute(f.manifest, file_hash(f.manifest), (str(f.root/'deps/native'),)), 0)
        intent = f.intent(); row = read(intent).mapping(); row['cwd'] = str(f.root)
        _ = intent.write_text(json.dumps(row))
        admission = f.admit()
        self.assertEqual(wrapper.postflight(admission), admission.evidence['identity_before'])
        _ = (f.root/'deps/compiled-00').write_bytes(b'changed unlaunched dependency')
        with self.assertRaises(GateContractError): _ = wrapper.postflight(admission)

    def test_parent_output_cannot_masquerade_as_immutable_provenance(self) -> None:
        f = capture_fixture(self)
        output = f.output/'stdout.log'
        data = read(f.manifest).mapping()
        inputs = Node(data['inputs']).mapping()
        inputs[str(output)] = file_hash(output); data['inputs'] = inputs
        policy = Node(data['doctest_policy']).mapping()
        policy['compiled_dependency_provenance'] = {'path': str(output), 'sha256': file_hash(output)}
        data['doctest_policy'] = policy; capture_tests.write(f.manifest, data)
        external = dict(f.spec.external_inputs); external[f.manifest] = file_hash(f.manifest)
        f.spec = replace(f.spec, external_inputs=tuple(external.items()), environment=tuple(
            (key, file_hash(f.manifest) if key == 'CARGO_CAPTURE_MANIFEST_SHA256' else value)
            for key, value in f.spec.environment))
        ORIGINAL_PARENT(f, 0)
        for prefix in ('expected_', 'before_', 'after_'): f.parent_data[prefix+'inputs'] = dict(f.spec.inputs)
        for name in ('process.json', 'process-start.json'): capture_tests.write(f.output/name, f.parent_data)
        _ = capture_tests.policy_static(f, 1); _ = capture_tests.policy_static(f, 2); _ = capture_tests.exec_record(f)
        self.assertNotIn(output, dict(f.spec.external_inputs))
        result = reconcile_capture(capture_tests.policy_plan(f), f.result)
        self.assertEqual(result.integrity, 'FAIL')
        self.assertTrue(any('actual parent bindings' in reason for _, reason in result.rejected))

    def test_workspace_parent_requires_compiled_and_provenance_inputs(self) -> None:
        for remove in (None, 'compiled', 'provenance'):
            with self.subTest(remove=remove), TemporaryDirectory() as temporary:
                base = Path(temporary).resolve(); support = workspace_tests.RuntimeTests(); self.addCleanup(support.doCleanups)
                request = support.fixture(base)
                f = fixture(self)
                policy = f.policy
                policy['parent_receipt'] = str(request.spec.output_root/'process.json')
                policy['exec_root'] = str(request.output/'rustdoc-execs')
                manifest = read(request.manifest).mapping(); manifest['doctest_policy'] = policy
                inputs = Node(manifest['inputs'], '/inputs').mapping()
                inputs.update(f.node().field('inputs').mapping()); manifest['inputs'] = inputs
                _ = request.manifest.write_text(json.dumps(manifest))
                bound = dict(request.spec.external_inputs)
                bound.update({Path(name): Node(value, name).text() for name, value in inputs.items() if not Path(name).is_relative_to(request.spec.input_root)})
                bound[request.manifest] = file_hash(request.manifest)
                if remove is not None: del bound[f.root/('deps/compiled-00' if remove == 'compiled' else 'compiled-provenance.json')]
                env = tuple((key, file_hash(request.manifest) if key == 'CARGO_CAPTURE_MANIFEST_SHA256' else value) for key, value in request.spec.environment)
                request = replace(request, manifest_sha256=file_hash(request.manifest), spec=replace(request.spec,
                    external_inputs=tuple(bound.items()), environment=(*env, ('RUSTDOC', str(f.entrypoint)))))
                with patch.object(workspace, 'LOCK', base/'lane'):
                    if remove is None: _ = workspace.checked(request)
                    else:
                        with self.assertRaises(GateContractError): _ = workspace.checked(request)


class Schema3LegacyWrapperTests(wrapper_tests.PolicyV2Tests):
    @override
    def setUp(self) -> None:
        context = patch.object(wrapper_tests, 'v2_fixture', fixture)
        _ = context.start(); self.addCleanup(context.stop)


class Schema3CaptureTests(capture_tests.TransientCaptureTests):
    @override
    def setUp(self) -> None:
        parent = patch.object(capture_tests.Fixture, 'parent', parent_fixture)
        _ = parent.start(); self.addCleanup(parent.stop)
        context = patch.object(capture_tests, 'policy_fixture', capture_fixture)
        _ = context.start(); self.addCleanup(context.stop)


class Schema3CaseTests(cases_tests.OrdinalTests):
    @override
    def setUp(self) -> None:
        parent = patch.object(capture_tests.Fixture, 'parent', parent_fixture)
        _ = parent.start(); self.addCleanup(parent.stop)
        context = patch.object(cases_tests, 'policy_fixture', capture_fixture)
        _ = context.start(); self.addCleanup(context.stop)
