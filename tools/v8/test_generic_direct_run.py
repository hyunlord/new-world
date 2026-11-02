# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run: python3 -B -m unittest discover -s tools/v8 -p test_generic_direct_run.py
"""Mock only component boundaries; native components have separate child tests."""

import tempfile
import unittest
from contextlib import ExitStack
from dataclasses import replace
from pathlib import Path
from typing import final
from unittest.mock import patch

import generic_baseline_run as baseline
import generic_direct_run as direct
import generic_f10_run as f10
import generic_fixture_registry as registry_module
import generic_m6a_run as m6a
from generic_declarations import Declarations
from generic_fixture_registry import DIRECT_IDS, Invocation, Registry, load_registry
from generic_fixture_run import ComponentResult, Products
from generic_fixtures import ProbeProduct
from generic_gate import GateContractError, document
from generic_process import atomic_json, file_hash
from generic_sources import inventory
from mux2_gate_inputs import Json


@final
class DirectTests(unittest.TestCase):
    def __init__(self, methodName: str = "runTest") -> None:
        super().__init__(methodName)
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name).resolve()
        self.registry = load_registry(Path(registry_module.__file__).with_name("gate_fixtures.json"))
        products: list[ProbeProduct] = []
        for arm, sha in (("base", "a" * 40), ("candidate", "b" * 40)):
            folder = self.root / arm
            folder.mkdir()
            executable = folder / "probe"
            _ = executable.write_text("never executed")
            products.append(
                ProbeProduct(
                    sha,
                    executable,
                    file_hash(executable),
                    folder,
                    (("probe", file_hash(executable)),),
                )
            )
        self.products = Products(products[0], products[1])
        self.declarations = Declarations(
            "b" * 40,
            "a" * 40,
            "publication",
            tuple(sorted(DIRECT_IDS)),
            (),
            (),
            (),
            (),
            "d" * 64,
        )
        self.calls: list[tuple[str, tuple[Invocation, ...]]] = []

    def invoke(
        self,
        *,
        fail: str = "",
        raised: str = "",
        missing: str = "",
        mismatch: str = "",
        preflight: str = "",
        tamper: str = "",
        artifact_tamper: str = "",
        artifact_kind: str = "raw",
        unsafe_manifest: bool = False,
        step: int | None = None,
        output_name: str = "out",
        raise_after_outputs: str = "",
        m6a_mutation: str = "",
    ) -> ComponentResult:
        def run(
            name: str, registry: Registry, plans: tuple[Invocation, ...], output: Path
        ) -> ComponentResult:
            self.calls.append((name, plans))
            if name == artifact_tamper:
                prior = next((output.parent / 'runs/base').glob('identity.baseline_samples/**/raw.json'))
                if artifact_kind == 'scorer':
                    prior = next((output.parent / 'runs/base').glob('**/standing-scorer/process.json'))
                if artifact_kind == 'symlink':
                    prior.unlink()
                    prior.symlink_to(self.products.base.executable)
                else:
                    _ = prior.write_text(prior.read_text() + ' ')
            if name == tamper:
                prior = output.parent / "baseline.json"
                _ = prior.write_text(prior.read_text() + " ")
            if name == raised:
                raise GateContractError("SYNTHETIC", name, "component did not return")
            status = "FAIL" if name == fail else "PASS"
            manifests: dict[str, Json] = {}
            for plan in plans:
                plan.out.mkdir(parents=True)
                _ = (plan.out / 'raw.json').write_text('{}')
                manifests[plan.id] = dict(inventory(plan.out, ['raw.json']))
            scorers: list[Json] = []
            oracles: list[Json] = []
            scorer_outputs: dict[str, Json] = {}
            if name == 'm6a':
                selected = next((p for p in plans if p.arm == 'base' and p.fixture_id == m6a.SELECTED),None)
                if selected is not None:
                    folder = selected.out.parent / 'matched-companion'
                    folder.mkdir()
                    _ = (folder / 'raw.json').write_text('{}')
                    manifests[selected.id + '/matched-companion'] = dict(inventory(folder, ['raw.json']))
                for arm in ('base', 'candidate'):
                    selected = next((p for p in plans if p.arm == arm and p.fixture_id == m6a.FIXED),None)
                    if selected is None:continue
                    folder = selected.out.parent / 'standing-scorer'
                    (folder / 'process').mkdir(parents=True)
                    for path in (folder / 'process/process.json', folder / 'score.json'):
                        _ = path.write_text('{}')
                    scorers.append({'arm': arm, 'receipt': m6a.reference(folder / 'process/process.json'),
                                    'score': m6a.reference(folder / 'score.json')})
            if name == 'f10':
                for plan in plans:
                    if plan.fixture_id == f10.INJECTED:
                        folder = plan.out / 'f10-scorer'
                        folder.mkdir()
                        _ = (folder / 'process.json').write_text('{}')
                        oracles.append({'invocation': plan.id, 'receipt': str(folder / 'process.json'),
                                        'outputs': dict(inventory(folder, ['process.json']))})
            if name == 'ordinary':
                folder = plans[0].out / 'standing-scorer'
                folder.mkdir()
                _ = (folder / 'process.json').write_text('{}')
                scorer_outputs[str(folder)] = dict(inventory(folder, ['process.json']))
                if unsafe_manifest:
                    scorer_outputs[str(self.root / 'outside')] = {'file': 'a' * 64}
            artifact_key = {'baseline': 'producer_outputs', 'ordinary': 'producer_outputs',
                            'f10': 'producer_artifacts', 'm6a': 'artifacts'}[name]
            if name != missing:
                atomic_json(
                    output,
                    {
                        "schema": 'm6a_step_component_v1' if name=='m6a' and step is not None else direct.SCHEMAS[name],
                        **({'step':step,'fixtures':list(m6a.step_fixtures(step)),'whole_step_complete':False,
                            'artifact_snapshots':{str(folder):dict(inventory(folder,[p.name for p in folder.iterdir()]))
                                for folder in [*(p.out for p in plans),
                                    *(p.out.parent/'standing-scorer' for p in plans if p.fixture_id==m6a.FIXED),
                                    *(p.out.parent/'matched-companion' for p in plans if p.fixture_id==m6a.SELECTED and p.arm=='base')]}}
                           if name=='m6a' and step is not None else {}),
                        "status": status,
                        "publication_authority": False,
                        "base": "a" * 40,
                        "candidate": "b" * 40,
                        "registry_sha256": registry.sha256,
                        "declaration_manifest_sha256": "d" * 64,
                        "executions": [],
                        "errors": [],
                        artifact_key: manifests,
                        "scorer_outputs": scorer_outputs,
                        "oracles": oracles,
                        "scorers": scorers,
                    },
                )
            if name=='m6a' and m6a_mutation:
                data=document(output.read_text()).mapping()
                if m6a_mutation=='step':data['step']=48 if step==47 else 47
                elif m6a_mutation=='schema':data['schema']='m6a_component_v1'
                elif m6a_mutation=='fixture':data['fixtures']=['m6a.foreign']
                elif m6a_mutation=='missing_scorer':data['scorers']=[]
                elif m6a_mutation=='missing_snapshot':data['artifact_snapshots']={}
                elif m6a_mutation=='foreign_snapshot':data['artifact_snapshots']={str(self.root/'foreign'): {}}
                elif m6a_mutation=='late_raw':
                    prior=plans[0].out/'raw.json';_=prior.write_text(prior.read_text()+' ')
                elif m6a_mutation=='selected_scorer':data['scorers']=[{'arm':'base'}]
                output.unlink();atomic_json(output,data)
            if name == raise_after_outputs:
                raise GateContractError("SYNTHETIC", name, "raised after retained artifacts")
            return ComponentResult(
                output,
                ("FAIL" if status == "PASS" else "PASS")
                if name == mismatch
                else status,
            )

        with ExitStack() as stack:
            for name, module in direct.MODULES.items():
                if name == preflight:
                    _ = stack.enter_context(
                        patch.object(
                            module,
                            "prepare_step" if name=="m6a" and step is not None else "prepare",
                            side_effect=GateContractError(
                                "SYNTHETIC", name, "preflight failed"
                            ),
                        )
                    )
                else:
                    _ = stack.enter_context(
                        patch.object(module, "prepare_step" if name=="m6a" and step is not None else "prepare", return_value=None)
                    )
                if name in ("ordinary", "m6a"):

                    def runner(
                        registry: Registry,
                        plans: tuple[Invocation, ...],
                        _products: Products,
                        _declarations: Declarations,
                        output: Path,
                        *,
                        component: str = name,
                    ) -> ComponentResult:
                        return run(component, registry, plans, output)

                    _ = stack.enter_context(
                        patch.object(module, "run_registered", side_effect=runner)
                    )
                    if name == "m6a" and step is not None:
                        def step_runner(registry: Registry, plans: tuple[Invocation,...], _products: Products,
                                        _declarations: Declarations, actual_step: int, output: Path) -> ComponentResult:
                            self.assertEqual(actual_step,step)
                            return run('m6a',registry,plans,output)
                        _=stack.enter_context(patch.object(module,'run_step',side_effect=step_runner))
                    if name == "ordinary":
                        _ = stack.enter_context(patch.object(module, "run_preservation", side_effect=runner))
                else:

                    def protected(
                        registry: Registry,
                        plans: tuple[Invocation, ...],
                        _products: Products,
                        output: Path,
                        *,
                        component: str = name,
                    ) -> ComponentResult:
                        return run(component, registry, plans, output)

                    _ = stack.enter_context(
                        patch.object(module, "run_registered", side_effect=protected)
                    )
            if step is not None:
                return direct.run_step(self.registry, self.products, self.declarations, step, self.root / output_name)
            return direct.run_all(self.registry, self.products, self.declarations, self.root / output_name)

    def test_complete_partition_own_arm_roots_repeats_and_no_authority(self) -> None:
        result = self.invoke()
        data = document(result.receipt.read_text())
        self.assertEqual(result.status, "PASS")
        self.assertFalse(data.field("publication_authority").value)
        self.assertEqual(data.field("registry_status").value, "INCOMPLETE_ADAPTERS")
        self.assertEqual(len(data.field("unresolved_backlog").array()), 12)
        all_plans = [p for _, plans in self.calls for p in plans]
        self.assertEqual({p.fixture_id for p in all_plans}, DIRECT_IDS)
        self.assertEqual(len(all_plans), 62)
        self.assertEqual(len({p.id for p in all_plans}), len(all_plans))
        for plan in all_plans:
            self.assertEqual(plan.cwd, self.products.arm(plan.arm).input_root)
            self.assertIsNotNone(plan.argv)
            self.assertEqual(
                plan.argv[0] if plan.argv else None,
                str(self.products.arm(plan.arm).executable),
            )
        lifecycle = [p for p in all_plans if p.fixture_id == "m6a.lifecycle_off"]
        self.assertEqual(len(lifecycle), 4)
        self.assertEqual({p.repeat for p in lifecycle}, {"a", "b"})
        for row in data.field("components").array():
            self.assertEqual(
                row.field("receipt_sha256").text(),
                file_hash(Path(row.field("receipt").text())),
            )

    def test_missing_fixture_rejected_before_any_component(self) -> None:
        self.registry = replace(self.registry, fixtures=self.registry.fixtures[:-1])
        result = self.invoke()
        self.assertEqual(result.status, "FAIL")
        self.assertFalse(self.calls)
        self.assertTrue(
            all(
                row.field("status").value == "NOT_RUN"
                for row in document(result.receipt.read_text())
                .field("components")
                .array()
            )
        )

    def test_extra_fixture_rejected_before_any_component(self) -> None:
        extra = replace(self.registry.fixtures[0], id="unknown.fixture")
        self.registry = replace(
            self.registry, fixtures=(*self.registry.fixtures, extra)
        )
        self.assertEqual(self.invoke().status, "FAIL")
        self.assertFalse(self.calls)

    def test_overlapping_component_partition_rejected_before_any_component(
        self,
    ) -> None:
        with patch.object(baseline, "SUPPORTED", baseline.SUPPORTED | {"colony.u10"}):
            self.assertEqual(self.invoke().status, "FAIL")
        self.assertFalse(self.calls)

    def test_last_component_preflight_blocks_all_children(self) -> None:
        result = self.invoke(preflight="m6a")
        self.assertEqual(result.status, "FAIL")
        self.assertFalse(self.calls)

    def test_failed_component_retained_and_other_components_run(self) -> None:
        result = self.invoke(fail="ordinary")
        self.assertEqual(result.status, "FAIL")
        self.assertEqual(len(self.calls), 4)
        rows = document(result.receipt.read_text()).field("components").array()
        self.assertEqual(
            [r.field("status").value for r in rows], ["PASS", "FAIL", "PASS", "PASS"]
        )

    def test_exception_retained_without_fabricating_receipt(self) -> None:
        result = self.invoke(raised="ordinary")
        self.assertEqual(result.status, "FAIL")
        row = document(result.receipt.read_text()).field("components").array()[1]
        self.assertIsNone(row.field("receipt_sha256").value)
        self.assertEqual(len(self.calls), 4)

    def test_missing_disk_receipt_cannot_use_returned_pass(self) -> None:
        self.assertEqual(self.invoke(missing="f10").status, "FAIL")

    def test_return_status_must_equal_actual_receipt(self) -> None:
        self.assertEqual(self.invoke(mismatch="f10").status, "FAIL")

    def test_fresh_output_required(self) -> None:
        _ = self.invoke()
        with self.assertRaises(GateContractError):
            _ = self.invoke()

    def test_returned_pass_cannot_override_failed_disk_receipt(self) -> None:
        self.assertEqual(self.invoke(fail="f10", mismatch="f10").status, "FAIL")

    def test_later_component_cannot_change_prior_receipt(self) -> None:
        result = self.invoke(tamper="m6a")
        self.assertEqual(result.status, "FAIL")
        row = document(result.receipt.read_text()).field("components").array()[0]
        self.assertEqual(row.field("status").value, "FAIL")
        self.assertIn("changed", row.field("postflight_error").text())

    def test_later_component_cannot_change_prior_raw_artifact(self) -> None:
        result = self.invoke(artifact_tamper='m6a')
        self.assertEqual(result.status, 'FAIL')
        row = document(result.receipt.read_text()).field('components').array()[0]
        self.assertEqual(row.field('status').value, 'FAIL')
        self.assertIn('artifact', row.field('postflight_error').text())

    def test_manifest_cannot_escape_registered_output_roots(self) -> None:
        result = self.invoke(unsafe_manifest=True)
        self.assertEqual(result.status, 'FAIL')
        row = document(result.receipt.read_text()).field('components').array()[1]
        self.assertIn('foreign', row.field('error').text())

    def test_later_component_cannot_change_prior_scorer(self) -> None:
        result = self.invoke(artifact_tamper='m6a', artifact_kind='scorer')
        self.assertEqual(result.status, 'FAIL')
        row = document(result.receipt.read_text()).field('components').array()[1]
        self.assertIn('artifact', row.field('postflight_error').text())

    def test_later_artifact_symlink_is_rejected(self) -> None:
        result = self.invoke(artifact_tamper='m6a', artifact_kind='symlink')
        self.assertEqual(result.status, 'FAIL')
        row = document(result.receipt.read_text()).field('components').array()[0]
        self.assertIn('symlink', row.field('postflight_error').text())

    def test_component_partition_gap_rejected(self) -> None:
        with patch.object(
            baseline, "SUPPORTED", baseline.SUPPORTED - {"core.baseline10"}
        ):
            self.assertEqual(self.invoke().status, "FAIL")
        self.assertFalse(self.calls)

    def test_missing_backlog_never_implies_completion(self) -> None:
        self.registry = replace(self.registry, backlog=self.registry.backlog[:-1])
        result = self.invoke()
        self.assertEqual(result.status, "FAIL")
        self.assertFalse(self.calls)


    def test_closed_map_partitions_all_direct_fixtures_once(self) -> None:
        assigned = [fixture for step in direct.DIRECT_STEP_FIXTURES for fixture in direct.step_fixtures(step)]
        self.assertEqual(len(assigned), len(set(assigned)))
        self.assertEqual(set(assigned), DIRECT_IDS)
        self.assertEqual(direct.step_fixtures(19), ("replay.generate", "replay.recorded", "replay.scripted"))
        for step in (0, 17, 51, True):
            with self.assertRaises(GateContractError):
                _ = direct.step_fixtures(step)

    def test_step44_runs_baseline_and_ordinary_lifecycle_but_no_joint_m6a(self) -> None:
        result = self.invoke(step=44)
        self.assertEqual(result.status, "PASS")
        self.assertEqual([name for name, _ in self.calls], ["baseline", "ordinary"])
        plans = [plan for _, group in self.calls for plan in group]
        self.assertEqual({plan.fixture_id for plan in plans}, set(direct.step_fixtures(44)))
        self.assertEqual(len([p for p in plans if p.fixture_id == "m6a.lifecycle_off"]), 4)
        receipt = document(result.receipt.read_text())
        self.assertEqual(receipt.field("schema").value, "direct_step_components_v1")
        self.assertEqual(receipt.field("step").value, 44)
        self.assertFalse(receipt.field("publication_authority").value)
        self.assertFalse(receipt.field("whole_step_complete").value)

    def test_step50_runs_six_fresh_fixtures_not_prior_receipts(self) -> None:
        _ = self.invoke(step=18, output_name="step18")
        earlier = [p for _, plans in self.calls for p in plans]
        self.calls.clear()
        result = self.invoke(step=50, output_name="step50")
        self.assertEqual(result.status, "PASS")
        self.assertEqual([name for name, _ in self.calls], ["ordinary"])
        current = [p for _, plans in self.calls for p in plans]
        self.assertEqual({p.fixture_id for p in current}, set(direct.step_fixtures(50)))
        self.assertEqual(len({p.fixture_id for p in current}), 6)
        self.assertEqual([(p.fixture_id, p.arm) for p in current],
                         [(fixture, arm) for fixture in direct.step_fixtures(50) for arm in ("base", "candidate")])
        self.assertTrue(all(p.out.is_relative_to(self.root / "step50/runs") for p in current))
        self.assertFalse({p.out for p in current}.intersection(p.out for p in earlier))
        with self.assertRaises(GateContractError):
            _ = self.invoke(step=50, output_name="step50")

    def test_global_registry_and_declarations_validated_before_subset(self) -> None:
        self.registry = replace(self.registry, fixtures=self.registry.fixtures[:-1])
        self.assertEqual(self.invoke(step=10).status, "FAIL")
        self.assertFalse(self.calls)

    def test_unselected_fixture_declaration_coverage_still_required(self) -> None:
        self.declarations = replace(self.declarations,
                                    known_fixtures=tuple(f for f in self.declarations.known_fixtures if f == "colony.u10"))
        self.assertEqual(self.invoke(step=18).status, "FAIL")
        self.assertFalse(self.calls)

    def test_f10_indivisible_four_plan_batch(self) -> None:
        self.assertEqual(self.invoke(step=28).status, "PASS")
        self.assertEqual(len(self.calls), 1)
        name, plans = self.calls[0]
        self.assertEqual(name, "f10")
        self.assertEqual(len(plans), 4)
        self.assertEqual({(p.arm, p.fixture_id) for p in plans},
                         {(arm, fixture) for arm in ("base", "candidate") for fixture in direct.step_fixtures(28)})

    def test_replay_keeps_complete_group_and_same_arm_dependency(self) -> None:
        self.assertEqual(self.invoke(step=19).status, "PASS")
        plans = self.calls[0][1]
        self.assertEqual(len(plans), 6)
        self.assertEqual({p.fixture_id for p in plans}, set(direct.step_fixtures(19)))
        for plan in plans:
            if plan.fixture_id == "replay.recorded":
                dependency = plan.generated_input
                self.assertIsNotNone(dependency)
                if dependency is None:
                    raise AssertionError("missing generated replay dependency")
                producer = next(p for p in plans if p.id == dependency.producer_invocation)
                self.assertEqual(plan.arm, producer.arm)
                self.assertEqual(dependency.path, producer.out / "commands/colony_s5_commands.json")

    def test_m6a47_and48_run_only_their_closed_fixture_groups(self) -> None:
        for step in (47, 48):
            with self.subTest(step=step):
                result = self.invoke(step=step, output_name=f"step{step}")
                self.assertEqual(result.status, "PASS")
                receipt = document(result.receipt.read_text())
                self.assertEqual(receipt.field("fixtures").value, list(direct.step_fixtures(step)))
                rows = receipt.field("components").array()
                self.assertEqual(len(rows), 1)
                self.assertEqual(rows[0].field("status").value, "PASS")
                self.assertTrue(rows[0].field("invocations").array())
        self.assertEqual([len(plans) for _,plans in self.calls],[4,2])
        self.assertEqual([{p.fixture_id for p in plans} for _,plans in self.calls],[{m6a.FIXED},{m6a.SELECTED}])

    def test_step_safety_failure_retained(self) -> None:
        result = self.invoke(step=44, fail="ordinary")
        self.assertEqual(result.status, "FAIL")
        plans = [p for _, group in self.calls for p in group]
        self.assertIn("core.steward_off", {p.fixture_id for p in plans})
        row = document(result.receipt.read_text()).field("components").array()[1]
        self.assertEqual(row.field("original_receipt_status").value, "FAIL")
        self.assertTrue(row.field("failed_artifact_snapshots").mapping())

    def test_step_exception_after_written_failure_keeps_artifacts(self) -> None:
        result = self.invoke(step=18, fail="ordinary", raise_after_outputs="ordinary")
        self.assertEqual(result.status, "FAIL")
        row = document(result.receipt.read_text()).field("components").array()[0]
        self.assertIsNotNone(row.field("receipt_sha256").value)
        self.assertTrue(row.field("failed_artifact_snapshots").mapping())


    def test_failed_component_raw_artifacts_still_detect_later_mutation(self) -> None:
        result = self.invoke(fail="baseline", raise_after_outputs="baseline", artifact_tamper="m6a")
        self.assertEqual(result.status, "FAIL")
        row = document(result.receipt.read_text()).field("components").array()[0]
        self.assertTrue(row.field("failed_artifact_snapshots").mapping())
        self.assertIn("artifact", row.field("postflight_error").text())


    def test_m6a_step_identity_and_artifact_receipts_fail_closed(self) -> None:
        for step,mutation in ((47,'step'),(48,'step'),(47,'schema'),(48,'fixture'),
                              (47,'missing_scorer'),(47,'missing_snapshot'),(48,'foreign_snapshot'),
                              (47,'late_raw'),(48,'selected_scorer')):
            with self.subTest(step=step,mutation=mutation):
                result=self.invoke(step=step,m6a_mutation=mutation,output_name=f'{step}-{mutation}')
                self.assertEqual(result.status,'FAIL')
                node=document(result.receipt.read_text())
                self.assertEqual(node.field('components').array()[0].field('status').value,'FAIL')
                self.assertFalse(node.field('whole_step_complete').value)

    def test_m6a_step_preflight_retains_full_registry_validation(self) -> None:
        self.registry=replace(self.registry,fixtures=tuple(f for f in self.registry.fixtures if f.id!='colony.u10'))
        result=self.invoke(step=47)
        self.assertEqual(result.status,'FAIL')
        self.assertFalse(self.calls)

    def test_m6a_step_failure_retains_failed_artifacts(self) -> None:
        for step in (47,48):
            with self.subTest(step=step):
                result=self.invoke(step=step,fail='m6a',output_name=f'failed{step}')
                self.assertEqual(result.status,'FAIL')
                row=document(result.receipt.read_text()).field('components').array()[0]
                self.assertTrue(row.field('failed_artifact_snapshots').mapping())
                self.assertIsNotNone(row.field('receipt_sha256').value)


class M6aDirectIntegrationTests(unittest.TestCase):
    def test_real_synthetic_m6a_step_children_bind_through_direct(self) -> None:
        from test_generic_m6a_run import scenario
        for step,fixture,count,scorers in ((47,m6a.FIXED,4,2),(48,m6a.SELECTED,3,0)):
            with self.subTest(step=step),scenario() as state:
                _,products=state.bindings()
                declarations=Declarations('b'*40,'a'*40,'publication',tuple(sorted(DIRECT_IDS)),(),(),(),(),'d'*64)
                result=direct.run_step(state.registry,products,declarations,step,state.root/f'step{step}')
                node=document(result.receipt.read_text())
                self.assertEqual(result.status,'PASS',node.value)
                child=document((result.receipt.parent/'m6a.json').read_text())
                self.assertEqual(child.field('schema').value,'m6a_step_component_v1')
                self.assertEqual(child.field('fixtures').value,[fixture])
                self.assertEqual(len(child.field('executions').array()),count)
                self.assertEqual(len(child.field('scorers').array()),scorers)
                self.assertEqual(len(node.field('components').array()[0].field('artifact_snapshots').mapping()),count+scorers)
