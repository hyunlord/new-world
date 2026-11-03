"""Offline successor-policy boundary tests; no native Cargo qualification."""
import json
import os
import sys
import tempfile
import unittest
from dataclasses import replace
from pathlib import Path
from unittest.mock import patch

import gate_rustdoc_exec as wrapper
import generic_cargo_child as child
import generic_compiled_dependencies as compiled
import generic_workspace_arguments as dynamic
import generic_workspace_run as workspace
import test_gate_rustdoc_exec as legacy
import test_generic_compiled_dependencies as compiled_tests
from generic_gate import document
from generic_process import ProcessSpec, environment_for, execute, file_hash
from generic_workspace_capture import CapturePlan, WorkspaceCapture, exec_binding
from generic_workspace_tests import Inventory
from mux2_gate_inputs import Json, Node


class ParentTests(unittest.TestCase):
    def fixture(self) -> tuple[Node, Node, Path, str]:
        temporary = tempfile.TemporaryDirectory(); self.addCleanup(temporary.cleanup)
        root = Path(temporary.name).resolve(); (root/'rust').mkdir(); (root/'parent').mkdir()
        binary = Path(sys.executable).resolve()
        projection: dict[str, Json] = {'executable': str(binary), 'executable_sha256': file_hash(binary),
            'argv': [str(binary), 'test', '--workspace', '--no-fail-fast', '--', '--test-threads=8'],
            'cwd': str(root/'rust'), 'input_root': str(root), 'environment': {}, 'inputs': {},
            'external_inputs': {}, 'fresh_directories': []}
        policy: dict[str, Json] = {'schema_version': 4, 'parent_spec': projection,
                                  'parent_receipt': str(root/'parent/process.json')}
        plan: dict[str, Json] = {'step': 4, 'doctest_policy': policy}
        manifest = root/'manifest.json'; _ = manifest.write_text(json.dumps(plan)); digest = file_hash(manifest)
        start: dict[str, Json] = {'schema_version': 1, 'argv': projection['argv'], 'cwd': projection['cwd'],
            'executable': str(binary), 'expected_executable_sha256': file_hash(binary), 'before_executable_sha256': file_hash(binary),
            'input_root': str(root), 'output_root': str(root/'parent'), 'environment': {'CARGO_CAPTURE_MANIFEST': str(manifest), 'CARGO_CAPTURE_MANIFEST_SHA256': digest},
            'expected_inputs': {}, 'before_inputs': {}, 'expected_external_inputs': {str(manifest): digest},
            'before_external_inputs': {str(manifest): digest}, 'fresh_directories': [],
            'environment_policy': 'fixed_defaults_plus_explicit_only_no_inheritance', 'pid': None, 'native_exit': None, 'errors': []}
        projection['environment'] = {'CARGO_CAPTURE_MANIFEST': str(manifest)}
        _ = manifest.write_text(json.dumps(plan)); digest = file_hash(manifest)
        start['environment'] = {'CARGO_CAPTURE_MANIFEST': str(manifest), 'CARGO_CAPTURE_MANIFEST_SHA256': digest}
        start['expected_external_inputs'] = {str(manifest): digest}; start['before_external_inputs'] = {str(manifest): digest}
        _ = (root/'parent/process-start.json').write_text(json.dumps(start))
        _ = (root/'parent/process-spawn.json').write_text(json.dumps({'pid': os.getppid()}))
        return Node(plan, '/'), Node(policy, '/policy'), manifest, digest

    def test_strict_parent_joins_spawn_start_and_live_image(self) -> None:
        plan, policy, manifest, digest = self.fixture()
        with patch.object(wrapper, 'live_parent_path', return_value=str(Path(sys.executable).resolve())):
            row = wrapper.strict_ancestor(plan, policy, manifest, digest, os.getppid())
        self.assertEqual(row['cargo_pid'], os.getppid())
        self.assertIn('strict_parent', row)

    def test_strict_parent_rejects_wrong_start_before_live_admission(self) -> None:
        plan, policy, manifest, digest = self.fixture()
        path = Path(policy.field('parent_receipt').text()).with_name('process-start.json')
        changed = document(path.read_text()).mapping(); changed['argv'] = ['foreign']
        _ = path.write_text(json.dumps(changed))
        with self.assertRaises(ValueError):
            _ = wrapper.strict_ancestor(plan, policy, manifest, digest, os.getppid())

    def test_actual_parent_identity_and_projection_negatives(self) -> None:
        for fault in ('pid', 'image', 'environment', 'manifest', 'threads', 'input', 'terminal'):
            with self.subTest(fault=fault):
                plan, policy, manifest, digest = self.fixture()
                terminal = Path(policy.field('parent_receipt').text()); start = terminal.with_name('process-start.json')
                body = wrapper.read(start).mapping()
                if fault == 'pid':
                    _ = terminal.with_name('process-spawn.json').write_text(json.dumps({'pid': os.getppid()+1}))
                if fault == 'environment': body['environment'] = {}
                if fault == 'manifest': _ = manifest.write_text('changed')
                if fault == 'threads': body['argv'] = ['changed']
                if fault == 'input': body['before_external_inputs'] = {}
                if fault == 'terminal': _ = terminal.write_text('{}')
                _ = start.write_text(json.dumps(body))
                image = str(manifest) if fault == 'image' else str(Path(sys.executable).resolve())
                with patch.object(wrapper, 'live_parent_path', return_value=image), self.assertRaises(ValueError):
                    _ = wrapper.strict_ancestor(plan, policy, manifest, digest, os.getppid())

    def test_parent_receipt_mutation_during_live_probe_rejects(self) -> None:
        plan, policy, manifest, digest = self.fixture()
        def mutate(_pid: int) -> str:
            path = Path(policy.field('parent_receipt').text()).with_name('process-start.json')
            _ = path.write_text('{}')
            return str(Path(sys.executable).resolve())
        with patch.object(wrapper, 'live_parent_path', side_effect=mutate), self.assertRaises(ValueError):
            _ = wrapper.strict_ancestor(plan, policy, manifest, digest, os.getppid())


