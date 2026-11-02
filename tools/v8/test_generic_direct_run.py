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
        self.registry = load_registry(Path(__file__).with_name("gate_fixtures.json"))
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
                selected = next(p for p in plans if p.arm == 'base' and p.fixture_id == m6a.SELECTED)
                folder = selected.out.parent / 'matched-companion'
                folder.mkdir()
                _ = (folder / 'raw.json').write_text('{}')
                manifests[selected.id + '/matched-companion'] = dict(inventory(folder, ['raw.json']))
                for arm in ('base', 'candidate'):
                    selected = next(p for p in plans if p.arm == arm and p.fixture_id == m6a.FIXED)
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
                        "schema": direct.SCHEMAS[name],
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
                            "prepare",
                            side_effect=GateContractError(
                                "SYNTHETIC", name, "preflight failed"
                            ),
                        )
                    )
                else:
                    _ = stack.enter_context(
                        patch.object(module, "prepare", return_value=None)
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
            return direct.run_all(
                self.registry, self.products, self.declarations, self.root / "out"
            )

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
