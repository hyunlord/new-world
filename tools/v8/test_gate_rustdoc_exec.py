"""Offline rustdoc wrapper and endpoint contracts; not Cargo qualification."""
import json
import os
import shlex
import signal
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from typing import final
from unittest.mock import patch

import gate_rustdoc_exec as wrapper
import generic_cargo_child as child
from generic_gate import GateContractError, document
from generic_process import atomic_json, file_hash
from mux2_gate_inputs import Json, Node


@final
class Fixture:
    def __init__(self, test: unittest.TestCase) -> None:
        temporary = tempfile.TemporaryDirectory(); test.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name).resolve()
        self.cwd = self.root/'package'; self.cwd.mkdir()
        self.exec_root = self.root/'exec'; self.exec_root.mkdir()
        self.capture = self.root/'capture'; self.capture.mkdir()
        self.temp = self.root/'tmp'; self.temp.mkdir()
        self.output = self.root/'output'; self.output.mkdir()
        self.parent = self.root/'parent'; self.parent.mkdir()
        self.rustdoc = self.root/'rustdoc'
        _ = self.rustdoc.write_text('#!'+sys.executable+'\nimport os,sys,json\nprint(json.dumps({"pid":os.getpid(),"argv":sys.argv[1:],"env":dict(os.environ),"fd":os.read(int(os.environ["TEST_FD"]),4).decode()}))\n')
        self.rustdoc.chmod(0o755)
        self.entrypoint = self.root/'rustdoc-wrapper'
        _ = self.entrypoint.write_text('#!/bin/sh\nexec '+shlex.quote(sys.executable)+' '+shlex.quote(str(Path(wrapper.__file__).resolve()))+' "$@"\n')
        self.entrypoint.chmod(0o755)
        self.version = self.root/'version'; _ = self.version.write_text('synthetic version')
        self.source = self.cwd/'lib.rs'; _ = self.source.write_text('synthetic source')
        self.package = self.cwd/'Cargo.toml'; _ = self.package.write_text('synthetic manifest')
        self.args = ('--test', 'lib.rs', '--crate-name', 'alpha', '--edition', '2021')
        inputs = {str(p): file_hash(p) for p in (self.rustdoc, self.entrypoint, self.version, self.source, self.package, Path(wrapper.__file__).resolve())}
        self.policy: dict[str, Json] = {'schema_version': 1, 'parent_receipt': str(self.parent/'process.json'),
            'entrypoint': str(self.entrypoint), 'entrypoint_sha256': file_hash(self.entrypoint),
            'rustdoc': str(self.rustdoc), 'rustdoc_sha256': file_hash(self.rustdoc),
            'version_receipt': str(self.version), 'version_sha256': file_hash(self.version),
            'wrapper_sha256': file_hash(Path(wrapper.__file__).resolve()), 'exec_root': str(self.exec_root),
            'temporary_root': str(self.temp), 'invocations': [{'id': 'alpha', 'cwd': str(self.cwd), 'source': str(self.source),
                'package_manifest': str(self.package), 'crate_name': 'alpha', 'edition': '2021', 'argv': list(self.args)}]}
        self.data: dict[str, Json] = {'schema_version': 1, 'invocation_id': 'toy', 'step': 4, 'arm': 'candidate', 'profile': 'debug',
            'output_root': str(self.capture), 'launcher_sha256': file_hash(Path(child.__file__).resolve()),
            'python_sha256': file_hash(Path(sys.executable).resolve()), 'inputs': {k: v for k, v in inputs.items()},
            'artifacts': [], 'observer_variable': None, 'doctest_policy': self.policy}
        self.manifest = self.root/'manifest.json'; self.save()
        self.binary = self.temp/'rustdoctestABC/rust_out'; self.binary.parent.mkdir()
        _ = self.binary.write_text('#!'+sys.executable+'\nraise SystemExit(0)\n'); self.binary.chmod(0o755)

    def save(self) -> None:
        _ = self.manifest.write_text(json.dumps(self.data))

    def node(self) -> Node:
        return document(self.manifest.read_bytes().decode())

    def intent(self) -> Path:
        directory = self.exec_root/'exec-one'; directory.mkdir()
        identity: dict[str, Json] = {'schema_version': 1, 'component': 'RUSTDOC_EXEC_INTENT', 'occurrence': 'exec-one',
            'invocation_id': 'toy', 'step': 4, 'arm': 'candidate', 'profile': 'debug', 'manifest_sha256': file_hash(self.manifest),
            'wrapper_pid': os.getppid(), 'cargo_parent_pid': 1234, 'cwd': str(self.cwd), 'source_invocation': 'alpha',
            'rustdoc_argv': [str(self.rustdoc), *self.args], 'wrapper_argv': [str(Path(wrapper.__file__).resolve()), *self.args],
            'environment': {'CARGO_CAPTURE_MANIFEST_SHA256': file_hash(self.manifest)}}
        atomic_json(directory/'exec-intent.json', identity)
        atomic_json(self.parent/'process-spawn.json', {'pid': 1234})
        return directory/'exec-intent.json'

    def admit(self) -> wrapper.Transient:
        with patch.object(wrapper, 'live_parent_path', return_value=str(self.rustdoc)):
            return wrapper.admit_transient(self.node(), file_hash(self.manifest), (str(self.binary),), self.cwd, self.output)