def dynamic_fixture(test: unittest.TestCase) -> legacy.Fixture:
    f = legacy.v2_fixture(test)
    compiled_tests.upgrade(f.data, f.root, count=1)
    f.policy = Node(f.data, '/').field('doctest_policy').mapping()
    inputs = Node(f.data, '/').field('inputs').mapping()
    for path in (Path(dynamic.__file__).resolve(), Path(sys.executable).resolve(), Path(child.__file__).resolve()):
        inputs[str(path)] = file_hash(path)
    prefix = ['--edition=2021', '--crate-type', 'lib', '--color', 'auto', '--crate-name', 'alpha',
              '--test', 'package/lib.rs', '--test-run-directory', str(f.cwd), '--test-runtool', str(Path(sys.executable).resolve()),
              '--test-runtool-arg', str(Path(child.__file__).resolve()), '-L', 'dependency='+str(f.root/'deps'),
              '-L', 'dependency='+str(f.root/'deps'), '--test-args', '--test-threads=8']
    suffix = ['-C', 'embed-bitcode=no', '--check-cfg', 'cfg(docsrs,test)', '--check-cfg', 'cfg(feature, values())', '--error-format', 'human']
    row = Node(f.policy, '/').field('invocations').array()[0].mapping(); del row['argv']
    arguments: dict[str, Json] = {'prefix': list(prefix), 'externs': {'alpha': str(f.root/'deps/native')}, 'suffix': list(suffix)}
    row['argument_policy'] = arguments
    f.policy.update({'schema_version': 4, 'parent_spec': {}, 'invocations': [row]})
    f.args = (*prefix, '--extern', 'alpha='+str(f.root/'deps/native'), *suffix)
    f.data['inputs'] = inputs; f.save()
    return f


