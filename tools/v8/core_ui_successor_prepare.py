"""Seal UI successor configuration after the final commit; never run simulations.

Usage: core_ui_successor_prepare.py NEW_EXTERNAL_DIR FRESH_PERF_SESSION
"""
from __future__ import annotations

import json
from pathlib import Path
import shlex
import sys

import core_ui_successor as ui

HISTORY = Path('/Users/rexxa/work/core-data/p7-execution-3')


def prepare(output: Path, fresh_session: Path) -> None:
    """Bind published evidence, exact final delta and independently pinned config."""
    root = Path(__file__).resolve().parents[2]
    output = ui.freeze.external(root, output)
    fresh_session = ui.freeze.external(root, fresh_session)
    output.mkdir(parents=True, exist_ok=False)
    head = ui.inventory.git(root, 'rev-parse', 'HEAD').decode().strip()
    paths = {'gate': HISTORY / 'full-gate/gate.txt', 'terminal': HISTORY / 'full-gate-launch/terminal.json',
             'evaluation': HISTORY / 'core-eval-config.json', 'shadow': HISTORY / 'core-shadow-config.json'}
    data: ui.freeze.Document = {
        'schema_version': 1, 'kind': 'published_ui_successor', 'head': head, 'base': ui.BASE,
        'rust_sha256': ui.perf.source_hash(), 'lock_sha256': ui.freeze.digest(root / 'rust/Cargo.lock'),
        'delta': ui.exact_delta(ui.inventory.committed(root, ui.BASE, '.'),
                               ui.inventory.committed(root, head, '.')),
        'history': {key: ui.reference(path) for key, path in paths.items()},
        'fresh_session': str(fresh_session),
    }
    ui.source(root, data)
    evaluation, _ = ui.history(root, data)
    evaluation['final_head'] = head
    evaluation_path = output / 'evaluation.json'
    config_path = output / 'successor.json'
    for path, content in ((evaluation_path, evaluation), (config_path, data)):
        with path.open('x') as stream:
            json.dump(content, stream, indent=2, sort_keys=True)
            stream.write('\n')
    settings = {
        'CORE_UI_CONFIG': str(config_path), 'CORE_UI_CONFIG_SHA256': ui.freeze.digest(config_path),
        'CORE_EVAL_CONFIG': str(evaluation_path), 'CORE_EVAL_CONFIG_SHA256': ui.freeze.digest(evaluation_path),
        'CORE_SHADOW_CONFIG': str(paths['shadow']), 'CORE_SHADOW_CONFIG_SHA256': ui.PINS['shadow'],
        'PYTHONDONTWRITEBYTECODE': '1',
    }
    with (output / 'env.sh').open('x') as stream:
        stream.write('unset CORE_P7_RUN CORE_P7_VERIFIER_VIEW BENCH_JUSTIFIED BENCH_JUSTIFIED_SCENES\n')
        for key, value in settings.items():
            stream.write('export ' + key + '=' + shlex.quote(value) + '\n')
    print('UI_SUCCESSOR_CONFIG sealed ' + head)
    print('Fresh session remains required: ' + str(fresh_session))


if __name__ == '__main__':
    if len(sys.argv) != 3:
        raise SystemExit('usage: core_ui_successor_prepare.py NEW_EXTERNAL_DIR FRESH_PERF_SESSION')
    prepare(Path(sys.argv[1]), Path(sys.argv[2]))
