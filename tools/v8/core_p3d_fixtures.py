"""Complete serialized colony observer shape; no new gameplay outcome bars.

Schema records the frozen switch nine OFF + ON observer shapes. Empty event and
nullable scalar shapes are completed from probe/{demography,economy,knowledge,
m55}.rs and mind_coverage.rs. Dynamic item/event maps are explicit, not a blanket
exception. Scheduling may change values, but cannot remove measured fields.
"""
from __future__ import annotations

import hashlib
import math
from pathlib import Path
import subprocess
import os
import re
import sys

import core_shadow_freeze as freeze
import core_shadow_successor as prior
import mux_safety_validate as safety
import core_why_check as why

SCHEMA = freeze.load(Path(__file__).resolve().with_name('core_p3d_fixture_schema.json'))
SCORERS = ('docs/v8/m4/m4_score.py', 'docs/v8/m5/m5_score.py', 'docs/v8/m5_5/m55_score.py')
VARIANT = '/mux/diagnostics/final_layout/house_orders/*/command/Steward/command'
TUPLES = {'/m4/learned/*': ('int', 'int', 'int', 'str'),
          '/m4/lost/*': ('int', 'str', 'int'), '/m4/at_risk_events/*': ('int', 'str', 'int'),
          '/m4/waits_end/*': ('int', 'int', 'int', 'str'), '/m4/start/at_risk/*': ('str', 'int'),
          '/m4/daily/*/nodes/*': ('int', 'int', 'int'), '/m3/blades_by_agent/*': ('int', 'int'),
          '/m55/neighbours/caravans/*/goods/*': ('str', 'int', 'int'),
          '/m55/neighbours/caravans/*/offered/*': ('str', 'int', 'int')}
need = prior.require


def shape(value: freeze.JSON, path: str = '', *, steward: bool = False) -> None:
    """Require exact fixed fields/types, while allowing declared variable maps."""
    need(type(value).__name__ in SCHEMA['types'].get(path, []), f'unknown/wrong metric type at {path}')
    if isinstance(value, float):
        need(math.isfinite(value), f'nonfinite metric at {path}')
    if isinstance(value, dict):
        dynamic = path in SCHEMA['dynamic_maps']
        if not dynamic:
            expected = set(SCHEMA['objects'][path])
            if path == '' and not steward:
                expected.remove('mux')
            need((len(value) == 1 and set(value) <= expected) if path == VARIANT else set(value) == expected,
                 f'missing/extra observer fields at {path}')
        for key, child in value.items():
            need(type(key) is str and bool(key), f'invalid metric map key at {path}')
            shape(child, path + '/' + ('*' if dynamic else key), steward=steward)
    if isinstance(value, list):
        if path in TUPLES:
            need(tuple(type(item).__name__ for item in value) == TUPLES[path], f'truncated/wrong tuple at {path}')
        for child in value:
            shape(child, path + '/*', steward=steward)


def complete_row(row: freeze.Document, days: int, *, steward: bool = False) -> None:
    shape(row, steward=steward)
    need(type(row['days']) is int and row['days'] == days, 'per-world days differ')
    need(row['first_violation'] is None, 'recorded first invariant violation')
    need(set(row['fingerprints']) == {str(day) for day in range(1, days + 1)}, 'daily fingerprints incomplete')
    need(all(re.fullmatch('[0-9a-f]{16}-[0-9a-f]{16}', value) for value in row['fingerprints'].values()), 'malformed daily fingerprint')
    for block in ('m4', 'm5'):
        need([entry['day'] for entry in row[block]['daily']] == list(range(1, days + 1)), 'daily observer truncated/reordered')
    for block, keys in (('m2', ('firewood_stock_by_day', 'firewood_ground_by_day', 'settlement_wood_by_day')),
                        ('m3', ('bedroom_sleepers', 'knap_top_level_by_day'))):
        need(all(len(row[block][key]) == days for key in keys), 'daily metric vector truncated')
    need([entry['day'] for entry in row['m2']['snapshots']] == [d for d in (3, 10, 20) if d <= days],
         'economy milestone observer incomplete')
    nodes = SCHEMA['knowledge_nodes']
    need(row['m4']['node_names'] == nodes and len(row['m4']['status_end']) == len(nodes), 'knowledge dictionary/status incomplete')
    need(all(len(entry['nodes']) == len(nodes) and len(entry['blades']) == 2 for entry in row['m4']['daily']),
         'daily knowledge node/blade coverage incomplete')
    need(all(len(entry['blades']) == 2 for entry in row['m5']['daily']), 'daily blade pair incomplete')
    for key in ('cold_hours', 'exposure_hours'):
        need(len(row['m5']['apparel'][key]) == 2, 'apparel observer pair incomplete')
    coverage = row['mind_coverage']
    need(coverage['observed_mind_ticks'] == days * 1440 and coverage['status'] != 'N/A', 'mind observer tick coverage incomplete')
    for key in ('covered_records', 'behaviour_covered', 'node_covered', 'critical_covered', 'critical_choice_covered'):
        expected = 'expected_records' if key == 'covered_records' else key.replace('_covered', '_expected')
        need(0 <= coverage[key] <= coverage[expected], 'invalid mind coverage counter relation')
    need(coverage['expected_records'] == coverage['behaviour_expected'] + coverage['node_expected'], 'mind denominator inconsistent')
    need(all(type(coverage[k]) is int and 0 <= coverage[k] < 2**64 for k in why.COUNTERS), 'mind counters malformed')
    need(coverage['first_choices'] > 0 and coverage['behaviour_expected'] == coverage['first_choices'] + coverage['behaviour_switches']
         and coverage['node_expected'] == coverage['node_only_choices'] + coverage['same_behaviour_restarts']
         and coverage['covered_records'] == coverage['behaviour_covered'] + coverage['node_covered']
         and coverage['failures'] == coverage['expected_records'] - coverage['covered_records'], 'mind counter accounting incomplete')
    need(coverage['status'] == ('FAIL' if coverage['failures'] else 'PASS')
         and len(coverage['examples']) == min(coverage['failures'], 16)
         and bool(coverage['definition']), 'mind observer status/examples incomplete')
    for metric, prefix in (('coverage', 'behaviour'), ('node_coverage', 'node'), ('critical_coverage', 'critical'), ('critical_choice_coverage', 'critical_choice')):
        why.check_ratio(coverage[metric], coverage[prefix + '_covered'], coverage[prefix + '_expected'], metric)
    for example in coverage['examples']:
        need(0 <= example['tick'] < days * 1440 and example['agent'] >= 0 and example['node'] >= 0
             and example['problem'] in why.PROBLEMS, 'invalid bounded mind example')
    if steward:
        need(row['mux']['ticks_per_day'] == 1440, 'ON observer tick units differ')


