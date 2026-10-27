#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
"""Run P7's fresh paired step25: python3 core_p7_bench.py EXTERNAL_GATE_BENCH_DIR."""
from __future__ import annotations

import os
from pathlib import Path
import subprocess
import sys
import time

from core_p7_acceptance import check_paired
from core_p7_perf import run
from core_p7_ledger import ledger_text
from core_perf_check import Document, EvidenceError, ROOT, git, require, sha, write


def reference(path: Path) -> dict[str, str | int]:
    return {'path': str(path.resolve()), 'sha256': sha(path), 'size': path.stat().st_size}


def main(out: Path) -> int:
    scope = Path(os.environ['CORE_P7_RUN']).resolve()
    require(out.resolve() == scope / 'full-gate/bench', 'P7 bench must belong to this gate')
    require(not out.exists(), 'P7 bench never resumes or reuses output')
    head = git('rev-parse', 'HEAD')
    require((scope / 'publication-head.txt').read_text().strip() == head, 'publication head differs')
    gate = scope / 'full-gate/gate.txt'
    require(gate.read_text().splitlines()[0] == 'head ' + head, 'gate head differs')
    quote = os.environ['BENCH_JUSTIFIED']
    require(quote in (ROOT / 'CHANGELOG.md').read_text(), 'justification absent from CHANGELOG')
    require(os.environ['BENCH_JUSTIFIED_SCENES'] == 'n200_side64', 'only side64 may be justified')
    out.mkdir(parents=True)
    started = time.time()
    config = Path(os.environ['CORE_P7_PERF_CONFIG'])
    code = run(config, out / 'perf-session')
    if code not in (0, 1):
        print('P7 NOT_COMPLETE: paired measurement exit', code, flush=True)
        return code
    receipt = Document.read(out / 'perf-session/receipt.json')
    paired_error = None
    waived: list[str] = []
    try:
        waived = check_paired(receipt)
    except EvidenceError as error:
        paired_error = str(error)
    comparison_ledger = out / 'comparison-ledger.md'
    with comparison_ledger.open('x') as stream:
        stream.write(ledger_text(out / 'perf-session/base-summary.json'))
    command = [sys.executable, str(ROOT / 'tools/v8/bench_ledger.py'), 'check',
               str(out / 'perf-session/new-summary.json'), str(comparison_ledger),
               '--pass', 'M-CORE',
               '--justified', quote, '--justified-scenes', 'n200_side64']
    ledger = subprocess.run(command, cwd=ROOT, check=False)
    accepted = paired_error is None and ledger.returncode == 0
    require(git('rev-parse', 'HEAD') == head, 'head changed during P7 timing')
    write(out / 'p7-adjudication.json', {
        'schema_version': 1, 'status': 'JUSTIFIED' if accepted else 'FAIL',
        'publication_head': head, 'gate': str(gate), 'started': started, 'finished': time.time(),
        'paired_receipt': reference(out / 'perf-session/receipt.json'),
        'paired_config': reference(out / 'perf-session/config.json'),
        'candidate_summary': reference(out / 'perf-session/new-summary.json'),
        'base_summary': reference(out / 'perf-session/base-summary.json'),
        'comparison_basis': 'same-session-interleaved-base',
        'comparison_ledger': reference(comparison_ledger),
        'justification': quote, 'justified_scenes': ['n200_side64'],
        'paired_error': paired_error, 'raw_failure_labels': waived,
        'ledger_exit_code': ledger.returncode,
    }, exclusive=True)
    if not accepted:
        print('P7 STOP: another standing scene or C3/prerequisite failed:', paired_error,
              'ledger exit', ledger.returncode, flush=True)
        return 1
    template = Document.read(Path(os.environ['CORE_P7_EVAL_TEMPLATE']))
    evaluation = dict(template.values)
    evaluation['final_head'] = head
    evaluation['perf_config'] = reference(out / 'perf-session/config.json')
    evaluation['perf_receipt'] = reference(out / 'perf-session/receipt.json')
    write(Path(os.environ['CORE_EVAL_CONFIG']), evaluation, exclusive=True)
    print('P7 JUSTIFIED: side64 exception only; other standing scenes and C3 pass', flush=True)
    return 0


if __name__ == '__main__':
    if len(sys.argv) != 2:
        raise SystemExit('usage: core_p7_bench.py EXTERNAL_GATE_BENCH_DIR')
    raise SystemExit(main(Path(sys.argv[1])))
