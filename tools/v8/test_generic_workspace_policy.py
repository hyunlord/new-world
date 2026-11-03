"""Pure source/argument fixtures; mocked build boundary grants no native evidence."""
import json
import shutil
import subprocess
import sys
import tempfile
import unittest
from dataclasses import replace
from pathlib import Path
from unittest.mock import patch

import generic_workspace_arguments as arguments
import generic_workspace_policy as policy
import test_generic_build as build_tests
from generic_build import (
    BuildSpec,
    WorkspaceBuildPair,
    WorkspaceSourceProof,
    build_workspace_pair,
)
from generic_gate import document
from generic_process import file_hash
from mux2_gate_inputs import Json, Node


def fixture(root: Path) -> tuple[Node, tuple[Node, ...], tuple[Path, Path]]:
    root.mkdir(exist_ok=True)
    _ = (root/'Cargo.lock').write_text('version = 4\npackage=[]\n')
    _ = (root/'Cargo.toml').write_text('[profile.release]\nlto="thin"\ncodegen-units=1\n')
    packages: list[Json] = []; messages: list[Node] = []
    for name in sorted(policy.TARGETS):
        directory = root/'crates'/name
        directory.mkdir(parents=True)
        _ = (directory/'lib.rs').write_text('pub fn sample() {}\n')
        _ = (directory/'Cargo.toml').write_text('[package]\nname='+json.dumps(name)+'\n[features]\nnew_observer=[]\n')
        crate = name.replace('-', '_'); ident = 'path+file://'+str(directory)+'#'+name+'@0.7.0'
        types = ['cdylib', 'rlib'] if name == 'sim-bridge' else ['lib']
        target: dict[str, Json] = {'name': crate, 'src_path': str(directory/'lib.rs'), 'kind': [*types],
            'crate_types': [*types], 'edition': '2021', 'doctest': True, 'test': True}
        package: dict[str, Json] = {'id': ident, 'name': name, 'manifest_path': str(directory/'Cargo.toml'),
            'features': {'new_observer': []}, 'targets': [target], 'dependencies': []}
        packages.append(package)
        binary = root/'target/debug/deps'/('lib'+crate+'.rlib'); binary.parent.mkdir(parents=True, exist_ok=True)
        _ = binary.write_bytes(b'compiled fixture')
        messages.append(document(json.dumps({'reason': 'compiler-artifact', 'package_id': ident, 'target': target,
            'profile': {'test': False}, 'features': [], 'fresh': False, 'filenames': [str(binary)]})))
    metadata: dict[str, Json] = {'version': 1, 'workspace_root': str(root), 'target_directory': str(root/'target'),
        'workspace_members': [document(json.dumps(p)).field('id').value for p in packages], 'packages': packages}
    runner = root.parent/'python'; helper = root.parent/'generic_cargo_child.py'
    _ = runner.write_bytes(b'not executable'); _ = helper.write_bytes(b'# fixture')
    return document(json.dumps(metadata)), tuple(messages), (runner, helper)


def rows(metadata: Node, messages: tuple[Node, ...], root: Path, runner: tuple[Path, Path]) -> Node:
    return document(json.dumps({'schema_version': 4, 'invocations': policy.argument_rows(metadata, messages, root, 'debug', runner)}))


def argv(row: Node) -> tuple[str, ...]:
    args = row.field('argument_policy')
    middle = tuple(token for name, value in args.field('externs').mapping().items()
                   for token in ('--extern', name+'='+Node(value, '/').text()))
    return (*(n.text() for n in args.field('prefix').array()), *middle,
            *(n.text() for n in args.field('suffix').array()))