class DynamicTests(unittest.TestCase):
    def test_dynamic_matcher_preserves_obligation_and_rejects_changed_argument(self) -> None:
        f = dynamic_fixture(self); policy = wrapper.policy_for(f.node())
        wrapper.match_invocation(f.node(), policy, f.args, f.root).field('id').equals('alpha')
        for args in ((*f.args, '--unknown'), tuple('--test-threads=9' if x == '--test-threads=8' else x for x in f.args)):
            with self.assertRaises(ValueError): _ = wrapper.match_invocation(f.node(), policy, args, f.root)
        self.assertEqual(len(compiled.read_compiled_dependencies(f.node(), policy)), 1)

    def test_missing_parent_fails_closed_before_exec(self) -> None:
        f = dynamic_fixture(self)
        # Existing schema1 provisional path remains tested in the unchanged suite.
        with patch.object(Path, 'cwd', return_value=f.root), self.assertRaises(ValueError):
            wrapper.exec_rustdoc(f.manifest, file_hash(f.manifest), f.args)
        self.assertFalse(tuple(f.exec_root.glob('*/exec-intent.json')))
        failure = wrapper.read(next(f.exec_root.glob('*/exec-failure.json')))
        self.assertNotIn('ancestor', failure.mapping())

    def test_actual_tiny_parent_admits_unmodified_argv_and_terminal_join(self) -> None:
        f = dynamic_fixture(self)
        source_root = f.root/'source'; compiler_cwd = source_root/'rust'; compiler_cwd.mkdir(parents=True)
        old_source, old_package = f.source, f.package
        _ = f.cwd.rename(compiler_cwd/'package'); f.cwd = compiler_cwd/'package'; f.source = f.cwd/'lib.rs'; f.package = f.cwd/'Cargo.toml'
        inputs = Node(f.data, '/').field('inputs').mapping(); del inputs[str(old_source)]; del inputs[str(old_package)]
        for path in (f.source, f.package): inputs[str(path)] = file_hash(path)
        _ = f.rustdoc.write_text('#!'+sys.executable+'\nimport sys,json,os\nprint(json.dumps({"argv":sys.argv[1:],"pid":os.getpid()}))\n')
        inputs[str(f.rustdoc)] = file_hash(f.rustdoc); f.policy['rustdoc_sha256'] = file_hash(f.rustdoc)
        row = Node(f.policy, '/').field('invocations').array()[0].mapping()
        row.update({'source': str(f.source), 'package_manifest': str(f.package), 'compiler_cwd': str(compiler_cwd), 'run_cwd': str(f.cwd)})
        args = tuple(str(f.cwd) if x == str(old_package.parent) else x for x in f.args)
        argument = Node(row['argument_policy'], '/').mapping()
        argument['prefix'] = [str(f.cwd) if x.text() == str(old_package.parent) else x.text() for x in Node(argument['prefix'], '/').array()]
        row['argument_policy'] = argument; f.policy['invocations'] = [row]
        python = Path(wrapper.live_parent_path(os.getpid())).resolve(); script = compiler_cwd/'test'
        _ = script.write_text('import subprocess,sys\nraise SystemExit(subprocess.call('+repr([str(f.entrypoint), *args])+'))\n')
        inputs[str(script)] = file_hash(script); f.data['inputs'] = inputs
        f.parent.rmdir()
        environment = (('PYTHONPATH', str(Path(wrapper.__file__).resolve().parent)+os.pathsep+str(Path(wrapper.__file__).resolve().parent/'mirror')),
                       ('CARGO_CAPTURE_MANIFEST', str(f.manifest)))
        relative = tuple((str(Path(path).relative_to(source_root)), str(digest)) for path, digest in inputs.items() if Path(path).is_relative_to(source_root))
        external = tuple((Path(path), str(digest)) for path, digest in inputs.items() if not Path(path).is_relative_to(source_root))
        spec = ProcessSpec((str(python),'test','--workspace','--no-fail-fast','--','--test-threads=8'), compiler_cwd,
                           python, file_hash(python), source_root, relative, f.parent, environment, external)
        f.policy['parent_spec'] = {'executable': str(python), 'executable_sha256': file_hash(python), 'argv': list(spec.argv),
            'cwd': str(compiler_cwd), 'input_root': str(source_root), 'environment': dict(environment_for(spec)),
            'inputs': dict(relative), 'external_inputs': {str(path): digest for path,digest in external}, 'fresh_directories': []}
        f.save(); checksum = file_hash(f.manifest)
        spec = replace(spec, environment=(*environment, ('CARGO_CAPTURE_MANIFEST_SHA256', checksum)), external_inputs=(*external,(f.manifest,checksum)))
        result = execute(spec)
        self.assertEqual(result.native_exit, 0, result.receipt.with_name('stderr.log').read_text())
        observed = document(result.receipt.with_name('stdout.log').read_text())
        observed.field('argv').equals(list(args))
        intent = wrapper.read(next(f.exec_root.glob('*/exec-intent.json')))
        intent.field('wrapper_pid').equals(observed.field('pid').value)
        intent.field('ancestor').field('cargo_pid').equals(result.pid)
        joined = wrapper.parent_start_binding(f.node(), Node(f.policy,'/'), f.manifest, checksum)
        intent.field('ancestor').field('strict_parent').field('start_sha256').equals(joined['start_sha256'])

        plan = CapturePlan('toy', 4, 'candidate', 'debug', spec, file_hash(result.receipt),
                           f.manifest, checksum, Inventory((), ()), f.capture, ())
        intent_path = next(f.exec_root.glob('*/exec-intent.json'))
        captured = exec_binding(plan, result, f.node(), Node(f.policy,'/'), intent_path.parent)
        capture = WorkspaceCapture((), (), (), (), 'PASS', 0, 'PASS', 'COMPLETE', 'SUCCEEDED', exec_intents=(captured,))
        request = workspace.WorkspaceRequest(4, 'candidate', 'toy', spec, Inventory((), ()), f.manifest, checksum, f.root/'request', -1)
        jobs = workspace.listing_jobs(request, capture)
        self.assertEqual(jobs[0][1], (str(f.rustdoc), *args))
        self.assertEqual(jobs[0][2], compiler_cwd)
        with self.assertRaises(ValueError): _ = workspace.listing_jobs(request, replace(capture, exec_intents=(captured, captured)))
        changed = wrapper.read(intent_path).mapping(); changed['rustdoc_argv'] = [str(f.rustdoc), *args, '--unknown']
        _ = intent_path.write_text(json.dumps(changed))
        with self.assertRaises(ValueError): _ = workspace.listing_jobs(request, capture)


