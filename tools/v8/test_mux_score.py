"""Acceptance-bar boundaries and coverage checks for the steward evaluation."""
import copy
import unittest
from tools.v8.mux_score import score


class ScoreTests(unittest.TestCase):
    def fixture(self):
        metric = {'ticks_per_day': 1440, 'stockpile_first_tick': 1440,
                  'walled_beds_for_all_first_tick': 14400, 'first_winter_observed': True,
                  'first_winter_complete': True, 'first_winter_cold_deaths': 0,
                  'cold_snap_observed_ticks': 360, 'cold_snap_firewood_zero_ticks': 0,
                  'alive_at_day48': True, 'alive_end': 10}
        return {'scene': 'colony', 'days': 48, 'seeds': [
            {'seed': n, 'mux': copy.deepcopy(metric), 'ledger_violation_ticks': 0,
             'reservation_violation_ticks': 0} for n in range(1000, 1032)]}

    def test_exact_deadlines_pass(self):
        self.assertTrue(score(self.fixture(), 1000, 32)['passed'])

    def test_no_observation_is_not_success(self):
        report = self.fixture()
        for row in report['seeds']:
            row['mux']['cold_snap_observed_ticks'] = 0
        self.assertFalse(score(report, 1000, 32)['passed'])

    def test_cold_and_firewood_thresholds_remain_literal(self):
        report = self.fixture()
        for row in report['seeds'][:4]:
            row['mux']['first_winter_cold_deaths'] = 1
        for row in report['seeds'][:8]:
            row['mux']['cold_snap_firewood_zero_ticks'] = 1
        self.assertTrue(score(report, 1000, 32)['passed'])
        report['seeds'][8]['mux']['cold_snap_firewood_zero_ticks'] = 1
        self.assertFalse(score(report, 1000, 32)['passed'])

    def test_late_or_missing_completion_fails(self):
        for field, value in [('stockpile_first_tick', 1441), ('walled_beds_for_all_first_tick', None)]:
            report = self.fixture()
            report['seeds'][0]['mux'][field] = value
            self.assertFalse(score(report, 1000, 32)['passed'])

    def test_duplicate_or_missing_seed_rejected(self):
        report = self.fixture()
        report['seeds'][0]['seed'] = 1001
        with self.assertRaises(ValueError):
            score(report, 1000, 32)
        report = self.fixture()
        report['seeds'].pop()
        with self.assertRaises(ValueError):
            score(report, 1000, 32)

    def test_missing_field_does_not_default_to_pass(self):
        report = self.fixture()
        del report['seeds'][0]['mux']['alive_at_day48']
        with self.assertRaises(KeyError):
            score(report, 1000, 32)

if __name__ == '__main__':
    unittest.main()
