"""Real tiny compiler subprocesses; no Cargo, native workloads or qualification."""
import unittest
from dataclasses import replace
from pathlib import Path
from typing import final, override
from unittest.mock import patch

import generic_build as build
from generic_gate import document
from generic_process import ProcessResult, ProcessSpec, atomic_json, execute
from mux2_gate_inputs import Json
from test_generic_build import WorkspaceBuildTests


@final
class WorkspaceChainTests(unittest.TestCase):
    def __init__(self, methodName: str = 'runTest') -> None:
        super().__init__(methodName)
        self.fixture = WorkspaceBuildTests()

    @override
    def setUp(self) -> None:
        self.fixture.setUp()
        self.addCleanup(self.fixture.doCleanups)
        self.fixture.workspace_spec()

    @property
    def debug_spec(self) -> build.BuildSpec:
        return replace(self.fixture.spec, profile='debug', output=self.fixture.home/'debug-evidence')

    @property
    def release(self) -> build.BuildSpec:
        return replace(self.fixture.spec, output=self.fixture.home/'release-evidence')

    @property
    def output(self) -> Path:
        return self.fixture.home/'chain'

    def test_two_actual_profiles_keep_original_fresh_api(self) -> None:
        result = build.build_workspace_pair(self.debug_spec, self.release, self.output)
        self.assertEqual(result.status, 'BUILD_ANTECEDENTS_ONLY')
        self.assertIsNotNone(result.release)
        for spec in (self.debug_spec, self.release):
            row = document((spec.output/'build.json').read_text())
            row.field('status').equals('PASS')
            row.field('profile').equals(spec.profile)
            self.assertTrue((spec.output/'cargo-build/process.json').is_file())
        with self.assertRaisesRegex(ValueError, 'fresh independent target'):
            _ = build.build_product(replace(self.release, output=self.fixture.home/'third'))

    def test_foreign_second_spec_rejected_before_child(self) -> None:
        for changed in (replace(self.release, source_sha='a'*40), replace(self.release, profile='debug'),
                        replace(self.release, cargo_home=self.fixture.home), replace(self.release, output=self.debug_spec.output)):
            with self.subTest(spec=changed), self.assertRaises(ValueError):
                _ = build.build_workspace_pair(self.debug_spec, changed, self.output)
        self.assertFalse(self.debug_spec.output.exists())

    def test_prior_failure_preserved_release_not_run(self) -> None:
        self.fixture.change_compiler(lambda text: text.replace("if sys.argv[1]=='metadata':", "if sys.argv[1]=='metadata':\n sys.exit(19)"))
        result = build.build_workspace_pair(self.debug_spec, self.release, self.output)
        self.assertEqual(result.status, 'FAIL')
        self.assertIsNotNone(result.debug)
        self.assertIsNone(result.release)
        self.assertFalse(self.release.output.exists())
        document((self.debug_spec.output/'build.json').read_text()).field('metadata_exit_code').equals(19)

    def test_mutation_by_later_child_fails_without_refresh(self) -> None:
        victim = self.debug_spec.output/'cargo-build/stdout.log'
        def run(spec: ProcessSpec) -> ProcessResult:
            result = execute(spec)
            if spec.output_root == self.release.output/'cargo-version':
                _ = victim.write_text('changed previous actual compiler receipt')
            return result
        with patch('generic_build.execute', side_effect=run):
            result = build.build_workspace_pair(self.debug_spec, self.release, self.output)
        self.assertEqual(result.status, 'FAIL')
        self.assertFalse((self.release.output/'cargo-build').exists())

    def test_second_producer_cannot_replace_debug_bytes(self) -> None:
        def run(spec: ProcessSpec) -> ProcessResult:
            result = execute(spec)
            if spec.output_root == self.release.output/'cargo-build':
                _ = (self.debug_spec.root/'rust/target/debug/deps/alpha-hash').write_bytes(b'changed')
            return result
        with patch('generic_build.execute', side_effect=run):
            result = build.build_workspace_pair(self.debug_spec, self.release, self.output)
        self.assertEqual(result.status, 'FAIL')
        self.assertIsNotNone(result.release)
        document((self.release.output/'build.json').read_text()).field('status').equals('FAIL')

    def test_empty_member_added_during_release_preflight_rejects(self) -> None:
        def run(spec: ProcessSpec) -> ProcessResult:
            result = execute(spec)
            if spec.output_root == self.release.output/'cargo-metadata':
                (self.debug_spec.root/'rust/target/unexpected-empty').mkdir()
            return result
        with patch('generic_build.execute', side_effect=run):
            result = build.build_workspace_pair(self.debug_spec, self.release, self.output)
        self.assertEqual(result.status, 'FAIL')
        self.assertFalse((self.release.output/'cargo-build').exists())

    def test_terminal_callback_cannot_modify_protected_output(self) -> None:
        original = atomic_json
        def publish(path: Path, value: dict[str, Json]) -> None:
            original(path, value)
            if path == self.output/'chain.json' and value.get('status') == 'BUILD_ANTECEDENTS_ONLY':
                (self.debug_spec.root/'rust/target/debug/new-empty').mkdir()
        with patch('generic_build.atomic_json', side_effect=publish):
            result = build.build_workspace_pair(self.debug_spec, self.release, self.output)
        self.assertEqual(result.status, 'FAIL')
        document(result.receipt.read_text()).field('status').equals('FAIL')

    def test_debug_return_publication_cannot_admit_new_baseline(self) -> None:
        original = atomic_json
        def publish(path: Path, value: dict[str, Json]) -> None:
            original(path, value)
            if path == self.output/'debug-returned.json':
                (self.debug_spec.root/'rust/target/late-empty').mkdir()
        with patch('generic_build.atomic_json', side_effect=publish):
            result = build.build_workspace_pair(self.debug_spec, self.release, self.output)
        self.assertEqual(result.status, 'FAIL')
        self.assertFalse(self.release.output.exists())

    def test_stale_target_never_reused_for_first_profile(self) -> None:
        (self.debug_spec.root/'rust/target').mkdir()
        result = build.build_workspace_pair(self.debug_spec, self.release, self.output)
        self.assertEqual(result.status, 'FAIL')
        self.assertFalse(self.debug_spec.output.exists())
        self.assertFalse(self.release.output.exists())

    def test_later_child_cannot_change_source_or_add_config(self) -> None:
        for kind in ('source', 'config'):
            with self.subTest(kind=kind):
                fixture = WorkspaceChainTests()
                fixture.setUp()
                try:
                    def run(spec: ProcessSpec, fixture: WorkspaceChainTests = fixture, kind: str = kind) -> ProcessResult:
                        returned = execute(spec)
                        if spec.output_root == fixture.release.output/'cargo-version':
                            victim = (fixture.debug_spec.root/'rust/crates/alpha/src/lib.rs' if kind == 'source'
                                      else fixture.debug_spec.cargo_home/'config.toml')
                            _ = victim.write_text('changed')
                        return returned
                    with patch('generic_build.execute', side_effect=run):
                        result = build.build_workspace_pair(fixture.debug_spec, fixture.release, fixture.output)
                    self.assertEqual(result.status, 'FAIL')
                    self.assertFalse((fixture.release.output/'rustc-version').exists())
                finally:
                    fixture.doCleanups()

    def test_unexpected_control_preserves_incomplete_without_release(self) -> None:
        marker = KeyboardInterrupt('retained')
        with patch('generic_build.execute', side_effect=marker), self.assertRaises(KeyboardInterrupt) as caught:
            _ = build.build_workspace_pair(self.debug_spec, self.release, self.output)
        self.assertIs(caught.exception, marker)
        document((self.output/'started.json').read_text()).field('status').equals('INCOMPLETE')
        self.assertFalse((self.output/'chain.json').exists())
        self.assertFalse(self.release.output.exists())

    def test_producer_publications_cannot_rebase_evidence_or_target(self) -> None:
        for profile in ('debug', 'release'):
            for boundary in ('build.json', 'artifact'):
                for change in ('stdout', 'empty-directory', 'mode'):
                    with self.subTest(profile=profile, boundary=boundary, change=change):
                        fixture = WorkspaceChainTests()
                        fixture.setUp()
                        try:
                            selected = fixture.debug_spec if profile == 'debug' else fixture.release
                            fired = False
                            def publish(path: Path, value: dict[str, Json], selected: build.BuildSpec = selected,
                                        boundary: str = boundary, change: str = change) -> None:
                                nonlocal fired
                                atomic_json(path, value)
                                matches = path.name == boundary or (boundary == 'artifact' and path.name.startswith('artifact-'))
                                if path.parent == selected.output and matches and not fired:
                                    fired = True
                                    if change == 'stdout':
                                        _ = (selected.output/'cargo-build/stdout.log').write_text('publication drift')
                                    elif change == 'empty-directory':
                                        (selected.root/'rust/target/undeclared-empty').mkdir()
                                    else:
                                        (selected.root/'rust/target'/selected.profile/'deps/alpha-hash').chmod(0o644)
                            with patch('generic_build.atomic_json', side_effect=publish):
                                result = build.build_workspace_pair(fixture.debug_spec, fixture.release, fixture.output)
                            self.assertTrue(fired)
                            self.assertEqual(result.status, 'FAIL')
                            if profile == 'debug':
                                self.assertFalse(fixture.release.output.exists())
                        finally:
                            fixture.doCleanups()

    def test_actual_process_hash_cannot_be_replaced_by_later_inventory(self) -> None:
        victim = self.debug_spec.output/'cargo-version/stdout.log'
        def publish(path: Path, value: dict[str, Json]) -> None:
            atomic_json(path, value)
            if path == self.debug_spec.output/'cargo-version/process.json':
                _ = victim.write_text(victim.read_text()+'extra diagnostic\n')
        with patch('generic_process.atomic_json', side_effect=publish):
            result = build.build_workspace_pair(self.debug_spec, self.release, self.output)
        self.assertEqual(result.status, 'FAIL')
        self.assertFalse(self.release.output.exists())

    def test_artifact_sidecar_content_is_predeclared(self) -> None:
        def publish(path: Path, value: dict[str, Json]) -> None:
            atomic_json(path, value)
            if path.parent == self.debug_spec.output and path.name.startswith('artifact-'):
                _ = path.write_text('{}\n')
        with patch('generic_build.atomic_json', side_effect=publish):
            result = build.build_workspace_pair(self.debug_spec, self.release, self.output)
        self.assertEqual(result.status, 'FAIL')
        self.assertFalse(self.release.output.exists())

    def test_chain_publication_preserves_exact_record_and_members(self) -> None:
        for change in ('content', 'empty-directory', 'mode', 'prior-content'):
            with self.subTest(change=change):
                fixture = WorkspaceChainTests()
                fixture.setUp()
                try:
                    def publish(path: Path, value: dict[str, Json], change: str = change,
                                fixture: WorkspaceChainTests = fixture) -> None:
                        atomic_json(path, value)
                        if path == fixture.output/'chain.json':
                            if change == 'content':
                                _ = path.write_text('{}\n')
                            elif change == 'empty-directory':
                                (fixture.output/'late-empty').mkdir()
                            elif change == 'mode':
                                path.chmod(path.stat().st_mode ^ 0o100)
                            else:
                                _ = (fixture.output/'started.json').write_text('{}\n')
                    with patch('generic_build.atomic_json', side_effect=publish):
                        result = build.build_workspace_pair(fixture.debug_spec, fixture.release, fixture.output)
                    self.assertEqual(result.status, 'FAIL')
                    self.assertEqual(result.receipt.name, 'chain-failure.json')
                    self.assertTrue((fixture.output/'chain.json').exists())
                finally:
                    fixture.doCleanups()

    def test_live_pair_recheck_rejects_late_drift(self) -> None:
        result = build.build_workspace_pair(self.debug_spec, self.release, self.output)
        _ = build.recheck_workspace_pair(result)
        victims = (self.output/'chain.json', self.debug_spec.output/'cargo-build/stdout.log',
                   self.release.output/'cargo-build/stdout.log',
                   self.debug_spec.root/'rust/target/debug/deps/alpha-hash',
                   self.debug_spec.root/'rust/target/release/deps/alpha-hash',
                   self.debug_spec.root/'rust/crates/alpha/src/lib.rs', self.debug_spec.cargo)
        for victim in victims:
            with self.subTest(path=victim):
                before = victim.read_bytes()
                try:
                    _ = victim.write_bytes(before+b'late drift')
                    with self.assertRaises(ValueError):
                        _ = build.recheck_workspace_pair(result)
                finally:
                    _ = victim.write_bytes(before)
                _ = build.recheck_workspace_pair(result)
        for parent in (self.output, self.debug_spec.output, self.release.output, self.debug_spec.root/'rust/target/release'):
            extra = parent/'late-empty'
            extra.mkdir()
            with self.assertRaises(ValueError):
                _ = build.recheck_workspace_pair(result)
            extra.rmdir()
            _ = build.recheck_workspace_pair(result)
        config = self.debug_spec.cargo_home/'config.toml'
        _ = config.write_text('[build]\njobs=8\n')
        with self.assertRaises(ValueError):
            _ = build.recheck_workspace_pair(result)
        config.unlink()
        self.assertEqual(build.recheck_workspace_pair(result), (self.debug_spec, self.release))

    def test_reconstructed_pair_does_not_confer_live_anchor(self) -> None:
        result = build.build_workspace_pair(self.debug_spec, self.release, self.output)
        copied = build.WorkspaceBuildPair(result.receipt, result.status, result.debug, result.release)
        with self.assertRaises(ValueError):
            _ = build.recheck_workspace_pair(copied)