class ObligationTests(unittest.TestCase):
    def test_missing_duplicate_and_foreign_obligations_do_not_become_optional(self) -> None:
        import test_generic_workspace_capture as captures
        from generic_workspace_capture import reconcile_capture
        for fault in ('missing-exec', 'duplicate', 'foreign'):
            with self.subTest(fault=fault):
                f = captures.policy_fixture(self)
                compiled_tests.upgrade(f.manifest_data, f.base)
                policy = Node(f.manifest_data, '/').field('doctest_policy').mapping()
                inputs = Node(f.manifest_data, '/').field('inputs').mapping()
                deps = f.base/'deps'; deps.mkdir(); member = deps/'libalpha.rlib'; _ = member.write_text('synthetic dependency')
                python = Path(sys.executable).resolve()
                for path in (member, python, Path(dynamic.__file__).resolve()): inputs[str(path)] = file_hash(path)
                row = Node(policy, '/').field('invocations').array()[0].mapping(); del row['argv']
                prefix = ['--edition=2021', '--crate-type', 'lib', '--color', 'auto', '--crate-name', 'alpha',
                    '--test', str(f.executables[0].target.source), '--test-run-directory', str(f.executables[0].cwd),
                    '--test-runtool', str(python), '--test-runtool-arg', str(f.helper), '-L', 'dependency='+str(deps),
                    '-L', 'dependency='+str(deps), '--test-args', '--test-threads=8']
                arguments: dict[str, Json] = {'prefix': list(prefix), 'externs': {'alpha': str(member)},
                    'suffix': ['-C','embed-bitcode=no','--check-cfg','cfg(docsrs,test)','--check-cfg','cfg(feature, values())','--error-format','human']}
                row['argument_policy'] = arguments
                policy.update({'schema_version': 4, 'parent_spec': {}, 'dependency_searches': [{'path': str(deps), 'directories': [],
                    'files': [{'relative_path': member.name, 'binding_kind': 'input', 'binding_id': str(member), 'sha256': file_hash(member)}]}],
                    'invocations': [row,row] if fault == 'duplicate' else [row]})
                if fault == 'foreign': row['package_id'] = 'foreign'
                f.manifest_data['inputs'] = inputs
                captures.write(f.manifest, f.manifest_data)
                external = {Path(name): str(digest) for name,digest in inputs.items()}
                external[f.manifest] = file_hash(f.manifest); external[f.cargo] = file_hash(f.cargo)
                f.spec = replace(f.spec, external_inputs=tuple(external.items()), environment=(('CARGO_CAPTURE_MANIFEST',str(f.manifest)),
                    ('CARGO_CAPTURE_MANIFEST_SHA256',file_hash(f.manifest))))
                f.parent(0); _ = captures.policy_static(f,1); _ = captures.policy_static(f,2)
                _ = captures.exec_record(f, probe=True)
                result = reconcile_capture(captures.policy_plan(f), f.result)
                self.assertNotEqual(result.completeness, 'COMPLETE')
                if fault == 'missing-exec': self.assertEqual(result.missing_exec_invocations, ('alpha-doc',))
                else: self.assertTrue(result.rejected)


