"""Offline build receipt tests use an explicit tiny compiler, never Cargo/worlds."""
import json
import subprocess
import sys
import tempfile
import unittest
from collections.abc import Callable
from dataclasses import replace
from pathlib import Path
from typing import final

from generic_build import Artifact, BuildSpec, Overlay, build_product
from generic_gate import document
from generic_process import file_hash


class BuildContractTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.home = Path(self.temp.name).resolve()
        self.root = self.home / 'product'
        self.root.mkdir()
        subprocess.run(['git', 'init', '-q', str(self.root)], check=True)
        for name in ('rust/crates', 'tools/v8', 'assets', 'scripts', 'scenes', 'shaders', 'localization'):
            path = self.root / name
            path.mkdir(parents=True, exist_ok=True)
            (path / 'input.txt').write_text(name)
        for name in ('rust/Cargo.toml', 'rust/Cargo.lock', 'rust/rust-toolchain.toml',
                     'project.godot', 'sim_bridge.gdextension', 'tools/play.sh'):
            (self.root / name).write_text('input')
        (self.root / 'rust/rust-toolchain.toml').write_text('[toolchain]\nchannel="1.93"\n')
        (self.root / '.gitignore').write_text('target/\n')
        subprocess.run(['git', '-C', str(self.root), 'add', '.'], check=True)
        subprocess.run(['git', '-C', str(self.root), '-c', 'user.name=Test', '-c', 'user.email=t@example.invalid',
                        'commit', '-qm', 'fixture'], check=True)
        sha = subprocess.check_output(['git', '-C', str(self.root), 'rev-parse', 'HEAD'], text=True).strip()
        self.compiler = self.home / 'compiler'
        self.compiler.write_text('#!' + sys.executable + '''
import json,os,pathlib,sys
if '--version' in sys.argv:
 print('synthetic compiler 1.93.0\\nrelease: 1.93.0\\nhost: synthetic-test-only');sys.exit(0)
target=pathlib.Path(os.environ['CARGO_TARGET_DIR'])/'release'/'worldsim_perf'
target.parent.mkdir(parents=True,exist_ok=True)
target.write_bytes(b'synthetic artifact, not executed')
target.chmod(0o755)
print(json.dumps({'reason':'compiler-artifact','target':{'name':'worldsim_perf'},'fresh':False,'executable':str(target),'filenames':[str(target)]}))
print(json.dumps({'reason':'build-finished','success':True}))
''')
        self.compiler.chmod(0o755)
        self.cargo_home = self.home / 'cargo-home'
        self.cargo_home.mkdir()
        self.spec = BuildSpec(self.root, sha, self.compiler, file_hash(self.compiler),
                              self.compiler, file_hash(self.compiler), self.cargo_home,
                              self.home / 'evidence', ('sim-test',),
                              (Artifact('worldsim_perf', 'release/worldsim_perf'),))

    def integration_spec(self):
        self.change_compiler(lambda source: source.replace(
            "target=pathlib.Path(os.environ['CARGO_TARGET_DIR'])/'release'/'worldsim_perf'",
            "assert sys.argv[1]=='test' and '--no-run' in sys.argv and '--test' in sys.argv\n"
            "target=pathlib.Path(os.environ['CARGO_TARGET_DIR'])/'release'/'deps'/'save_load-1234'"
        ).replace("'target':{'name':'worldsim_perf'}", "'target':{'name':'save_load','kind':['test']},'profile':{'test':True}"))
        self.spec = replace(self.spec, artifacts=(), test_targets=('save_load',))

    def test_integration_binary_is_selected_from_actual_compiler_message(self):
        self.integration_spec()
        result = build_product(self.spec)
        self.assertEqual(result.status, 'PASS')
        receipt = json.loads(result.receipt.read_text())
        self.assertEqual(receipt['build_kind'], 'integration_tests')
        self.assertEqual(receipt['test_targets'], ['save_load'])
        self.assertEqual(result.artifacts[0][1], self.root / 'rust/target/release/deps/save_load-1234')
        self.assertTrue((self.spec.output / 'artifact-save_load.json').is_file())

    def test_integration_non_test_target_is_rejected(self):
        self.integration_spec()
        self.change_compiler(lambda text: text.replace("'kind':['test']", "'kind':['bin']"))
        self.assertEqual(build_product(self.spec).status, 'FAIL')

    def test_integration_non_test_profile_is_rejected(self):
        self.integration_spec()
        self.change_compiler(lambda text: text.replace("'test':True", "'test':False"))
        self.assertEqual(build_product(self.spec).status, 'FAIL')

    def test_integration_foreign_executable_is_rejected(self):
        self.integration_spec()
        self.change_compiler(lambda text: text.replace("/'release'/'deps'/", "/'release'/"))
        self.assertEqual(build_product(self.spec).status, 'FAIL')

    def test_integration_missing_filename_binding_is_rejected(self):
        self.integration_spec()
        self.change_compiler(lambda text: text.replace("'filenames':[str(target)]", "'filenames':[]"))
        self.assertEqual(build_product(self.spec).status, 'FAIL')

    def test_integration_cached_binary_is_rejected(self):
        self.integration_spec()
        self.change_compiler(lambda text: text.replace("'fresh':False", "'fresh':True"))
        self.assertEqual(build_product(self.spec).status, 'FAIL')

    def test_integration_duplicate_target_is_rejected_before_build(self):
        self.integration_spec()
        with self.assertRaisesRegex(ValueError, 'duplicate'):
            build_product(replace(self.spec, test_targets=('save_load', 'save_load')))
        self.assertFalse(self.spec.output.exists())

    def test_integration_cannot_mix_normal_artifacts(self):
        with self.assertRaisesRegex(ValueError, 'separate builds'):
            build_product(replace(self.spec, test_targets=('save_load',)))
        self.assertFalse(self.spec.output.exists())

    def test_fresh_build_receipt_binds_actual_artifact(self):
        result = build_product(self.spec)
        self.assertEqual(result.status, 'PASS')
        receipt = json.loads(result.receipt.read_text())
        self.assertEqual(receipt['source_sha'], self.spec.source_sha)
        self.assertEqual(receipt['binary_sha256'], file_hash(self.root / 'rust/target/release/worldsim_perf'))
        self.assertEqual(receipt['source_kind'], 'PRISTINE_COMMIT')

    def test_stale_target_is_rejected_before_compiler(self):
        (self.root / 'rust/target').mkdir()
        with self.assertRaises(ValueError):
            build_product(self.spec)

    def change_compiler(self, transform: Callable[[str], str]) -> None:
        self.compiler.write_text(transform(self.compiler.read_text()))
        self.spec = replace(self.spec, cargo_sha256=file_hash(self.compiler), rustc_sha256=file_hash(self.compiler))

    def test_nonzero_build_retains_native_exit(self):
        self.change_compiler(lambda text: text.replace("target=pathlib", "sys.exit(73)\ntarget=pathlib"))
        result = build_product(self.spec)
        self.assertEqual(result.status, 'FAIL')
        self.assertEqual(json.loads(result.receipt.read_text())['exit_code'], 73)
        self.assertFalse((self.spec.output / 'artifact-worldsim_perf.json').exists())

    def test_missing_output_fails_despite_success_terminal(self):
        self.change_compiler(lambda text: text.replace("target.write_bytes(b'synthetic artifact, not executed')", "pass").replace('target.chmod(0o755)', 'pass'))
        result = build_product(self.spec)
        self.assertEqual(result.status, 'FAIL')

    def test_cargo_fresh_artifact_is_not_an_independent_build(self):
        self.change_compiler(lambda text: text.replace("'fresh':False", "'fresh':True"))
        result = build_product(self.spec)
        self.assertEqual(result.status, 'FAIL')
        self.assertIn('expected False', json.loads(result.receipt.read_text())['error'])

    def test_wrong_head_rejects_before_build(self):
        with self.assertRaises(ValueError):
            build_product(replace(self.spec, source_sha='b' * 40))

    def test_dirty_source_rejects_before_build(self):
        (self.root / 'rust/crates/input.txt').write_text('changed')
        with self.assertRaises(ValueError):
            build_product(self.spec)

    def test_build_that_mutates_source_fails(self):
        self.change_compiler(lambda text: text.replace("target=pathlib", "pathlib.Path('crates/input.txt').write_text('mutated')\ntarget=pathlib"))
        result = build_product(self.spec)
        self.assertEqual(result.status, 'FAIL')
        self.assertIn('inputs changed', (self.spec.output / 'cargo-build/process.json').read_text())

    def test_unbound_cargo_config_fails(self):
        (self.cargo_home / 'config.toml').write_text('[build]\njobs=2\n')
        with self.assertRaisesRegex(ValueError, 'unbound'):
            build_product(self.spec)

    def test_bound_cargo_config_is_recorded(self):
        config = self.cargo_home / 'config.toml'
        config.write_text('[build]\njobs=2\n')
        result = build_product(replace(self.spec, config_inputs=((str(config), file_hash(config)),)))
        self.assertEqual(result.status, 'PASS')

    def make_overlay(self, name='tools/v8/input.txt'):
        path = self.root / name
        before = file_hash(path)
        original = path.read_text()
        path.write_text('observer-derived')
        after = file_hash(path)
        patch = subprocess.check_output(['git', '-C', str(self.root), 'diff', '--', name])
        path.write_text(original)
        patch_file = self.home / 'overlay.patch'
        patch_file.write_bytes(patch)
        manifest = self.home / 'overlay.json'
        manifest.write_text(json.dumps({'schema_version': 1, 'product_base_sha': self.spec.source_sha,
            'observer_patch_sha256': file_hash(patch_file), 'allowed_paths': [name], 'files': [
            {'path': name, 'base0_sha256': before, 'observer_sha256': after, 'operation': 'modify'}]}))
        return Overlay(patch_file, file_hash(patch_file), manifest, file_hash(manifest))

    def test_observer_overlay_is_explicitly_derived_not_pristine(self):
        overlay = self.make_overlay()
        result = build_product(replace(self.spec, overlay=overlay))
        self.assertEqual(result.status, 'PASS')
        receipt = json.loads(result.receipt.read_text())
        self.assertEqual(receipt['source_kind'], 'DERIVED_OBSERVER')
        self.assertEqual(receipt['observer_semantic_qualification'], 'NOT_RUN')
        self.assertEqual((self.root / 'tools/v8/input.txt').read_text(), 'observer-derived')

    def test_wrong_overlay_digest_fails_without_apply(self):
        overlay = replace(self.make_overlay(), patch_sha256='f' * 64)
        result = build_product(replace(self.spec, overlay=overlay))
        self.assertEqual(result.status, 'FAIL')
        self.assertEqual((self.root / 'tools/v8/input.txt').read_text(), 'tools/v8')

    def test_overlay_extra_unlisted_mutation_fails(self):
        overlay = self.make_overlay()
        other = self.root / 'tools/v8/extra.txt'
        other.write_text('unlisted')
        extra = subprocess.run(['git', 'diff', '--no-index', '/dev/null', str(other)], check=False, capture_output=True).stdout
        other.unlink()
        # A patch may not smuggle another new file beyond the approved manifest.
        extra = extra.replace(str(other).encode(), b'tools/v8/extra.txt')
        overlay.patch.write_bytes(overlay.patch.read_bytes() + extra)
        data = json.loads(overlay.manifest.read_text())
        data['observer_patch_sha256'] = file_hash(overlay.patch)
        overlay.manifest.write_text(json.dumps(data))
        overlay = replace(overlay, patch_sha256=file_hash(overlay.patch), manifest_sha256=file_hash(overlay.manifest))
        result = build_product(replace(self.spec, overlay=overlay))
        self.assertEqual(result.status, 'FAIL')

    def test_artifact_traversal_rejected(self):
        with self.assertRaises(ValueError):
            build_product(replace(self.spec, artifacts=(Artifact('escape', 'release/../escape'),)))

    def test_wrong_toolchain_version_fails(self):
        self.change_compiler(lambda text: text.replace('release: 1.93.0', 'release: 1.94.0'))
        result = build_product(self.spec)
        self.assertEqual(result.status, 'FAIL')
        self.assertIn('pinned source toolchain', json.loads(result.receipt.read_text())['error'])
        self.assertFalse((self.root / 'rust/target').exists())

    def test_authorized_bootstrap_overlay_passes(self):
        name = 'scripts/core/main_bootstrap.gd'
        (self.root / name).parent.mkdir(parents=True, exist_ok=True)
        (self.root / name).write_text('bootstrap')
        subprocess.run(['git', '-C', str(self.root), 'add', name], check=True)
        subprocess.run(['git', '-C', str(self.root), '-c', 'user.name=Test', '-c', 'user.email=t@example.invalid',
                        'commit', '-qm', 'bootstrap fixture'], check=True)
        sha = subprocess.check_output(['git', '-C', str(self.root), 'rev-parse', 'HEAD'], text=True).strip()
        self.spec = replace(self.spec, source_sha=sha)
        result = build_product(replace(self.spec, overlay=self.make_overlay(name)))
        self.assertEqual(result.status, 'PASS')

    def test_authorized_test_observer_overlay_passes(self):
        name = 'rust/crates/sim-test/tests/save_load.rs'
        (self.root / name).parent.mkdir(parents=True, exist_ok=True)
        (self.root / name).write_text('test observer')
        subprocess.run(['git', '-C', str(self.root), 'add', name], check=True)
        subprocess.run(['git', '-C', str(self.root), '-c', 'user.name=Test', '-c', 'user.email=t@example.invalid',
                        'commit', '-qm', 'test observer fixture'], check=True)
        sha = subprocess.check_output(['git', '-C', str(self.root), 'rev-parse', 'HEAD'], text=True).strip()
        self.spec = replace(self.spec, source_sha=sha)
        result = build_product(replace(self.spec, overlay=self.make_overlay(name)))
        self.assertEqual(result.status, 'PASS')

    def test_unreviewed_script_overlay_still_rejected(self):
        name = 'scripts/world/ordinary_gameplay.gd'
        (self.root / name).parent.mkdir(parents=True, exist_ok=True)
        (self.root / name).write_text('unchanged gameplay')
        subprocess.run(['git', '-C', str(self.root), 'add', name], check=True)
        subprocess.run(['git', '-C', str(self.root), '-c', 'user.name=Test', '-c', 'user.email=t@example.invalid',
                        'commit', '-qm', 'gameplay fixture'], check=True)
        sha = subprocess.check_output(['git', '-C', str(self.root), 'rev-parse', 'HEAD'], text=True).strip()
        self.spec = replace(self.spec, source_sha=sha)
        result = build_product(replace(self.spec, overlay=self.make_overlay(name)))
        self.assertEqual(result.status, 'FAIL')
        self.assertIn('outside tooling/bridge scope', json.loads(result.receipt.read_text())['error'])
        self.assertEqual((self.root / name).read_text(), 'unchanged gameplay')

    def test_effective_inventory_includes_named_helpers_and_legacy_tools(self):
        helpers = ('rust/crates/sim-test/tests/save_gate_observation/mod.rs',
                   'tools/v8/gate_legacy_observer.gd', 'tools/v8/generic_legacy_overlay.py',
                   'scripts/core/main_bootstrap.gd')
        for name in helpers:
            path = self.root / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text('input ' + name)
        subprocess.run(['git', '-C', str(self.root), 'add', *helpers], check=True)
        subprocess.run(['git', '-C', str(self.root), '-c', 'user.name=Test', '-c', 'user.email=t@example.invalid',
                        'commit', '-qm', 'effective input fixtures'], check=True)
        sha = subprocess.check_output(['git', '-C', str(self.root), 'rev-parse', 'HEAD'], text=True).strip()
        result = build_product(replace(self.spec, source_sha=sha))
        self.assertEqual(result.status, 'PASS')
        inputs = json.loads(result.receipt.read_text())['effective_inputs_before']
        for name in helpers:
            self.assertEqual(inputs[name], file_hash(self.root / name))

    def test_new_named_observer_helper_is_bound(self):
        name = 'rust/crates/sim-test/tests/save_gate_observation/mod.rs'
        path = self.root / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text('new observer helper')
        after = file_hash(path)
        patch = subprocess.run(['git', 'diff', '--no-index', '/dev/null', name], cwd=self.root,
                               capture_output=True, check=False)
        self.assertEqual(patch.returncode, 1)
        path.unlink()
        patch_file = self.home / 'observer-add.patch'
        patch_file.write_bytes(patch.stdout)
        manifest = self.home / 'observer-add.json'
        manifest.write_text(json.dumps({'schema_version': 1, 'product_base_sha': self.spec.source_sha,
            'observer_patch_sha256': file_hash(patch_file), 'allowed_paths': [name], 'files': [
            {'path': name, 'base0_sha256': None, 'observer_sha256': after, 'operation': 'add'}]}))
        overlay = Overlay(patch_file, file_hash(patch_file), manifest, file_hash(manifest))
        result = build_product(replace(self.spec, overlay=overlay))
        self.assertEqual(result.status, 'PASS')
        self.assertEqual(json.loads(result.receipt.read_text())['effective_inputs_before'][name], after)

    def test_unreviewed_test_file_is_not_blanket_admitted(self):
        name = 'rust/crates/sim-test/tests/unrelated.rs'
        path = self.root / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text('unrelated test')
        subprocess.run(['git', '-C', str(self.root), 'add', name], check=True)
        subprocess.run(['git', '-C', str(self.root), '-c', 'user.name=Test', '-c', 'user.email=t@example.invalid',
                        'commit', '-qm', 'unrelated test fixture'], check=True)
        sha = subprocess.check_output(['git', '-C', str(self.root), 'rev-parse', 'HEAD'], text=True).strip()
        self.spec = replace(self.spec, source_sha=sha)
        result = build_product(replace(self.spec, overlay=self.make_overlay(name)))
        self.assertEqual(result.status, 'FAIL')
        self.assertEqual(path.read_text(), 'unrelated test')

    def test_reviewed_named_paths_build_with_overlay(self):
        names = (
            'rust/crates/sim-test/tests/colony_steward.rs',
            'rust/crates/sim-test/tests/colony_steward_housing/construction.rs',
            'rust/crates/sim-test/tests/steward_defence.rs',
            'rust/crates/sim-test/tests/legacy_migration_current.rs',
            'rust/crates/sim-test/tests/steward_defence_migration.rs',
            'rust/crates/sim-test/tests/b5_flora.rs',
            'rust/crates/sim-test/tests/engine_appearance_ffi.rs',
            'rust/crates/sim-test/tests/engine_ascii.rs',
            'rust/crates/sim-test/tests/engine_knowledge_ffi.rs',
            'rust/crates/sim-test/tests/engine_colony_ffi.rs',
            'rust/crates/sim-test/tests/m6b_presentation_defaults.rs',
            'rust/crates/sim-test/tests/save_gate_observation/external.rs',
        )
        for name in names:
            with self.subTest(path=name):
                fixture = BuildContractTests()
                fixture.setUp()
                try:
                    path = fixture.root / name
                    path.parent.mkdir(parents=True, exist_ok=True)
                    path.write_text('reviewed observer seam')
                    subprocess.run(['git', '-C', str(fixture.root), 'add', name], check=True)
                    subprocess.run(['git', '-C', str(fixture.root), '-c', 'user.name=Test', '-c',
                                    'user.email=t@example.invalid', 'commit', '-qm', 'reviewed seam'], check=True)
                    sha = subprocess.check_output(['git', '-C', str(fixture.root), 'rev-parse', 'HEAD'], text=True).strip()
                    fixture.spec = replace(fixture.spec, source_sha=sha)
                    result = build_product(replace(fixture.spec, overlay=fixture.make_overlay(name)))
                    self.assertEqual(result.status, 'PASS')
                finally:
                    fixture.doCleanups()


