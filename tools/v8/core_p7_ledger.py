"""Render only the fresh paired BASE as a lossless benchmark comparison ledger."""
from pathlib import Path
import re

from bench_ledger import COLUMNS, format_row
from core_perf_check import Document, SCENES, loads, require


def cell(document: Document, key: str) -> str:
    """Round-trip the finite binary64 value used by the unchanged ledger checker."""
    return repr(document.number(key))


def ledger_text(summary: Path) -> str:
    """Deterministic six-scene table; this does not replace raw pair authentication."""
    doc = Document.read(summary)
    commit = doc.text('commit')
    require(re.fullmatch(r'[0-9a-f]{40}', commit) is not None, 'BASE needs full commit SHA')
    scenes = doc.rows('scenes')
    by_name = {row.text('label'): row for row in scenes}
    require(len(scenes) == len(SCENES) and set(by_name) == set(SCENES),
            'BASE needs exactly six unique expected scenes')
    rows = ['# Same-session interleaved BASE comparison', '',
            '| ' + ' | '.join(COLUMNS) + ' |',
            '| ' + ' | '.join('---' for _ in COLUMNS) + ' |']
    for name in SCENES:
        scene = by_name[name]
        require(type(scene.values.get('repeats')) is int and scene.number('repeats') == 3,
                'BASE needs three repeats')
        repeats = scene.rows('per_repeat')
        require(len(repeats) == 3, 'BASE needs three repeat load samples')
        samples = [repeat.text('load_before') for repeat in repeats]
        for sample in samples:
            require('|' not in sample and '\n' not in sample and '\r' not in sample,
                    'invalid ledger load cell')
            require(loads(sample)[0] < 8, 'contended BASE repeat')
        for metric in ('tick_mean_us', 'tick_p99_us', 'tick_max_us'):
            require(scene.number(metric) > 0, 'BASE time must be positive')
        expected_agents = 1000 if name.startswith('n1000') else 2000 if name == 'n2000' else 200
        require(scene.number('agents') == expected_agents, 'BASE scene agent count differs')
        row = dict.fromkeys(COLUMNS, '')
        row.update({'commit': commit, 'pass': 'P7-SAME-SESSION-BASE', 'N': str(expected_agents),
                    'scene': name, 'tick mean (µs)': cell(scene, 'tick_mean_us'),
                    'p99 (µs)': cell(scene, 'tick_p99_us'), 'max (µs)': cell(scene, 'tick_max_us'),
                    'RSS/agent (B)': cell(doc, 'bytes_per_agent'),
                    'alloc/agent (B)': cell(doc, 'alloc_per_agent'),
                    'RSS/tile (B)': cell(doc, 'bytes_per_tile'),
                    'load (1/5/15 min)': '; '.join(samples),
                    'note': 'same-session-interleaved-base; raw paired receipt required'})
        rows.append(format_row(row))
    return '\n'.join(rows) + '\n'
