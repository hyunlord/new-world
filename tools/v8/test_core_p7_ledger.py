"""Lossless same-session ledger tests; no measured binaries or external fixtures."""
import copy
import json
from pathlib import Path
import tempfile
import unittest

import bench_ledger
from core_perf_check import EvidenceError, SCENES
from core_p7_ledger import ledger_text


def summary_fixture():
    return {'commit': 'a' * 40, 'bytes_per_agent': 100.123456789,
            'alloc_per_agent': 100.049, 'bytes_per_tile': 10.123456789,
            'scenes': [{'label': scene, 'agents': 1000 if scene.startswith('n1000') else
                        2000 if scene == 'n2000' else 200,
                        'repeats': 3, 'tick_mean_us': 100.049, 'tick_p99_us': 200.49,
                        'tick_max_us': 301.123456789,
                        'per_repeat': [{'load_before': '7.999 12.5 15.5'} for _ in range(3)]}
                       for scene in SCENES]}


class P7LedgerTests(unittest.TestCase):
    def render(self, summary):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'base-summary.json'
            path.write_text(json.dumps(summary))
            return ledger_text(path)

    def test_exact_commit_all_numeric_precision_and_deterministic_order(self):
        summary = summary_fixture()
        text = self.render(summary)
        rows = bench_ledger.parse_ledger(text)
        self.assertEqual([r['scene'] for r in rows], list(SCENES))
        self.assertEqual({r['commit'] for r in rows}, {summary['commit']})
        row = rows[0]
        for column, expected in [('tick mean (µs)', 100.049), ('p99 (µs)', 200.49),
                                 ('max (µs)', 301.123456789), ('alloc/agent (B)', 100.049),
                                 ('RSS/agent (B)', 100.123456789), ('RSS/tile (B)', 10.123456789)]:
            self.assertEqual(float(row[column]), expected)
        self.assertEqual(row['load (1/5/15 min)'], '; '.join(['7.999 12.5 15.5'] * 3))
        summary['scenes'].reverse()
        self.assertEqual(text, self.render(summary))
        self.assertTrue(text.endswith('\n'))

    def test_precision_changes_boundary_verdict_if_presentation_rounding_is_used(self):
        base = summary_fixture()
        candidate = copy.deepcopy(base)
        for scene in candidate['scenes']:
            scene['tick_mean_us'] = 110.04
        exact = bench_ledger.parse_ledger(self.render(base))
        rounded = bench_ledger.run_rows(base, 'BASE', '', '')
        self.assertEqual(bench_ledger.compare(candidate, exact)[0], [])
        self.assertTrue(bench_ledger.compare(candidate, rounded)[0])
        for scene in candidate['scenes']:
            scene['tick_mean_us'] = 100.049 * 1.1 + 0.000001
        self.assertTrue(bench_ledger.compare(candidate, exact)[0])

    def test_rejects_missing_duplicate_extra_or_contended_scene_evidence(self):
        mutations = [lambda d: d['scenes'].pop(),
                     lambda d: d['scenes'].append(copy.deepcopy(d['scenes'][0])),
                     lambda d: d['scenes'][0].update(label='unknown'),
                     lambda d: d['scenes'][0].update(repeats=2),
                     lambda d: d['scenes'][0]['per_repeat'].pop(),
                     lambda d: d['scenes'][0]['per_repeat'][0].update(load_before='8 1 1'),
                     lambda d: d['scenes'][0]['per_repeat'][0].update(load_before='nan 1 1'),
                     lambda d: d['scenes'][0]['per_repeat'][0].update(load_before='7 1'),
                     lambda d: d['scenes'][0].update(tick_mean_us=float('nan')),
                     lambda d: d.update(commit='invalid|commit')]
        for index, mutate in enumerate(mutations):
            with self.subTest(index=index):
                summary = summary_fixture()
                mutate(summary)
                with self.assertRaises((EvidenceError, ValueError)):
                    self.render(summary)


if __name__ == '__main__':
    unittest.main()