def capture_contract(root: Path, commands: freeze.Document, commit: str) -> None:
    need(set(commands) == {fixture.name for fixture in safety.FIXTURES}, 'missing/extra fixture command receipts')
    for fixture in safety.FIXTURES:
        item = commands[fixture.name]
        prior.fields(item, 'argv exit_code')
        expected = [str(root / 'bin/worldsim_probe'), '--scene', 'colony', '--seeds', f'0..{fixture.count}',
                    '--days', str(fixture.days), '--workers', '4', '--commands', str(root / 'inputs' / fixture.script),
                    '--story', fixture.story, '--neighbours', fixture.neighbours, '--out', str(root / 'colony' / fixture.name)]
        if fixture.determinism == 'yes':
            expected += ['--check-determinism']
        expected += ['--steward', 'off']
        need(item['argv'] == expected and type(item['exit_code']) is int and item['exit_code'] == 0,
             f'fixture command/mode/exit mismatch: {fixture.name}')
        need((root / (fixture.name + '.log')).is_file(), 'missing actual fixture process log')
        document = freeze.load(root / 'colony' / fixture.name / 'summary.json')
        for row in document['seeds']:
            complete_row(row, fixture.days)

    expected_table = ''.join(f'{f.name}\t{f.count}\t{f.days}\t{f.script}\t{f.story}\t{f.neighbours}\t{f.determinism}\n' for f in safety.FIXTURES)
    need((root / 'fixtures.tsv').read_text() == expected_table, 'capture fixture table differs')
    need((root / 'steward_mode.txt').read_text().strip() == 'off (explicit --steward off)', 'capture mode marker differs')
    need((root / 'capture_checkout_sha.txt').read_text().strip() == commit, 'capture checkout differs')
    need((root / 'capture_complete.txt').read_text().strip() == 'MUX_SAFETY capture PASS ' + str(root), 'capture terminal marker incomplete')
    for directory, manifest, names in (('inputs', 'input_sha256.txt', {f.script for f in safety.FIXTURES}), ('bin', 'probe_sha256.txt', {'worldsim_probe'})):
        rows = [line.split() for line in (root / manifest).read_text().splitlines()]
        need(all(len(row) == 2 for row in rows) and len(rows) == len(names) and {row[1] for row in rows} == names, 'missing/duplicate capture hash entries')
        need(all(freeze.digest(root / directory / name) == digest for digest, name in rows), 'capture input/binary hash differs')

def existing_scorers(repo: Path, commit: str, root: Path) -> None:
    """Use standing gate28/29/30 checks, with m4's stdout bar as its exit is always zero."""
    for script in SCORERS:
        committed = prior.git(repo, 'show', commit + ':' + script)
        need(freeze.digest(repo / script) == hashlib.sha256(committed).hexdigest(), 'fixture scorer differs from candidate commit')
    for script, flag, fixture in zip(SCORERS, ('--f10', '--story', '--neighbours'), ('knowledge_f10', 'story', 'neighbours')):
        result = subprocess.run([sys.executable, '-B', str(repo / script), flag,
                                 str(root / 'colony' / fixture / 'summary.json')], cwd=repo, check=True,
                                capture_output=True, text=True, env={**os.environ, 'PYTHONDONTWRITEBYTECODE': '1'})
        print(result.stdout, end='')
        if flag == '--f10':
            need('F10: 8/8 PASS' in result.stdout.splitlines(), 'standing knowledge fixture failed')
