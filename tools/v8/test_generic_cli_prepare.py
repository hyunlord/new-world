"""Real tiny Python compiler/Git fixtures; never Cargo or game workloads."""
import shutil
import subprocess
import unittest
from dataclasses import replace
from pathlib import Path
from typing import final, override
from unittest.mock import patch

import generic_build as build
import generic_cli_prepare as cli
import test_generic_build as fixtures
from generic_gate import document
from generic_process import ProcessResult, ProcessSpec, execute, file_hash


@final
class CliPrepareTests(unittest.TestCase):
    def __init__(self, methodName: str = 'runTest') -> None:
        super().__init__(methodName)
        self.fixture = fixtures.WorkspaceBuildTests()
        self.selection: cli.CliSelectors | None = None

    @override
    def setUp(self) -> None:
        fixture = self.fixture
        fixture.setUp(); self.addCleanup(fixture.doCleanups); fixture.workspace_spec()
        registry = Path(build.__file__).with_name('gate_fixtures.json')
        _ = shutil.copy2(registry, fixture.root/'tools/v8/gate_fixtures.json')
        _ = subprocess.run(('git','-C',str(fixture.root),'rm','--cached','rust/Cargo.lock'),check=True,capture_output=True)
        _ = (fixture.root/'.gitignore').write_text('target/\nrust/Cargo.lock\n')
        _ = subprocess.run(('git','-C',str(fixture.root),'add','.'),check=True,capture_output=True)
        _ = subprocess.run(('git','-C',str(fixture.root),'-c','user.name=Test','-c','user.email=t@example.invalid','commit','-qm','registry'),check=True,capture_output=True)
        sha = subprocess.check_output(('git','-C',str(fixture.root),'rev-parse','HEAD'),text=True).strip()
        fixture.spec = replace(fixture.spec,source_sha=sha)
        candidate, harness = fixture.home/'candidate', fixture.home/'harness'
        for root in (candidate,harness):
            _ = subprocess.run(('git','clone','-q',str(fixture.root),str(root)),check=True,capture_output=True)
            _ = shutil.copy2(fixture.root/'rust/Cargo.lock',root/'rust/Cargo.lock')
        _ = subprocess.run(('git','-C',str(harness),'update-ref','refs/remotes/origin/v8/main',sha),check=True,capture_output=True)
        candidate_home=fixture.home/'candidate-cache';candidate_home.mkdir()
        def arm(root: Path, home: Path) -> cli.ArmSelector:
            environment = {'PATH':f'{fixture.compiler.parent}:/usr/bin:/bin:/usr/sbin:/sbin',
                           'LANG':'C','LC_ALL':'C','CARGO_HOME':str(home),'CARGO_TARGET_DIR':str(root/'rust/target'),
                           'RUSTC':str(fixture.compiler),'RUSTC_WRAPPER':'','RUSTC_WORKSPACE_WRAPPER':''}
            return cli.ArmSelector(root,sha,home,(),tuple(sorted(environment.items())))
        self.selection=cli.CliSelectors(harness,sha,sha,'self_proof',arm(fixture.root,fixture.cargo_home),
            arm(candidate,candidate_home),fixture.compiler,file_hash(fixture.compiler),fixture.compiler,file_hash(fixture.compiler),fixture.home/'cli-output')

    def selected(self) -> cli.CliSelectors:
        assert self.selection is not None
        return self.selection

    def test_two_real_pairs_and_full_registry_retained(self) -> None:
        prepared=cli.prepare_cli(self.selected())
        self.assertEqual(len(prepared.registry.fixtures),28)
        self.assertEqual(len(prepared.registry.backlog),12)
        self.assertEqual(len(prepared.declarations.known_fixtures),40)
        self.assertFalse(prepared.selectors.output.exists())
        result=cli.build_cli(prepared)
        self.assertEqual(result.status,'BUILD_ANTECEDENTS_ONLY')
        self.assertFalse(result.publication_authority)
        self.assertFalse(result.original_gate_steps_executed)
        self.assertEqual(result.runtime_qualification,'NOT_RUN')
        assert result.candidate is not None
        for plan,pair in zip(prepared.arms,(result.base,result.candidate),strict=True):
            self.assertEqual(build.recheck_workspace_pair(pair),(plan.debug,plan.release))
            for spec in (plan.debug,plan.release):
                row=document((spec.output/'cargo-build/process.json').read_text())
                self.assertEqual(row.field('environment').mapping(),dict(plan.environment))
                self.assertIn('--no-run',[item.text() for item in row.field('argv').array()])
        self.assertNotEqual(result.base.debug,result.candidate.debug)

    def test_overlay_is_applied_only_after_declarations_and_pristine_freeze(self) -> None:
        selection=self.selected(); overlay=self.fixture.make_overlay()
        selection=replace(selection,candidate=replace(selection.candidate,overlay=overlay))
        prepared=cli.prepare_cli(selection)
        self.assertEqual((selection.candidate.root/'tools/v8/input.txt').read_bytes(),b'tools/v8')
        result=cli.build_cli(prepared)
        self.assertEqual(result.status,'BUILD_ANTECEDENTS_ONLY')
        self.assertNotEqual((selection.base.root/'tools/v8/input.txt').read_bytes(),(selection.candidate.root/'tools/v8/input.txt').read_bytes())
        self.assertEqual(prepared.declarations.changes,())
        assert result.candidate is not None and result.candidate.debug is not None
        document(result.candidate.debug.receipt.read_text()).field('source_kind').equals('DERIVED_OBSERVER')

    def test_bad_selectors_reject_before_any_child(self) -> None:
        selection=self.selected()
        for defect in ('same_root','same_cache','wrapper_absent','wrapper_value','extra_env','published','output_inside','tool'):
            with self.subTest(defect=defect):
                changed=selection
                if defect=='same_root':changed=replace(selection,candidate=selection.base)
                if defect=='same_cache':changed=replace(selection,candidate=replace(selection.candidate,cargo_home=selection.base.cargo_home))
                if defect in ('wrapper_absent','wrapper_value','extra_env'):
                    env=dict(selection.base.environment)
                    if defect=='wrapper_absent':del env['RUSTC_WRAPPER']
                    elif defect=='wrapper_value':env['RUSTC_WRAPPER']='foreign'
                    else:env['EXTRA']='1'
                    changed=replace(selection,base=replace(selection.base,environment=tuple(sorted(env.items()))))
                if defect=='published':changed=replace(selection,published_sha='a'*40)
                if defect=='output_inside':changed=replace(selection,output=selection.candidate.root/'out')
                if defect=='tool':changed=replace(selection,cargo_sha256='0'*64)
                with self.assertRaises(ValueError):_=cli.prepare_cli(changed)
        self.assertFalse(selection.output.exists())
        self.assertFalse((selection.base.root/'rust/target').exists())

    def test_stale_preparation_and_registry_subset_reject(self) -> None:
        selection=self.selected(); prepared=cli.prepare_cli(selection)
        with self.assertRaises(ValueError):_=cli.build_cli(replace(prepared,registry=replace(prepared.registry,fixtures=prepared.registry.fixtures[:1])))
        _=(selection.candidate.root/'tools/v8/input.txt').write_text('late source')
        with self.assertRaises(ValueError):_=cli.build_cli(prepared)
        self.assertFalse(selection.output.exists())

    def test_actual_base_failure_stops_candidate(self) -> None:
        self.fixture.change_compiler(lambda value:value.replace("if sys.argv[1]=='metadata':","if sys.argv[1]=='metadata':\n sys.exit(19)"))
        selection=replace(self.selected(),cargo_sha256=file_hash(self.fixture.compiler),rustc_sha256=file_hash(self.fixture.compiler))
        result=cli.build_cli(cli.prepare_cli(selection))
        self.assertEqual(result.status,'FAIL');self.assertIsNone(result.candidate)
        self.assertFalse((selection.output/'candidate-debug').exists())
        assert result.base.debug is not None
        document(result.base.debug.receipt.read_text()).field('metadata_exit_code').equals(19)

    def test_later_arm_cannot_mutate_prior_evidence(self) -> None:
        prepared=cli.prepare_cli(self.selected())
        def run(spec: ProcessSpec) -> ProcessResult:
            result=execute(spec)
            if spec.output_root==prepared.arms[1].debug.output/'cargo-version':
                _=(prepared.arms[0].debug.output/'cargo-build/stdout.log').write_text('mutated')
            return result
        with patch('generic_build.execute',side_effect=run),self.assertRaises(ValueError):_=cli.build_cli(prepared)
        self.assertTrue((prepared.arms[0].debug.output/'cargo-build/process.json').exists())

    def test_interruption_preserves_original_exception_without_later_arm(self) -> None:
        prepared=cli.prepare_cli(self.selected()); original=KeyboardInterrupt('unknown actual owner state')
        with patch('generic_cli_prepare.build_workspace_pair',side_effect=original) as call, self.assertRaises(KeyboardInterrupt) as raised:
            _=cli.build_cli(prepared)
        self.assertIs(raised.exception,original);self.assertEqual(call.call_count,1)
        self.assertFalse(prepared.arms[1].debug.output.exists())

    def test_actual_candidate_failure_retains_both_objects(self) -> None:
        self.fixture.change_compiler(lambda value:value.replace("if sys.argv[1]=='metadata':",
            "if sys.argv[1]=='metadata' and 'candidate-cache' in os.environ['CARGO_HOME']:\n sys.exit(19)\nif sys.argv[1]=='metadata':"))
        selection=replace(self.selected(),cargo_sha256=file_hash(self.fixture.compiler),rustc_sha256=file_hash(self.fixture.compiler))
        result=cli.build_cli(cli.prepare_cli(selection))
        self.assertEqual(result.status,'FAIL')
        self.assertEqual(result.base.status,'BUILD_ANTECEDENTS_ONLY')
        assert result.candidate is not None and result.candidate.debug is not None
        self.assertEqual(result.candidate.status,'FAIL');self.assertIsNone(result.candidate.release)
        document(result.candidate.debug.receipt.read_text()).field('metadata_exit_code').equals(19)
        _=build.recheck_workspace_pair(result.base)

    def assert_second_blocked(self, defect: str) -> None:
        prepared=cli.prepare_cli(self.selected())
        original=build.build_workspace_pair
        victim=prepared.selectors.harness_root/'tools/v8/input.txt' if defect=='harness' else prepared.selectors.candidate.root/'rust/Cargo.lock'
        def run(debug: build.BuildSpec, release: build.BuildSpec, output: Path) -> build.WorkspaceBuildPair:
            result=original(debug,release,output)
            _=victim.write_bytes(b'late drift')
            return result
        with patch('generic_cli_prepare.build_workspace_pair',side_effect=run) as call,self.assertRaises(ValueError):
            _=cli.build_cli(prepared)
        self.assertEqual(call.call_count,1)
        self.assertFalse(prepared.arms[1].debug.output.exists())

    def test_harness_drift_blocks_second_arm(self) -> None:
        self.assert_second_blocked('harness')

    def test_ignored_lock_drift_blocks_second_arm(self) -> None:
        selection=self.selected()
        self.assertEqual(subprocess.check_output(('git','-C',str(selection.candidate.root),'ls-files','rust/Cargo.lock')),b'')
        self.assert_second_blocked('ignored_input')

    def assert_terminal_mutation_rejects(self, defect: str) -> None:
        prepared=cli.prepare_cli(self.selected());original=build.recheck_workspace_pair
        def check(pair: build.WorkspaceBuildPair) -> tuple[build.BuildSpec, build.BuildSpec]:
            result=original(pair)
            if pair.debug is not None and pair.debug.receipt.parent==prepared.arms[1].debug.output:
                paths={'harness':prepared.selectors.harness_root/'tools/v8/input.txt',
                       'base':prepared.arms[0].debug.output/'cargo-build/stdout.log',
                       'candidate':prepared.arms[1].debug.output/'cargo-build/stdout.log',
                       'target':prepared.arms[0].debug.root/'rust/target/late-empty',
                       'output':prepared.selectors.output/'late-empty',
                       'config':prepared.selectors.base.cargo_home/'config.toml'}
                target=paths[defect]
                if defect in {'target','output'}:target.mkdir()
                else:_=target.write_text('late callback mutation')
            return result
        with patch('generic_cli_prepare.recheck_workspace_pair',side_effect=check),self.assertRaisesRegex(ValueError,'retained source/input/producer evidence changed'):
            _=cli.build_cli(prepared)

    def test_final_callback_harness_bytes_rejected(self) -> None:
        self.assert_terminal_mutation_rejects('harness')

    def test_final_callback_base_bytes_rejected(self) -> None:
        self.assert_terminal_mutation_rejects('base')

    def test_final_callback_candidate_bytes_rejected(self) -> None:
        self.assert_terminal_mutation_rejects('candidate')

    def test_final_callback_empty_target_member_rejected(self) -> None:
        self.assert_terminal_mutation_rejects('target')

    def test_final_callback_empty_output_member_rejected(self) -> None:
        self.assert_terminal_mutation_rejects('output')

    def test_final_callback_previously_absent_config_rejected(self) -> None:
        self.assert_terminal_mutation_rejects('config')

    def test_live_result_recheck_rejects_identity_and_missing_closure(self) -> None:
        result=cli.build_cli(cli.prepare_cli(self.selected()))
        cli.recheck_cli_builds(result)
        assert result.retained_anchor is not None
        variants=(replace(result,retained_anchor=None),replace(result,prepared=replace(result.prepared)),
                  replace(result,base=replace(result.base)),replace(result,status='FAIL'),
                  replace(result,retained_anchor=replace(result.retained_anchor,closure=())))
        for changed in variants:
            with self.subTest(changed=changed.status),self.assertRaises(ValueError):cli.recheck_cli_builds(changed)

    def test_live_result_late_target_and_harness_drift_never_refreshes(self) -> None:
        result=cli.build_cli(cli.prepare_cli(self.selected()));root=result.prepared.selectors.harness_root
        target=result.prepared.selectors.base.root/'rust/target/late-empty';target.mkdir()
        with self.assertRaises(ValueError):cli.recheck_cli_builds(result)
        target.rmdir();cli.recheck_cli_builds(result)
        source=root/'tools/v8/input.txt';old=source.read_bytes();_=source.write_bytes(b'late harness')
        with self.assertRaises(ValueError):cli.recheck_cli_builds(result)
        _=source.write_bytes(old);cli.recheck_cli_builds(result)