class WrapperTests(unittest.TestCase):
    def test_same_pid_exec_preserves_arguments_environment_and_selected_fd(self) -> None:
        f = Fixture(self)
        fd, path = tempfile.mkstemp(dir=f.root)
        try:
            _ = os.write(fd, b'abcd'); _ = os.lseek(fd, 0, os.SEEK_SET)
            env = {**os.environ, 'CARGO_CAPTURE_MANIFEST': str(f.manifest), 'CARGO_CAPTURE_MANIFEST_SHA256': file_hash(f.manifest),
                   'TEST_FD': str(fd), 'PRESERVED_TEST': 'space 한글', 'PYTHONPATH': '/Users/rexxa/work/new-world-m6b-dev/tools/v8'}
            with subprocess.Popen([str(f.entrypoint), *f.args], cwd=f.cwd, env=env, pass_fds=(fd,),
                                  stdout=subprocess.PIPE, stderr=subprocess.PIPE) as process:
                stdout, stderr = process.communicate(timeout=5)
                self.assertEqual((process.returncode, stderr), (0, b''))
                result = document(stdout.decode())
                result.field('pid').equals(process.pid); result.field('argv').equals(list(f.args))
                result.field('fd').equals('abcd'); result.field('env').field('PRESERVED_TEST').equals('space 한글')
            intent = wrapper.read(next(f.exec_root.glob('*/exec-intent.json')))
            intent.field('wrapper_pid').equals(result.field('pid').value)
            intent.field('ancestor').field('status').equals('PROVISIONAL')
        finally:
            os.close(fd); Path(path).unlink()

    def test_admitted_copy_matches_and_postflight_retains_endpoint(self) -> None:
        f = Fixture(self); _ = f.intent()
        admission = f.admit()
        self.assertEqual(admission.lexical, f.binary)
        self.assertEqual(file_hash(f.output/'transient-binary'), file_hash(f.binary))
        self.assertEqual(wrapper.postflight(admission), admission.evidence['identity_before'])
        self.assertEqual(Node(admission.evidence['ancestor'], '/ancestor').field('final_parent_qualification').text(),
                         'PENDING_AUTHENTICATED_PROCESS_RECEIPT_JOIN')
        _ = f.binary.write_text('changed')
        with self.assertRaises(GateContractError): _ = wrapper.postflight(admission)

    def test_missing_ambiguous_failed_and_wrong_ancestor_intents_reject(self) -> None:
        for defect in ('missing', 'ambiguous', 'failed', 'ancestor', 'spawn_missing', 'manifest', 'source'):
            with self.subTest(defect=defect):
                f = Fixture(self)
                if defect != 'missing':
                    intent = f.intent()
                    if defect == 'ambiguous':
                        other = f.exec_root/'exec-two'; other.mkdir(); _ = (other/'exec-intent.json').write_bytes(intent.read_bytes())
                    if defect == 'failed': atomic_json(intent.with_name('exec-failure.json'), {'error': 'exec failure'})
                    if defect == 'ancestor': _ = (f.parent/'process-spawn.json').write_text('{"pid":999}')
                    if defect == 'spawn_missing': (f.parent/'process-spawn.json').unlink()
                    if defect == 'manifest':
                        body = wrapper.read(intent).mapping(); body['manifest_sha256'] = 'wrong'; _ = intent.write_text(json.dumps(body))
                    if defect == 'source': _ = f.source.write_text('changed')
                with self.assertRaises((GateContractError, OSError)): _ = f.admit()

    def test_wrong_live_parent_and_changed_ppid_reject(self) -> None:
        for changed in (False, True):
            f = Fixture(self); _ = f.intent()
            with patch.object(wrapper, 'live_parent_path', return_value=str(f.source) if not changed else str(f.rustdoc)), \
                 patch.object(os, 'getppid', side_effect=[os.getppid(), 999] if changed else None, return_value=os.getppid()), self.assertRaises(GateContractError):
                _ = wrapper.admit_transient(f.node(), file_hash(f.manifest), (str(f.binary),), f.cwd, f.output)

    def test_alternate_symlink_and_static_arguments_reject(self) -> None:
        f = Fixture(self); _ = f.intent()
        alias = f.root/'alias'; alias.symlink_to(f.temp, target_is_directory=True)
        with self.assertRaises(GateContractError): _ = wrapper.transient_identity(alias/'rustdoctestABC/rust_out', f.temp)
        with self.assertRaises(GateContractError):
            _ = wrapper.admit_transient(f.node(), file_hash(f.manifest), (str(f.binary), '--test-threads=8'), f.cwd, f.output)

    def test_explicit_platform_var_alias_preserves_lexical_path(self) -> None:
        f = Fixture(self)
        if not str(f.binary).startswith('/private/var/'):
            self.skipTest('platform /var alias unavailable')
        lexical = Path(str(f.binary).replace('/private/var/', '/var/', 1))
        canonical, identity = wrapper.transient_identity(lexical, f.temp)
        self.assertEqual(canonical, f.binary)
        self.assertEqual(identity['lexical'], str(lexical))
        self.assertEqual(identity['alias'], '/var -> /private/var')

    def test_missing_spawn_retains_failed_closed_child_receipt(self) -> None:
        f = Fixture(self); _ = f.intent(); (f.parent/'process-spawn.json').unlink()
        with patch.object(Path, 'cwd', return_value=f.cwd):
            self.assertEqual(child.execute(f.manifest, file_hash(f.manifest), (str(f.binary),)), 125)
        receipt = wrapper.read(f.capture/'000001/receipt.json')
        receipt.field('native_exit').equals(None); receipt.field('child_pid').equals(None)
        receipt.field('capture_status').equals('FAIL')
        self.assertTrue((f.capture/'000001/pending.json').is_file())

    def test_parent_future_receipt_cannot_be_immutable_input(self) -> None:
        f = Fixture(self)
        values = Node(f.data['inputs'], '/inputs').mapping(); values[str(f.parent/'process.json')] = 'future'
        f.data['inputs'] = values; f.save()
        with self.assertRaises(GateContractError): _ = wrapper.policy_for(f.node())

    def test_exec_failure_is_retained_without_fake_terminal(self) -> None:
        f = Fixture(self)
        with patch.object(Path, 'cwd', return_value=f.cwd), patch.object(os, 'execve', side_effect=OSError('injected exec failure')), self.assertRaises(OSError):
            wrapper.exec_rustdoc(f.manifest, file_hash(f.manifest), f.args)
        self.assertEqual(len(tuple(f.exec_root.glob('*/exec-intent.json'))), 1)
        self.assertEqual(len(tuple(f.exec_root.glob('*/exec-failure.json'))), 1)

    def test_unbound_dependency_is_rejected_even_in_exact_declared_arguments(self) -> None:
        f = Fixture(self)
        dependency = f.cwd/'unbound.rlib'; _ = dependency.write_bytes(b'not frozen')
        args = (*f.args, '--extern', 'dep='+str(dependency))
        rows = Node(f.policy['invocations'], '/invocations').array()
        declaration = rows[0].mapping(); declaration['argv'] = list(args)
        f.policy['invocations'] = [declaration]; f.save()
        with self.assertRaises(GateContractError):
            _ = wrapper.match_invocation(f.node(), wrapper.policy_for(f.node()), args, f.cwd)

    def test_unsupported_argv_retains_raw_failure(self) -> None:
        f = Fixture(self)
        with patch.object(Path, 'cwd', return_value=f.cwd), self.assertRaises(GateContractError):
            wrapper.exec_rustdoc(f.manifest, file_hash(f.manifest), (*f.args, '--unknown'))
        failure = wrapper.read(next(f.exec_root.glob('*/exec-failure.json')))
        failure.field('rustdoc_argv').equals([str(f.rustdoc), *f.args, '--unknown'])
        self.assertEqual(len(tuple(f.exec_root.glob('*/exec-intent.json'))), 0)

    def test_bound_policy_input_failure_retains_entry(self) -> None:
        f = Fixture(self); _ = f.source.write_text('drift')
        with patch.object(Path, 'cwd', return_value=f.cwd), self.assertRaises(GateContractError):
            wrapper.exec_rustdoc(f.manifest, file_hash(f.manifest), f.args)
        self.assertEqual(len(tuple(f.exec_root.glob('*/exec-entry.json'))), 1)
        failure = wrapper.read(next(f.exec_root.glob('*/exec-failure.json')))
        failure.field('rustdoc_argv').equals([str(f.rustdoc), *f.args])

    def test_joined_dependency_flags_rejected(self) -> None:
        for flag in ('--extern=dep=', '-Ldependency='):
            f = Fixture(self)
            path = f.cwd/'unbound'; _ = path.write_bytes(b'unbound')
            args = (*f.args, flag+str(path))
            declaration = Node(f.policy['invocations'], '/invocations').array()[0].mapping()
            declaration['argv'] = list(args); f.policy['invocations'] = [declaration]; f.save()
            with self.assertRaises(GateContractError):
                _ = wrapper.match_invocation(f.node(), wrapper.policy_for(f.node()), args, f.cwd)

    def test_intent_mutation_during_probe_rejected(self) -> None:
        f = Fixture(self); intent = f.intent()
        def mutate(_pid: int) -> str:
            data = wrapper.read(intent).mapping(); data['source_invocation'] = 'forged'
            _ = intent.write_text(json.dumps(data))
            return str(f.rustdoc)
        with patch.object(wrapper, 'live_parent_path', side_effect=mutate), self.assertRaises(GateContractError):
            _ = wrapper.admit_transient(f.node(), file_hash(f.manifest), (str(f.binary),), f.cwd, f.output)

    def test_wrong_image_retains_raw_live_probe(self) -> None:
        f = Fixture(self); _ = f.intent()
        with patch.object(wrapper, 'live_parent_path', return_value=str(f.source)), self.assertRaises(GateContractError):
            _ = wrapper.admit_transient(f.node(), file_hash(f.manifest), (str(f.binary),), f.cwd, f.output)
        wrapper.read(f.output/'live-parent.json').field('path').equals(str(f.source))

    def test_transient_native_nonzero_and_signal_retained(self) -> None:
        for code in (7, -signal.SIGTERM):
            with self.subTest(code=code):
                f = Fixture(self); _ = f.intent()
                body = 'raise SystemExit(7)' if code == 7 else 'import os,signal; os.kill(os.getpid(),signal.SIGTERM)'
                _ = f.binary.write_text('#!'+sys.executable+'\n'+body+'\n')
                with patch.object(wrapper, 'live_parent_path', return_value=str(f.rustdoc)), patch.object(Path, 'cwd', return_value=f.cwd):
                    self.assertEqual(child.execute(f.manifest, file_hash(f.manifest), (str(f.binary),)), code)
                receipt = wrapper.read(f.capture/'000001/receipt.json')
                receipt.field('native_exit').equals(code); receipt.field('capture_status').equals('COMPLETE')
                receipt.field('artifact_id').equals(None); receipt.field('transient').field('classification').equals('RUSTDOC_TRANSIENT')


if __name__ == '__main__':
    _ = unittest.main()