class PolicyTests(unittest.TestCase):
    def test_pure_matcher_import_has_no_setup_or_constructor_dependency(self) -> None:
        self.assertIs(policy.validate_policy, arguments.validate_policy)
        self.assertIs(policy.match_invocation, arguments.match_invocation)
        result = subprocess.run((sys.executable, '-B', '-c',
            "import sys; import generic_workspace_arguments; "
            + "assert 'generic_workspace_policy' not in sys.modules; "
            + "assert not any(n.startswith('generic_setup_') for n in sys.modules)"),
            cwd=Path(arguments.__file__).parent, capture_output=True, text=True, check=False)
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_complete_five_rows_derive_new_default_off_feature(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()/'rust'; metadata, messages, runner = fixture(root)
            result = rows(metadata, messages, root, runner)
            self.assertEqual(len(result.field('invocations').array()), 5)
            for row in result.field('invocations').array():
                self.assertIn('cfg(feature, values("new_observer"))', argv(row))
                self.assertEqual(policy.match_invocation(result, argv(row), root), row)

    def test_unknown_duplicate_filter_foreign_path_and_profile_arguments_reject(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()/'rust'; metadata, messages, runner = fixture(root)
            result = rows(metadata, messages, root, runner); row = result.field('invocations').array()[0]
            original = argv(row)
            for changed in ((*original, '--ignored'), ('@args',), tuple('foreign' if s == '--test-threads=8' else s for s in original),
                            tuple('lto=off' if s == 'embed-bitcode=no' else s for s in original),
                            tuple('/foreign' if s == str(root/'crates/sim-bridge') else s for s in original),
                            (*original[:-8], '--extern', 'sim_bridge=/foreign', *original[-8:])):
                with self.subTest(argv=changed), self.assertRaises(ValueError):
                    _ = policy.match_invocation(result, changed, root)
            with self.assertRaises(ValueError):
                _ = policy.match_invocation(result, original, root.parent)

    def test_missing_duplicate_and_foreign_compiler_identity_reject(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()/'rust'; metadata, messages, runner = fixture(root)
            for changed in (messages[1:], (*messages, messages[0])):
                with self.assertRaises(ValueError):
                    _ = rows(metadata, changed, root, runner)
            changed = messages[0].mapping().copy(); changed['package_id'] = 'registry+foreign#sim-bridge@0.7.0'
            with self.assertRaises(ValueError):
                _ = rows(metadata, (document(json.dumps(changed)), *messages[1:]), root, runner)

    def test_missing_actual_pair_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            pair = WorkspaceBuildPair(root/'chain.json', 'BUILD_ANTECEDENTS_ONLY', None, None)
            with self.assertRaises(ValueError):
                _ = policy.build_policy(pair, 'debug', (root/'python', root/'generic_cargo_child.py'))

    def test_constructor_fixture_retains_closure_and_rejects_replacement(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            parent = Path(temporary).resolve(); root = parent/'product'; root.mkdir()
            workspace = root/'rust'; metadata, messages, runner = fixture(workspace)
            output = parent/'debug'; (output/'cargo-metadata').mkdir(parents=True); (output/'cargo-build').mkdir()
            _ = (output/'cargo-metadata/stdout.log').write_text(json.dumps(metadata.value))
            _ = (output/'cargo-build/stdout.log').write_text('\n'.join(json.dumps(row.value) for row in messages))
            _ = (output/'build.json').write_text(json.dumps({'toolchain_versions': {'cargo': {'text':
                'commit-hash: '+policy.CARGO_COMMIT+'\nhost: aarch64-apple-darwin\n'}, 'rustc': {'text': 'commit-hash: '+policy.RUST_COMMIT+'\nhost: aarch64-apple-darwin\n'}}, 'environment': {'PATH': str(parent)+':/usr/bin:/bin:/usr/sbin:/sbin', 'CARGO_HOME': str(parent/'cargo-home'), 'CARGO_TARGET_DIR': str(workspace/'target'), 'RUSTC': str(parent/'rustc'), 'RUSTC_WRAPPER': '', 'RUSTC_WORKSPACE_WRAPPER': ''}}))
            _ = (output/'cargo-build/process.json').write_text(json.dumps({'environment': {'LANG': 'C', 'LC_ALL': 'C', **document((output/'build.json').read_text()).field('environment').mapping()}}))
            spec = BuildSpec(root, 'a'*40, parent/'cargo', 'b'*64, parent/'rustc', 'c'*64, parent/'cargo-home',
                output, (), (), profile='debug', workspace_tests=True)
            pair_root = parent/'pair'; pair_root.mkdir()
            pair = WorkspaceBuildPair(pair_root/'chain.json', 'BUILD_ANTECEDENTS_ONLY', None, None)
            _ = (parent/'cargo').write_bytes(b'fixture cargo')
            _ = (parent/'rustc').write_bytes(b'fixture rustc')
            proof = WorkspaceSourceProof('{}', '{}', (), 'PRISTINE_COMMIT')
            with patch.object(policy, 'recheck_workspace_pair', return_value=(spec, replace(spec, profile='release'))), \
                 patch.object(policy, 'workspace_pair_source', return_value=proof), \
                 patch.object(policy, 'recheck_workspace_source'):
                original = (output/'cargo-build/stdout.log').read_text()
                script: dict[str, Json] = {'reason': 'build-script-executed',
                    'package_id': messages[0].field('package_id').value,
                    'linked_paths': [], 'linked_libs': [], 'cfgs': [], 'env': [],
                    'out_dir': str(workspace/'target/debug/build/sim-bridge-abc/out')}
                _ = (output/'cargo-build/stdout.log').write_text(original+'\n'+json.dumps(script))
                with self.assertRaises(ValueError):
                    _ = policy.build_policy(pair, 'debug', runner)
                out = workspace/'target/debug/build/sim-bridge-abc/out'; out.mkdir(parents=True)
                _ = (out/'material_files.rs').write_text('// generated fixture')
                _ = (out.parent/'output').write_text('cargo:rustc-cfg=foreign\n')
                with self.assertRaises(ValueError):
                    _ = policy.build_policy(pair, 'debug', runner)
                _ = (out.parent/'output').write_text('cargo:rerun-if-changed=build.rs\n')
                with self.subTest('actual raw generated input'):
                    script_template = policy.build_policy(pair, 'debug', runner)
                    self.assertIn(out/'material_files.rs', dict(script_template.inputs))
                _ = (output/'cargo-build/stdout.log').write_text(original)
                template = policy.build_policy(pair, 'debug', runner)
                self.assertEqual(len(policy.recheck_policy(template).field('invocations').array()), 5)
                original_validate = policy.validate_policy
                def late(node: Node) -> None:
                    original_validate(node)
                    _ = runner[0].write_bytes(b'late callback mutation')
                original_bytes = runner[0].read_bytes()
                with patch.object(policy, 'validate_policy', side_effect=late), self.assertRaises(ValueError):
                    _ = policy.recheck_policy(template)
                _ = runner[0].write_bytes(original_bytes)
                for victim in (runner[0], workspace/'target/debug/deps'):
                    mode = victim.stat().st_mode
                    victim.chmod(0o700 if mode & 0o777 != 0o700 else 0o755)
                    with self.assertRaises(ValueError):
                        _ = policy.recheck_policy(template)
                    victim.chmod(mode)
                retained = template.retained_anchor
                assert retained is not None
                reduced = json.dumps({'schema_version': 4, 'invocations':
                    [document(template.policy_json).field('invocations').array()[0].value]}, sort_keys=True)
                forged = replace(template, policy_json=reduced, retained_anchor=replace(retained, policy_json=reduced))
                with self.assertRaises(ValueError):
                    _ = policy.recheck_policy(forged)

                with self.assertRaises(ValueError):
                    _ = policy.recheck_policy(replace(template, policy_json='{}'))
                _ = (workspace/'target/debug/deps/late-empty').mkdir()
                with self.assertRaises(ValueError):
                    _ = policy.recheck_policy(template)

    def test_actual_issuer_rejects_replaced_callback_and_policy(self) -> None:
        pair, runner = actual_tiny_pair(self)
        template = policy.build_policy(pair, 'debug', runner)
        retained = template.retained_anchor
        assert retained is not None
        reduced = json.dumps({'schema_version': 4, 'invocations':
            [document(template.policy_json).field('invocations').array()[0].value]}, sort_keys=True)
        self.assertFalse(hasattr(retained, 'issued_check'))
        forged = replace(template, policy_json=reduced, retained_anchor=replace(
            retained, policy_json=reduced))
        with self.assertRaises(ValueError):
            _ = policy.recheck_policy(forged)
        with self.assertRaises(ValueError):
            _ = policy.recheck_policy(replace(template))
        self.assertEqual(len(policy.recheck_policy(template).field('invocations').array()), 5)

    def test_release_uses_source_profile_and_compiler_paths(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()/'rust'; metadata, messages, runner = fixture(root)
            release: list[Node] = []
            for message in messages:
                changed = message.mapping().copy()
                old = Path(message.field('filenames').array()[0].text())
                new = root/'target/release/deps'/old.name; new.parent.mkdir(parents=True, exist_ok=True)
                _ = new.write_bytes(old.read_bytes()); changed['filenames'] = [str(new)]
                release.append(document(json.dumps(changed)))
            result = document(json.dumps({'schema_version': 4, 'invocations':
                policy.argument_rows(metadata, tuple(release), root, 'release', runner)}))
            for row in result.field('invocations').array():
                self.assertIn('lto=thin', argv(row))
                self.assertNotIn('embed-bitcode=no', argv(row))
                self.assertEqual(policy.match_invocation(result, argv(row), root), row)

    def test_declared_features_and_enabled_features_are_distinct(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()/'rust'; metadata, messages, runner = fixture(root)
            changed = messages[0].mapping().copy(); changed['features'] = ['new_observer']
            with self.assertRaises(ValueError):
                _ = rows(metadata, (document(json.dumps(changed)), *messages[1:]), root, runner)
            package = metadata.field('packages').array()[0]
            manifest = Path(package.field('manifest_path').text())
            _ = manifest.write_text(manifest.read_text()+'foreign=[]\n')
            with self.assertRaises(ValueError):
                _ = rows(metadata, messages, root, runner)

    def test_duplicate_obligations_and_missing_linkable_artifact_reject(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()/'rust'; metadata, messages, runner = fixture(root)
            result = rows(metadata, messages, root, runner)
            duplicate = document(json.dumps({'schema_version': 4, 'invocations':
                [row.value for row in result.field('invocations').array()]*2}))
            with self.assertRaises(ValueError):
                policy.validate_policy(duplicate)
            Path(messages[0].field('filenames').array()[0].text()).unlink()
            with self.assertRaises((ValueError, OSError)):
                _ = rows(metadata, messages, root, runner)

    def test_actual_tiny_pair_uses_real_build_source_and_receipt_authentication(self) -> None:
        pair, runner = actual_tiny_pair(self)
        debug = policy.build_policy(pair, 'debug', runner)
        release = policy.build_policy(pair, 'release', runner)
        for template in (debug, release):
            self.assertEqual(len(policy.recheck_policy(template).field('invocations').array()), 5)
            self.assertEqual(template.runtime_qualification, 'NOT_RUN')
        selected = Path(document(debug.policy_json).field('invocations').array()[0]
                        .field('argument_policy').field('externs').field('sim_bridge').text())
        _ = selected.write_bytes(b'late actual pair library mutation')
        with self.assertRaises(ValueError):
            _ = policy.recheck_policy(debug)


def actual_tiny_pair(test: unittest.TestCase) -> tuple[WorkspaceBuildPair, tuple[Path, Path]]:
    """Real generic_build/Popen/Git evidence with inert synthetic compiler tools."""
    base = build_tests.BuildContractTests()
    base.setUp()
    test.addCleanup(base.doCleanups)
    metadata, messages, old_runner = fixture(base.root/'rust')
    runner = (base.home/'python', base.home/'generic_cargo_child.py')
    for source, target in zip(old_runner, runner, strict=True):
        _ = shutil.move(source, target)
    shutil.rmtree(base.root/'rust/target')
    payload = json.dumps(metadata.value)
    emitted = json.dumps([message.value for message in messages])
    cargo = '#!'+sys.executable+'\n'+f'''
import json,os,pathlib,sys
if '--version' in sys.argv:
 print('cargo 1.93.1 synthetic fixture\\nrelease: 1.93.1\\ncommit-hash: {policy.CARGO_COMMIT}\\nhost: aarch64-apple-darwin');sys.exit(0)
metadata=json.loads({payload!r})
if sys.argv[1]=='metadata':
 print(json.dumps(metadata));sys.exit(0)
profile='release' if '--release' in sys.argv else 'debug'
for original in json.loads({emitted!r}):
 row=json.loads(json.dumps(original).replace('/target/debug/','/target/'+profile+'/'))
 for filename in row['filenames']:
  path=pathlib.Path(filename);path.parent.mkdir(parents=True,exist_ok=True);path.write_bytes(b'inert library '+profile.encode())
 if row['target']['name']=='sim_bridge':
  sibling=path.with_suffix('.dylib');sibling.write_bytes(b'inert cdylib');row['filenames'].append(str(sibling))
 row['manifest_path']=str(pathlib.Path(row['target']['src_path']).parent/'Cargo.toml')
 row['executable']=None
 print(json.dumps(row))
 binary=path.parent/(row['target']['name']+'-testfixture')
 binary.write_bytes(b'inert native fixture never executed');binary.chmod(0o755)
 row['profile']={{'test':True}};row['executable']=str(binary);row['filenames']=[str(binary)]
 print(json.dumps(row))
print(json.dumps({{'reason':'build-finished','success':True}}))
'''
    _ = base.compiler.write_text(cargo)
    rustc = base.home/'rustc'
    _ = rustc.write_text('#!'+sys.executable+'\nprint('+repr('rustc 1.93.1 synthetic fixture\nrelease: 1.93.1\ncommit-hash: '+policy.RUST_COMMIT+'\nhost: aarch64-apple-darwin')+')\n')
    rustc.chmod(0o755)
    _ = subprocess.run(['git', '-C', str(base.root), 'add', '.'], check=True, capture_output=True)
    _ = subprocess.run(['git', '-C', str(base.root), '-c', 'user.name=Test', '-c',
                        'user.email=t@example.invalid', 'commit', '-qm', 'five inert workspace targets'], check=True, capture_output=True)
    sha = subprocess.check_output(['git', '-C', str(base.root), 'rev-parse', 'HEAD'], text=True).strip()
    debug = replace(base.spec, source_sha=sha, cargo_sha256=file_hash(base.compiler), rustc=rustc,
        rustc_sha256=file_hash(rustc), packages=(), artifacts=(), workspace_tests=True,
        profile='debug', output=base.home/'debug')
    release = replace(debug, profile='release', output=base.home/'release')
    result = build_workspace_pair(debug, release, base.home/'pair')
    test.assertEqual(result.status, 'BUILD_ANTECEDENTS_ONLY', result.receipt.read_text()+(result.debug.receipt.read_text() if result.debug is not None else ''))
    return result, runner
