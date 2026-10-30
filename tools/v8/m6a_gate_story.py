"""Fresh M6a/BASE story-yield completeness, with no numerical acceptance bar."""
from __future__ import annotations

from pathlib import Path

import core_story_check as historical
from m6a_evidence import Node, reference_json, write_json
from m6a_gate_source import BASE, Gate, keys, need, ref


def story(gate: Gate, output: Path) -> None:
    references = gate.data.get('story')
    keys(references, {'candidate', 'base'})
    totals: dict[str, dict[str, int]] = {}
    for arm in ('candidate', 'base'):
        receipt = Node.load(ref(gate.root, references.get(arm)))
        keys(receipt, {'schema_version', 'source_head', 'binary', 'report', 'command', 'exit_code'})
        need(receipt.get('schema_version').integer() == 1 and receipt.get('exit_code').integer() == 0,
             'incomplete story process')
        binary = ref(gate.root, receipt.get('binary'))
        build_key = arm + '_build'
        build = gate.build(build_key, gate.head if arm == 'candidate' else BASE)
        need(binary == ref(gate.root, build.get('probe')), 'story binary not bound to build')
        need(receipt.get('source_head').text() == build.get('source_head').text(), 'story source differs')
        report_path = ref(gate.root, receipt.get('report'))
        report = Node.load(report_path)
        need(report.get('schema_version').integer() == 2 and report.get('scene').text() == 'colony'
             and report.get('days').integer() == 72
             and report.get('source_sha').text() == receipt.get('source_head').text(), 'story report inputs differ')
        need(report.get('inputs').value == historical.INPUTS, 'story world inputs differ')
        need(report.get('definition').value == historical.DEFINITION, 'story metric definition changed')
        command = [row.text() for row in receipt.get('command').rows()]
        need(command[:2] == [str(binary), 'story-yield'], 'story command missing')
        for flag, value in (('--seeds', f'{gate.protocol().first_seed}..{gate.protocol().first_seed + 32}'), ('--days', '72'),
                            ('--steward', 'on'), ('--defence', 'on')):
            need(command.count(flag) == 1 and command[command.index(flag) + 1] == value,
                 'story command differs: ' + flag)
        worlds = report.get('worlds').rows()
        need(sorted(row.get('seed').integer() for row in worlds) == gate.protocol().heldout,
             'story worlds missing or duplicated')
        counts = dict.fromkeys(historical.COUNTS, 0)
        for world in worlds:
            need(world.get('end_tick').integer() == 103680
                 and world.get('ticks_per_year').integer() == 34560
                 and bool(world.get('fingerprint').text()), 'incomplete story world')
            years = world.get('years').rows()
            need(sorted(row.get('year').integer() for row in years) == [0, 1, 2], 'story years incomplete')
            for year in years:
                keys(year, set(historical.COUNTS) | {'year', 'observed_ticks', 'complete_year'})
                need(year.get('observed_ticks').integer() == 34560 and year.get('complete_year').boolean(),
                     'partial story year')
                row = {name: year.get(name).integer() for name in historical.COUNTS}
                need(row['distinct_deep_chains'] <= row['deep_entries'] <= row['entries']
                     and row['people_with_three_entries'] <= row['entries']
                     and row['missing_chains'] + row['cyclic_chains'] <= row['entries']
                     and row['deep_entries'] + row['cyclic_chains'] <= row['entries'], 'impossible story counts')
                for name, count in row.items():
                    counts[name] += count
        totals[arm] = counts
    write_json(output, {'schema_version': 1, 'status': 'MEASURED', 'acceptance_claim': False,
                       'numerical_bar': None, 'worlds_per_arm': 32, 'years_per_world': 3,
                       'totals': totals, 'config': reference_json(gate.path)})
