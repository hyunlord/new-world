"""P7 accepts only the authorized side64 exception after raw verification."""
import copy
import unittest

from core_perf_check import CEILINGS, RSS_SCENES, SCENES, TICK_BAR, EvidenceError
from core_p7_acceptance import check_paired


def fixture():
    standing = [dict(scene=s, metric=m, base=100., new=100., ratio=1., bar=1.1)
                for s in SCENES for m in ('tick_mean_us', 'tick_p99_us')]
    for row in standing:
        if row['scene'] == 'n200_side64':
            row.update(new=130., ratio=1.3)
    comparisons = [dict(scene=s, base_decision_mean_us=50., new_mind_mean_us=50.,
                        base_tick_mean_us=100., new_tick_mean_us=100.,
                        mind_ratio=1., tick_ratio=1.) for s in ('n200', 'n1000', 'n2000')]
    files = [dict(path=f'/session/accepted/{role}/{kind}-{s}-r{r}.json', sha256='a'*64)
             for role in ('base', 'new') for kind, scenes, repeats in
             [('timed', SCENES, 3), ('rss', RSS_SCENES, 5)]
             for s in scenes for r in range(1, repeats+1)]
    return dict(schema=1, status='FAIL', accepted_pairs=33, invalid_attempt_members=0,
                historical_ledger_used_as_base=False, files=files,
                standing_ratios=standing, standing_c3=dict.fromkeys(CEILINGS, 'PASS'),
                alloc_per_agent_ratio=1., comparisons=comparisons,
                mind_ratio_bar=.5, tick_ratio_bar=TICK_BAR,
                saturation_companion=dict(status='MEASURED'), failures=[
                    'n200_side64/tick_mean_us>1.1', 'n200_side64/tick_p99_us>1.1',
                    'N1000 mind/old decision>0.5',
                    'N1000 tick ratio exceeds accepted arithmetic bar'])


class P7AcceptanceTests(unittest.TestCase):
    def test_authorized_exception_retains_failure_labels_without_mutation(self):
        receipt = fixture()
        before = copy.deepcopy(receipt)
        self.assertEqual(check_paired(receipt), receipt['failures'])
        self.assertEqual(receipt, before)

    def test_rejects_malformed_or_unjustified_evidence(self):
        changes = [
            lambda d: d.update(accepted_pairs=32),
            lambda d: d.update(accepted_pairs=33.),
            lambda d: d.update(invalid_attempt_members=-1),
            lambda d: d.update(invalid_attempt_members=True),
            lambda d: d.update(status='PENDING'),
            lambda d: d.update(historical_ledger_used_as_base=True),
            lambda d: d.update(alloc_per_agent_ratio=1.10001),
            lambda d: d.update(alloc_per_agent_ratio=float('nan')),
            lambda d: d['standing_c3'].pop(CEILINGS[0]),
            lambda d: d['standing_c3'].update({CEILINGS[0]: 'MISS'}),
            lambda d: d['standing_ratios'].pop(),
            lambda d: d['standing_ratios'].append(d['standing_ratios'][0]),
            lambda d: d['standing_ratios'][0].update(new=120., ratio=1.2),
            lambda d: d['standing_ratios'][0].update(base=0.),
            lambda d: d['standing_ratios'][0].update(ratio=float('inf')),
            lambda d: d['standing_ratios'][0].update(ratio=True),
            lambda d: d['standing_ratios'][0].update(ratio=.9),
            lambda d: d['standing_ratios'][0].update(bar=1.5),
            lambda d: d['files'].pop(),
            lambda d: d['files'].append(d['files'][0]),
            lambda d: d['files'][0].update(sha256='bad'),
            lambda d: d['files'][0].update(path='/session/accepted/base/rss-n200-r6.json'),
            lambda d: d['failures'].pop(),
            lambda d: d['failures'].append('alloc_per_agent>1.1'),
            lambda d: d['failures'].append(d['failures'][0]),
            lambda d: d['comparisons'].pop(),
            lambda d: d['comparisons'][0].update(mind_ratio=float('nan')),
            lambda d: d['saturation_companion'].update(status='PENDING'),
        ]
        for index, change in enumerate(changes):
            with self.subTest(index=index):
                receipt = fixture()
                change(receipt)
                with self.assertRaises(EvidenceError):
                    check_paired(receipt)

    def test_each_non_side64_metric_and_each_ceiling_is_mandatory(self):
        for scene in SCENES:
            if scene == 'n200_side64':
                continue
            for metric in ('tick_mean_us', 'tick_p99_us'):
                with self.subTest(scene=scene, metric=metric):
                    receipt = fixture()
                    row = next(r for r in receipt['standing_ratios']
                               if r['scene'] == scene and r['metric'] == metric)
                    row.update(new=110.00001, ratio=1.1000001)
                    with self.assertRaises(EvidenceError):
                        check_paired(receipt)
        for ceiling in CEILINGS:
            with self.subTest(ceiling=ceiling):
                receipt = fixture()
                receipt['standing_c3'][ceiling] = 'MISS'
                with self.assertRaises(EvidenceError):
                    check_paired(receipt)

    def test_clean_results_and_exact_threshold_need_no_exception(self):
        receipt = fixture()
        for row in receipt['standing_ratios']:
            row.update(new=110., ratio=1.1)
        for row in receipt['comparisons']:
            row.update(new_mind_mean_us=20., mind_ratio=.4,
                       new_tick_mean_us=60., tick_ratio=.6)
        receipt.update(failures=[], status='PASS', alloc_per_agent_ratio=1.1)
        self.assertEqual(check_paired(receipt), [])

    def test_side64_cannot_hide_missing_or_nonfinite_values(self):
        for field in ('base', 'new', 'ratio'):
            for value in (None, False, 0., -1., float('nan'), float('inf')):
                with self.subTest(field=field, value=value):
                    receipt = fixture()
                    row = next(r for r in receipt['standing_ratios']
                               if r['scene'] == 'n200_side64')
                    row[field] = value
                    with self.assertRaises(EvidenceError):
                        check_paired(receipt)

    def test_discarded_invalid_attempts_are_not_invalid_accepted_members(self):
        receipt = fixture()
        receipt['invalid_attempt_members'] = 2
        self.assertEqual(check_paired(receipt), receipt['failures'])


if __name__ == '__main__':
    unittest.main()
