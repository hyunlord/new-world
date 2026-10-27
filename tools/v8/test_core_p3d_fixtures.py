"""Synthetic complete observers and malformed capture controls; no world runs."""
import json
from pathlib import Path
import tempfile
import unittest
from unittest import mock

import core_p3d_fixtures as f


def metric(path):
    kinds = f.SCHEMA['types'][path]
    if 'NoneType' in kinds:
        return None
    if 'dict' in kinds:
        return {} if path in f.SCHEMA['dynamic_maps'] else {key: metric(path + '/' + key) for key in f.SCHEMA['objects'][path]}
    if 'list' in kinds:
        return []
    if 'str' in kinds:
        return 'synthetic'
    if 'bool' in kinds:
        return False
    return 0.0 if 'float' in kinds else 0


def complete(days=4, steward=False):
    row = metric('')
    if not steward:
        row.pop('mux')
    else:
        row['mux']['ticks_per_day'] = 1440
    row['days'] = days
    row['fingerprints'] = {str(day): '0000000000000000-0000000000000000' for day in range(1, days + 1)}
    for block in ('m4', 'm5'):
        row[block]['daily'] = []
        for day in range(1, days + 1):
            entry = metric('/' + block + '/daily/*')
            entry.update(day=day, blades=[0, 0])
            if block == 'm4':
                entry['nodes'] = [[0, 0, 0] for _ in f.SCHEMA['knowledge_nodes']]
            row[block]['daily'].append(entry)
    row['m4']['node_names'] = f.SCHEMA['knowledge_nodes'].copy()
    row['m4']['status_end'] = [0] * len(row['m4']['node_names'])
    for block, keys in (('m2', ('firewood_stock_by_day', 'firewood_ground_by_day', 'settlement_wood_by_day')),
                        ('m3', ('bedroom_sleepers', 'knap_top_level_by_day'))):
        for key in keys:
            row[block][key] = [0] * days
    row['m2']['snapshots'] = [{**metric('/m2/snapshots/*'), 'day': d} for d in (3, 10, 20) if d <= days]
    row['m5']['apparel']['cold_hours'] = [0, 0]
    row['m5']['apparel']['exposure_hours'] = [0, 0]
    c = row['mind_coverage']
    c.update(observed_mind_ticks=days * 1440, status='PASS', first_choices=1, expected_records=1,
             covered_records=1, behaviour_expected=1, behaviour_covered=1, coverage=1.0,
             node_coverage=None, critical_coverage=None, critical_choice_coverage=None)
    return row


