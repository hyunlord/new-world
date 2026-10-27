#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
"""P7 timing admission: python3 core_p7_admission.py --log FILE --label NAME [-- COMMAND]."""
from __future__ import annotations

import argparse
from collections.abc import Callable, Sequence
import json
import math
import os
from pathlib import Path
import subprocess
import time

Loads = tuple[float, float, float]


class AdmissionError(RuntimeError):
    """Timing was not admitted; callers must preserve the incomplete run."""


def admit(log: Path, label: str, *, deadline: float | None = None,
          sample: Callable[[], Loads] = os.getloadavg,
          clock: Callable[[], float] = time.time,
          sleep: Callable[[float], None] = time.sleep) -> Loads:
    if deadline is None:
        deadline = float(os.environ['CORE_P7_DEADLINE_EPOCH'])
    if not math.isfinite(deadline):
        raise AdmissionError('non-finite deadline')
    log.parent.mkdir(parents=True, exist_ok=True)
    for window in range(9):
        now = clock()
        loads = sample()
        if len(loads) != 3 or any(not math.isfinite(v) or v < 0 for v in loads):
            raise AdmissionError('malformed 1/5/15 load sample')
        allowed = now < deadline and loads[0] < 8
        with log.open('a') as stream:
            stream.write(json.dumps({'label': label, 'utc': now, 'window': window,
                                     'loads_1_5_15': loads,
                                     'status': 'ADMITTED' if allowed else 'NOT_RUN'}) + '\n')
        if now >= deadline:
            raise AdmissionError('owner hard stop reached')
        if allowed:
            return loads
        if window == 8:
            raise AdmissionError('eight 15-minute windows exhausted')
        if now + 900 >= deadline:
            raise AdmissionError('next 15-minute window exceeds owner hard stop')
        sleep(900)
    raise AssertionError('bounded admission loop must return or raise')


def main(argv: Sequence[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--log', type=Path, required=True)
    parser.add_argument('--label', required=True)
    parser.add_argument('command', nargs=argparse.REMAINDER)
    args = parser.parse_args(argv)
    try:
        loads = admit(args.log, args.label)
    except (AdmissionError, KeyError, ValueError, OSError) as error:
        print(f'P7 NOT_RUN: {error}', flush=True)
        return 75
    print(f'P7 ADMITTED {args.label} load1/5/15={loads}', flush=True)
    command = args.command[1:] if args.command[:1] == ['--'] else args.command
    return subprocess.run(command, check=False).returncode if command else 0


if __name__ == '__main__':
    raise SystemExit(main())
