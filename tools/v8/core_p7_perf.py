#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
"""P7 paired controller; python3 core_p7_perf.py CONFIG NEW_SESSION.

Uses the original complete-pair protocol and unchanged raw verifier, with P7's
900-second/eight-window admission. Config.controller pins this actual runner.
The original controller remains byte-identical for historical receipt checks.
"""
from __future__ import annotations

import os
from pathlib import Path
import shutil
import subprocess
import sys
import time

from core_p7_admission import AdmissionError, admit
from core_perf_check import (Document, EvidenceError, Json, ROOT, check, loads,
                             sequence, sha, verify_config, verify_file, write)

def run(config_path: Path, out: Path) -> int:
    """Execute only frozen binaries sequentially; never accept half of a pair."""
    config = Document.read(config_path)
    verify_config(config)
    out.mkdir(parents=True, exist_ok=False)
    shutil.copyfile(config_path, out / 'config.json')
    for role in ('base', 'new'):
        (out / 'accepted' / role).mkdir(parents=True)
    (out / 'attempts').mkdir()
    records: list[Json] = []
    for build_only in (False, True):
        for repeat, scene in sequence(build_only):
            accepted = False
            for attempt in range(1, 4):
                pair: list[Json] = []
                roles = ('base', 'new') if repeat % 2 else ('new', 'base')
                for role in roles:
                    label = f'{"rss" if build_only else "timed"}-{scene}-r{repeat}-a{attempt}-{role}'
                    try:
                        external = admit(out / 'admission.jsonl', label)
                    except AdmissionError as error:
                        write(out / 'attempts.json', records + pair)
                        write(out / 'receipt.json', {'status': 'PENDING', 'reason': str(error)})
                        return 75
                    verify_config(config)
                    raw = out / 'attempts' / (label + '.json')
                    argv = [str(verify_file(config.child(role))), 'bench-one', '--scene-label', scene,
                            '--out', str(raw), '--commands', str(verify_file(config.child('commands')))]
                    if build_only:
                        argv += ['--build-only', str(repeat)]
                    environment = os.environ.copy()
                    environment.pop('WORLDSIM_MIND_HISTORY_SATURATE', None)
                    if role == 'new' and not build_only:
                        environment['WORLDSIM_MIND_HISTORY_SATURATE'] = '1'
                    started = time.time()
                    with (out / 'attempts' / (label + '.log')).open('w') as stream:
                        result = subprocess.run(argv, cwd=ROOT, env=environment, stdout=stream, stderr=subprocess.STDOUT, check=False)
                    ended = time.time()
                    verify_config(config)
                    internal = None
                    parse_error = None
                    if result.returncode == 0 and raw.exists():
                        try:
                            internal = loads(Document.read(raw).text('load_before'))
                        except (EvidenceError, ValueError) as error:
                            parse_error = str(error)
                    valid = internal is not None and internal[0] < 8
                    row: dict[str, Json] = {'role': role, 'scene': scene, 'repeat': repeat, 'attempt': attempt,
                        'build_only': build_only, 'started': started, 'ended': ended, 'external_load': list(external),
                        'internal_load': list(internal) if internal else None, 'exit_code': result.returncode,
                        'raw': str(raw.resolve()), 'sha256': sha(raw) if raw.exists() else None,
                        'status': 'VALID' if valid else 'INVALID', 'pair_accepted': False, 'parse_error': parse_error}
                    pair.append(row)
                accepted = all(Document.parse(row).text('status') == 'VALID' for row in pair)
                for value in pair:
                    row = dict(Document.parse(value).values)
                    row['pair_accepted'] = accepted
                    records.append(row)
                    if accepted:
                        role = Document.parse(row).text('role')
                        shutil.copyfile(Document.parse(row).text('raw'), out / 'accepted' / role / f'{"rss" if build_only else "timed"}-{scene}-r{repeat}.json')
                write(out / 'attempts.json', records)
                if accepted:
                    break
            if not accepted:
                write(out / 'receipt.json', {'status': 'PENDING', 'reason': 'three invalid pair attempts retained'})
                return 75
    for role in ('base', 'new'):
        subprocess.run([str(verify_file(config.child(role))), 'bench-summarise', '--runs', str(out / 'accepted' / role),
                        '--out', str(out / f'{role}-summary.json'), '--commit', config.text('base_head' if role == 'base' else 'candidate_head')], cwd=ROOT, check=True)
    return check(out)


if __name__ == '__main__':
    if len(sys.argv) != 3:
        raise SystemExit('usage: core_p7_perf.py CONFIG NEW_SESSION')
    raise SystemExit(run(Path(sys.argv[1]), Path(sys.argv[2])))
