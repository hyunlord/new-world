"""Actual tiny step44 producer -> read-only step45; never native/world admission."""
import json
import os
import unittest
from contextlib import nullcontext
from dataclasses import replace
from pathlib import Path
from unittest.mock import patch

import generic_core_evaluation as evaluation
import generic_direct_run as direct
import generic_full_gate as gate
import test_generic_core_evaluation as fixtures
import test_generic_full_gate as routing
from generic_fixture_run import ComponentResult, Products
from generic_fixtures import ProbeProduct
from generic_gate import GateContractError, document
from generic_process import file_hash
from generic_sources import inventory


class CoreCoordinatorTests(unittest.TestCase):
    def fixture(self, *, fail: bool = False) -> tuple[fixtures.EvaluationTests, gate.Inputs, gate.Outcome]:
        fixture = fixtures.EvaluationTests()
        self.addCleanup(fixture.doCleanups)
        if fail:_ = (fixture.inputs/'config.json').write_text(json.dumps({'candidate':1}))
        files = tuple(inventory(fixture.inputs, [p.name for p in fixture.inputs.iterdir()]).items())
        products = Products(*(ProbeProduct(c*40, fixture.binary, file_hash(fixture.binary), fixture.inputs, files) for c in ('a','b')))
        inputs = gate.Inputs(fixture.registry, products, products, fixture.declarations)
        prior = gate.invoke('direct', lambda out: direct.run_step(inputs.registry, products, inputs.declarations, 44, out), fixture.root/'prior', inputs, step=44)
        self.assertEqual(prior.status, 'FAIL' if fail else 'PASS', prior.error)
        return fixture, inputs, prior

    def test_owned_order_inserts_exact45_and_preserves_other_calls(self) -> None:
        actual = gate.ordered_owned_calls()
        call = gate.OrderedCall(45, 'core_evaluation', ('45:core_evaluation',))
        self.assertEqual(actual.count(call), 1)
        self.assertEqual(actual[actual.index(call)-1].identity, '44:direct')
        self.assertEqual(actual[actual.index(call)+1].step, 46)
        self.assertNotIn(call, gate.ordered_calls())

    def test_actual_saved_return_evaluated_without_new_process_or_authority(self) -> None:
        fixture, inputs, prior = self.fixture()
        with patch('subprocess.Popen', side_effect=AssertionError('step45 may not launch')):
            row = gate.invoke_core_evaluation(inputs, prior, fixture.root/'evaluation')
            gate.recheck_core_evaluation(inputs, row)
        self.assertEqual(row.status, 'PENDING', row.error)
        self.assertEqual(row.returned_status, 'PENDING')
        self.assertIsNotNone(row.receipt)
        assert row.receipt is not None
        node = document(row.receipt.read_text())
        node.field('evaluated_scope').equals('PASS')
        node.field('publication_authority').equals(False)
        node.field('legacy_core').field('heldout_outcomes').equals('HISTORICAL_NOT_FRESH_ACCEPTANCE')

    def test_missing_or_unreturned_prior_never_calls_evaluator(self) -> None:
        fixture, inputs, prior = self.fixture()
        for index, invalid in enumerate((None, replace(prior, returned_status=None), replace(prior, status='FAIL', error='unsettled source'))):
            with patch.object(evaluation, 'evaluate') as evaluate:
                row = gate.invoke_core_evaluation(inputs, invalid, fixture.root/f'reject-{index}')
                evaluate.assert_not_called()
            self.assertEqual(row.status, 'FAIL')
            self.assertIsNone(row.receipt)

    def test_changed_saved_artifact_rejects_without_rebinding(self) -> None:
        fixture, inputs, prior = self.fixture()
        assert prior.root is not None
        _ = (prior.root/'extra.txt').write_text('late')
        with patch.object(evaluation, 'evaluate') as evaluate:
            row = gate.invoke_core_evaluation(inputs, prior, fixture.root/'evaluation')
            evaluate.assert_not_called()
        self.assertEqual(row.status, 'FAIL')

    def test_historical_original_or_evaluation_mutation_rejects(self) -> None:
        for mutate_prior in (False, True):
            with self.subTest(prior=mutate_prior):
                fixture, inputs, prior = self.fixture()
                row = gate.invoke_core_evaluation(inputs, prior, fixture.root/'evaluation')
                assert prior.receipt is not None and row.receipt is not None
                target = prior.receipt if mutate_prior else row.receipt
                _ = target.write_text(target.read_text()+'\n')
                with self.assertRaises(GateContractError):gate.recheck_core_evaluation(inputs, row)

    def test_actual_failed44_is_evaluated_as_fail_and_retained(self) -> None:
        fixture, inputs, prior = self.fixture(fail=True)
        row = gate.invoke_core_evaluation(inputs, prior, fixture.root/'evaluation')
        self.assertEqual(row.status, 'FAIL')
        self.assertEqual(row.returned_status, 'FAIL')
        self.assertIsNotNone(row.core_result)
        assert row.receipt is not None
        self.assertIn('"status": "FAIL"', row.receipt.read_text())
        gate.recheck_core_evaluation(inputs, row)
        self.assertEqual(prior.status, 'FAIL')

    def test_owned_loop_orders_actual44_then45_and_continues_after_failure(self) -> None:
        fixture, inputs, _ = self.fixture(fail=True)
        setup = fixture.root/'context';setup.mkdir()
        context = replace(routing.ordered_inputs(setup), components=inputs)
        calls = (gate.OrderedCall(44,'direct',tuple('44:'+f for f in direct.step_fixtures(44))),
                 gate.OrderedCall(45,'core_evaluation',('45:core_evaluation',)),
                 gate.OrderedCall(50,'direct',('50:read.preservation',)))
        original = gate.ordered_action
        def action(call: gate.OrderedCall, bound: gate.OrderedInputs) -> gate.Action:
            if call.step == 50:
                def tail(_out: Path) -> ComponentResult:
                    raise RuntimeError('tail reached after retained evaluation failure')
                return tail
            return original(call,bound)
        read, write = os.pipe()
        try:
            with patch.object(gate,'ordered_owned_calls',return_value=calls), patch.object(gate,'ordered_bindings'), \
                 patch.object(gate,'perf_bindings'), patch.object(gate,'lane',return_value=nullcontext()), \
                 patch.object(gate,'ordered_action',side_effect=action):
                terminal = gate.run_ordered_owned(context,gate.StrictContext(context.static_product,context.environment),fixture.root/'ordered',read)
        finally:
            os.close(read);os.close(write)
        node = document(terminal.read_text())
        self.assertEqual(node.field('actual_component_sequence').value,[call.identity for call in calls])
        rows = node.field('components').array()
        self.assertEqual(rows[0].field('status').value,'FAIL')
        self.assertEqual(rows[1].field('status').value,'FAIL')
        self.assertIn('tail reached',rows[2].field('error').text())
        self.assertEqual(rows[1].field('returned_status').value,'FAIL')
        self.assertFalse(node.field('publication_authorized').value)