@final
class WorkspaceBuildTests(BuildContractTests):
    def workspace_spec(self) -> None:
        for package in ('alpha','beta'):
            for name in ('Cargo.toml','src/lib.rs'):
                path = self.root/'rust/crates'/package/name
                path.parent.mkdir(parents=True,exist_ok=True)
                _ = path.write_text('synthetic '+name)
        _ = subprocess.run(['git','-C',str(self.root),'add','.'],check=True)
        _ = subprocess.run(['git','-C',str(self.root),'-c','user.name=Test','-c','user.email=t@example.invalid',
                            'commit','-qm','synthetic workspace inputs'],check=True)
        sha = subprocess.check_output(['git','-C',str(self.root),'rev-parse','HEAD'],text=True).strip()
        _ = self.compiler.write_text('#!'+sys.executable+WORKSPACE_COMPILER)
        self.compiler.chmod(0o755)
        self.spec = replace(self.spec,source_sha=sha,cargo_sha256=file_hash(self.compiler),
                            rustc_sha256=file_hash(self.compiler),packages=(),artifacts=(),workspace_tests=True)

    def test_workspace_metadata_and_package_qualified_artifacts(self) -> None:
        self.workspace_spec()
        result = build_product(self.spec)
        self.assertEqual(result.status,'PASS')
        receipt = document(result.receipt.read_text())
        receipt.field('build_kind').equals('workspace_tests')
        receipt.field('packages').equals([])
        self.assertEqual([row.text() for row in receipt.field('argv').array()][1:],
            ['test','--locked','--message-format=json-render-diagnostics','--workspace','--no-run','--release'])
        self.assertEqual(len(result.artifacts),2)
        self.assertEqual(len({name for name,_,_ in result.artifacts}),2)
        receipt.field('workspace_runtime').equals('NOT_RUN')
        receipt.field('workspace_world_safety').equals('INCOMPLETE')
        self.assertEqual(len(receipt.field('workspace_doctests').array()),2)
        for name,_,_ in result.artifacts:
            artifact = document((self.spec.output/f'artifact-{name}.json').read_text())
            artifact.field('workspace_target').equals(receipt.field('workspace_targets').field(name).value)
            artifact.field('metadata_receipt_sha256').equals(file_hash(self.spec.output/'cargo-metadata/process.json'))

    def test_workspace_debug_has_separate_profile_artifacts(self) -> None:
        self.workspace_spec()
        result = build_product(replace(self.spec,profile='debug'))
        self.assertEqual(result.status,'PASS')
        self.assertTrue(all(path.parent == self.root/'rust/target/debug/deps' for _,path,_ in result.artifacts))
        self.assertNotIn('--release',[row.text() for row in document(result.receipt.read_text()).field('argv').array()])

    def test_workspace_selectors_cannot_narrow_inventory(self) -> None:
        self.workspace_spec()
        for changed in (replace(self.spec,packages=('alpha',)),replace(self.spec,features=('extra',)),
                        replace(self.spec,test_targets=('one',)),replace(self.spec,artifacts=(Artifact('bin','release/bin'),))):
            with self.subTest(spec=changed), self.assertRaises(ValueError):
                _ = build_product(changed)
        self.assertFalse(self.spec.output.exists())

    def test_workspace_missing_artifact_retains_failure_without_artifact_receipts(self) -> None:
        self.workspace_spec()
        self.change_compiler(lambda text:text.replace('for package in packages:', 'for package in packages[:1]:'))
        result = build_product(self.spec)
        self.assertEqual(result.status,'FAIL')
        self.assertIn('missing selected',document(result.receipt.read_text()).field('error').text())
        self.assertEqual(list(self.spec.output.glob('artifact-*.json')),[])

    def test_workspace_metadata_failure_retains_real_exit(self) -> None:
        self.workspace_spec()
        self.change_compiler(lambda text:text.replace("if sys.argv[1]=='metadata':", "if sys.argv[1]=='metadata':\n sys.exit(19)"))
        result = build_product(self.spec)
        self.assertEqual(result.status,'FAIL')
        document(result.receipt.read_text()).field('metadata_exit_code').equals(19)
        self.assertFalse((self.spec.output/'cargo-build').exists())

    def test_workspace_foreign_metadata_source_rejected_before_build(self) -> None:
        self.workspace_spec()
        self.change_compiler(lambda text:text.replace("str(package/'src/lib.rs')", "'/tmp/foreign-source.rs'"))
        result = build_product(self.spec)
        self.assertEqual(result.status,'FAIL')
        self.assertFalse((self.spec.output/'cargo-build').exists())

    def test_workspace_build_cannot_rewrite_metadata_receipt(self) -> None:
        self.workspace_spec()
        victim = self.spec.output/'cargo-metadata/stdout.log'
        self.change_compiler(lambda text:text.replace("assert '--workspace'", f"pathlib.Path({str(victim)!r}).write_text('rewritten')\nassert '--workspace'"))
        result = build_product(self.spec)
        self.assertEqual(result.status,'FAIL')
        self.assertIn('earlier metadata',document(result.receipt.read_text()).field('error').text())

    def test_workspace_metadata_cannot_rewrite_prior_version_receipt(self) -> None:
        self.workspace_spec()
        victim = self.spec.output/'cargo-version/stdout.log'
        self.change_compiler(lambda text:text.replace("if sys.argv[1]=='metadata':", "if sys.argv[1]=='metadata':\n pathlib.Path("+repr(str(victim))+').write_text("rewritten")'))
        result = build_product(self.spec)
        self.assertEqual(result.status,'FAIL')
        self.assertIn('earlier metadata',document(result.receipt.read_text()).field('error').text())

    def test_workspace_conditional_selection_fails_without_feature_resolution(self) -> None:
        self.workspace_spec()
        self.change_compiler(lambda text:text.replace("'test':True,'doctest':True", "'test':True,'doctest':True,'required-features':['extra']"))
        result = build_product(self.spec)
        self.assertEqual(result.status,'FAIL')
        self.assertIn('conditional target',document(result.receipt.read_text()).field('error').text())


