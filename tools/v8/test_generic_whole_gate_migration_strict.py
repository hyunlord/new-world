"""Strict history policy boundaries; no simulation execution."""
import unittest
from dataclasses import replace
from pathlib import Path
from generic_gate import GateContractError
from generic_test_endpoint import Endpoint
from generic_test_endpoint_types import ExpectedOccurrence, Loss, PendingReason, Snapshot
from generic_test_history_graph import ResolvedHistory
from generic_whole_gate_migration_origin import Origin
from generic_whole_gate_migrations import proposed_window, recorded_zero, strict_window


def endpoint(losses: tuple[Loss, ...] = ()) -> Endpoint:
    start = Snapshot(100, (1, 2), (1, 2), True, losses, True, len(losses), (), 0,
        tuple((key, 0) for key in ('starvation', 'dehydration', 'exposure', 'combat', 'old_age')))
    return Endpoint(start, replace(start, tick=160), None, False,
        (PendingReason.EXTERNAL_ORIGIN_UNRESOLVED,), '0'*64, '60_ticks')


def origin(missing: tuple[str, ...] = ()) -> Origin:
    return Origin(Path('/fixture.save'), '0'*64, 6, '1'*40, (), (), missing)


def history() -> ResolvedHistory:
    return ResolvedHistory(ExpectedOccurrence('test', 'test', 0, 'branch', 100, 160, True),
        None, False, (PendingReason.EXTERNAL_ORIGIN_UNRESOLVED,), ())


class StrictHistoryTests(unittest.TestCase):
    def test_inherited_loss_fails_despite_missing_origin(self) -> None:
        row = endpoint((Loss(5, 'Fire', None),))
        self.assertEqual(proposed_window(row, row)['status'], 'PASS')
        self.assertFalse(proposed_window(row, row)['strict_ordinary_admission'])
        with self.assertRaisesRegex(GateContractError, 'FULL recorded'):
            _ = strict_window(row, row, history(), origin(('missing actual producer',)))

    def test_new_loss_fails(self) -> None:
        row = endpoint()
        changed = replace(row, terminal=replace(row.terminal, losses=(Loss(120, 'Fire', 1),)))
        with self.assertRaisesRegex(GateContractError, 'FULL recorded'):
            recorded_zero(row, changed)

    def test_strict_zero_uses_conservative_floor_without_founders(self) -> None:
        row = strict_window(endpoint(), endpoint(), history(), origin())
        self.assertEqual(row['status'], 'PASS')
        self.assertIsNone(row['founding_census'])
        self.assertEqual(row['candidate_full_recorded_losses'], [])

    def test_empty_loss_with_missing_origin_stays_incomplete(self) -> None:
        row = strict_window(endpoint(), endpoint(), history(), origin(('format10 origin absent',)))
        self.assertEqual(row['status'], 'INCOMPLETE')

    def test_lower_survival_is_unresolved_not_false_floor_failure(self) -> None:
        row = endpoint()
        changed = replace(row, terminal=replace(row.terminal, alive_ids=(1,)))
        result = strict_window(row, changed, history(), origin())
        self.assertEqual(result['status'], 'INCOMPLETE')
        self.assertIn('INCOMPLETE_SURVIVAL_FLOOR', str(result['missing']))

    def test_pair_horizon_mismatch_rejected(self) -> None:
        row = endpoint()
        with self.assertRaisesRegex(GateContractError, 'horizon'):
            recorded_zero(row, replace(row, terminal=replace(row.terminal, tick=161)))

    def test_invented_founder_rejected(self) -> None:
        row = endpoint()
        with self.assertRaisesRegex(GateContractError, 'invented founders'):
            recorded_zero(row, replace(row, founding=row.initial))


if __name__ == '__main__':
    _ = unittest.main()