class FinalPublicationTests(unittest.TestCase):
    def test_final_parent_callback_cannot_change_intended_exec_evidence(self) -> None:
        for fault in ('entry', 'intent', 'directory'):
            with self.subTest(fault=fault):
                f = dynamic_fixture(self)
                parent_case = ParentTests(); self.addCleanup(parent_case.doCleanups)
                _, partial, _, _ = parent_case.fixture()
                recipe = partial.field('parent_spec').mapping()
                recipe['environment'] = {'CARGO_CAPTURE_MANIFEST': str(f.manifest)}
                inputs = Node(f.data, '/').field('inputs').mapping()
                recipe['external_inputs'] = dict(inputs)
                f.policy['parent_spec'] = recipe
                f.policy['parent_receipt'] = partial.field('parent_receipt').value
                f.save(); checksum = file_hash(f.manifest)
                start_path = Path(partial.field('parent_receipt').text()).with_name('process-start.json')
                start = wrapper.read(start_path).mapping()
                start['environment'] = {'CARGO_CAPTURE_MANIFEST': str(f.manifest), 'CARGO_CAPTURE_MANIFEST_SHA256': checksum}
                external: dict[str, Json] = {**inputs, str(f.manifest): checksum}
                start['expected_external_inputs'] = external; start['before_external_inputs'] = dict(external)
                _ = start_path.write_text(json.dumps(start))
                calls = 0
                def live(_pid: int, *, fault: str = fault, root: Path = f.exec_root) -> str:
                    nonlocal calls
                    calls += 1
                    if calls == 2:
                        occurrence = next(root.iterdir())
                        if fault == 'directory':
                            (occurrence/'late-empty').mkdir()
                        else:
                            path = occurrence/('exec-'+fault+'.json')
                            body = wrapper.read(path).mapping(); body['late_injected'] = True
                            _ = path.write_text(json.dumps(body))
                    return str(Path(sys.executable).resolve())
                with patch.object(wrapper, 'live_parent_path', side_effect=live), patch.object(Path, 'cwd', return_value=f.root), \
                        patch.object(os, 'execve') as launch:
                    with self.assertRaisesRegex(ValueError, 'intended publication changed'):
                        wrapper.exec_rustdoc(f.manifest, checksum, ('-vV',))
                    launch.assert_not_called()
                self.assertEqual(calls, 2)
                failure = wrapper.read(next(f.exec_root.glob('*/exec-failure.json')))
                failure.field('rustdoc_argv').equals([str(f.rustdoc), '-vV'])