class FixtureTests(unittest.TestCase):
    def test_complete_synthetic_off_and_on(self):
        for mode in (False, True):
            f.complete_row(complete(steward=mode), 4, steward=mode)

    def test_minimal_metrics_rejected_in_off_and_on(self):
        for mode in (False, True):
            with self.assertRaises(f.freeze.FreezeError):
                f.complete_row({'seed': 0, 'ledger_violation_ticks': 0, 'reservation_violation_ticks': 0}, 4, steward=mode)

    def test_removed_nested_field_rejected(self):
        row = complete()
        del row['m5']['daily'][0]['deer']
        with self.assertRaises(f.freeze.FreezeError):
            f.complete_row(row, 4)

    def test_truncated_daily_fingerprint_and_mind_observer_rejected(self):
        for modify in (lambda r: r['m4']['daily'].pop(), lambda r: r['fingerprints'].pop('4'),
                       lambda r: r['mind_coverage'].update(observed_mind_ticks=5759),
                       lambda r: r['m2']['firewood_stock_by_day'].pop(),
                       lambda r: r['m4']['daily'][0]['nodes'].pop()):
            row = complete()
            modify(row)
            with self.assertRaises(f.freeze.FreezeError):
                f.complete_row(row, 4)

    def test_false_in_numeric_metric_and_nonfinite_rejected(self):
        for key, value in (('alive_end', True), ('alive_end', float('nan'))):
            row = complete()
            row[key] = value
            with self.assertRaises(f.freeze.FreezeError):
                f.complete_row(row, 4)

    def test_missing_on_observer_rejected(self):
        with self.assertRaises(f.freeze.FreezeError):
            f.complete_row(complete(), 4, steward=True)

    def test_observer_ratio_does_not_hide_truncation(self):
        row = complete()
        row['mind_coverage']['coverage'] = 0.5
        with self.assertRaises(f.why.ScoreError):
            f.complete_row(row, 4)

    def commands(self, root):
        result = {}
        for fixture in f.safety.FIXTURES:
            argv = [str(root / 'bin/worldsim_probe'), '--scene', 'colony', '--seeds', f'0..{fixture.count}',
                    '--days', str(fixture.days), '--workers', '4', '--commands', str(root / 'inputs' / fixture.script),
                    '--story', fixture.story, '--neighbours', fixture.neighbours, '--out', str(root / 'colony' / fixture.name)]
            if fixture.determinism == 'yes':
                argv += ['--check-determinism']
            result[fixture.name] = {'argv': argv + ['--steward', 'off'], 'exit_code': 0}
        return result

    def test_missing_wrong_mode_or_nonzero_exit_receipt_rejected(self):
        root = Path('/synthetic')
        for mutate in (lambda d: d.pop('soak'), lambda d: d['soak']['argv'].__setitem__(-1, 'on'),
                       lambda d: d['soak'].update(exit_code=1), lambda d: d['soak'].update(exit_code=False)):
            commands = self.commands(root)
            mutate(commands)
            with self.assertRaises(f.freeze.FreezeError):
                f.capture_contract(root, commands, 'commit')

    def test_correct_commands_still_require_actual_process_log(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            with self.assertRaises(f.freeze.FreezeError):
                f.capture_contract(root, self.commands(root), 'commit')

    def test_minimal_rows_pass_old_coverage_but_fail_complete_schema(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            fixture = f.safety.FIXTURES[0]
            directory = root / 'colony' / fixture.name
            directory.mkdir(parents=True)
            rows = [{'seed': seed, 'ledger_violation_ticks': 0, 'reservation_violation_ticks': 0} for seed in range(fixture.count)]
            (directory / 'summary.json').write_text(json.dumps({'scene': 'colony', 'days': fixture.days, 'seeds': rows}))
            f.safety.validate_fixture(root, fixture)
            with self.assertRaises(f.freeze.FreezeError):
                f.complete_row(rows[0], fixture.days)

    def test_complete_synthetic_capture_metadata_and_missing_hash_manifest(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            (root / 'inputs').mkdir()
            (root / 'bin').mkdir()
            (root / 'bin/worldsim_probe').write_text('synthetic, never executable')
            inputs = {item.script for item in f.safety.FIXTURES}
            for name in inputs:
                (root / 'inputs' / name).write_text('synthetic input')
            for directory, filename, names in [('inputs', 'input_sha256.txt', inputs), ('bin', 'probe_sha256.txt', {'worldsim_probe'})]:
                (root / filename).write_text(''.join(f.freeze.digest(root / directory / name) + '  ' + name + '\n' for name in sorted(names)))
            (root / 'fixtures.tsv').write_text(''.join(f'{x.name}\t{x.count}\t{x.days}\t{x.script}\t{x.story}\t{x.neighbours}\t{x.determinism}\n' for x in f.safety.FIXTURES))
            (root / 'steward_mode.txt').write_text('off (explicit --steward off)\n')
            (root / 'capture_checkout_sha.txt').write_text('commit\n')
            (root / 'capture_complete.txt').write_text('MUX_SAFETY capture PASS ' + str(root) + '\n')
            for item in f.safety.FIXTURES:
                directory = root / 'colony' / item.name
                directory.mkdir(parents=True)
                rows = [{**complete(item.days), 'seed': seed} for seed in range(item.count)]
                (directory / 'summary.json').write_text(json.dumps({'scene': 'colony', 'days': item.days, 'seeds': rows}))
                (root / (item.name + '.log')).write_text('synthetic process log, not runtime proof')
            commands = self.commands(root)
            f.capture_contract(root, commands, 'commit')
            (root / 'input_sha256.txt').unlink()
            with self.assertRaises(OSError):
                f.capture_contract(root, commands, 'commit')

    def test_knowledge_zero_exit_without_pass_marker_rejected(self):
        with mock.patch.object(f.prior, 'git', return_value=b'script'), mock.patch.object(f.freeze, 'digest', return_value=f.hashlib.sha256(b'script').hexdigest()), mock.patch.object(f.subprocess, 'run', return_value=mock.Mock(stdout='F10: 0/8 FAIL\n')):
            with self.assertRaises(f.freeze.FreezeError):
                f.existing_scorers(Path('/repo'), 'commit', Path('/capture'))


if __name__ == '__main__':
    unittest.main()