WORKSPACE_COMPILER = r"""
import json,os,pathlib,sys
if '--version' in sys.argv:
 print('synthetic compiler 1.93.0\nrelease: 1.93.0\nhost: synthetic-test-only');sys.exit(0)
root=pathlib.Path.cwd();target=pathlib.Path(os.environ['CARGO_TARGET_DIR'])
packages=[]
for name in ('alpha','beta'):
 package=root/'crates'/name
 packages.append({'id':'path+'+str(package),'name':name,'manifest_path':str(package/'Cargo.toml'),
   'targets':[{'name':'shared','kind':['lib'],'src_path':str(package/'src/lib.rs'),'test':True,'doctest':True}]})
if sys.argv[1]=='metadata':
 assert sys.argv[2:]==['--locked','--format-version=1','--no-deps']
 print(json.dumps({'version':1,'workspace_root':str(root),'target_directory':str(target),
  'workspace_members':[p['id'] for p in packages],'packages':packages}));sys.exit(0)
assert '--workspace' in sys.argv and '--no-run' in sys.argv
for package in packages:
 binary=target/('release' if '--release' in sys.argv else 'debug')/'deps'/(package['name']+'-hash')
 binary.parent.mkdir(parents=True,exist_ok=True);binary.write_bytes(b'synthetic only');binary.chmod(0o755)
 print(json.dumps({'reason':'compiler-artifact','package_id':package['id'],'manifest_path':package['manifest_path'],
   'target':package['targets'][0],'profile':{'test':True},'fresh':False,'executable':str(binary),'filenames':[str(binary)]}))
print(json.dumps({'reason':'build-finished','success':True}))
"""
