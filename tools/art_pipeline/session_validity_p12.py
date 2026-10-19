#!/usr/bin/env python3
"""Record both admission snapshots and fail closed on contended measurements."""
from __future__ import annotations

import json
import math
import os
import subprocess
import sys
import time
from pathlib import Path


def snapshot() -> dict:
    """Snapshot exactly the top fifteen ps lines; only this wrapper is owned."""
    raw = subprocess.check_output(['ps', '-Ao', 'pid,pcpu,comm', '-r'], text=True)
    lines = raw.splitlines()[:15]
    return {'epoch': time.time(), 'load_1m': os.getloadavg()[0],
            'ps': '\n'.join(lines) + '\n', 'owned_pids': [os.getpid()]}


def valid_snapshot(value: dict) -> bool:
    load = value.get('load_1m')
    if isinstance(load, bool) or not isinstance(load, (float, int)) or not math.isfinite(load) or load >= 6.0:
        return False
    lines = str(value.get('ps', '')).splitlines()
    if len(lines) < 2 or 'PID' not in lines[0]:
        return False
    for line in lines[1:]:
        fields = line.split(None, 2)
        if len(fields) != 3:
            return False
        try:
            pid, cpu = int(fields[0]), float(fields[1])
        except ValueError:
            return False
        if not math.isfinite(cpu) or cpu < 0 or (cpu > 50 and pid not in value.get('owned_pids', [])):
            return False
    return True


def require_valid(path: Path) -> None:
    value = json.loads(path.read_text())
    if not all(valid_snapshot(value.get(key, {})) for key in ('start', 'end')) or value.get('exit_code') != 0:
        raise ValueError('NOT_VERIFIED (contended or unsuccessful): ' + str(path))


def main() -> int:
    output = Path(os.environ['P12_VALIDITY_OUT'])
    if output.exists():
        raise FileExistsError(output)
    output.parent.mkdir(parents=True, exist_ok=True)
    start = snapshot()
    output.write_text(json.dumps({"start": start, "valid": False, "verdict": "INCOMPLETE"}, indent=2) + "\n")
    # Record invalid admission without launching a heavy process.
    code = subprocess.run(sys.argv[1:], check=False).returncode if valid_snapshot(start) else 75
    end = snapshot()
    valid = valid_snapshot(start) and valid_snapshot(end) and code == 0
    output.write_text(json.dumps({'start': start, 'end': end, 'exit_code': code,
                                  'valid': valid, 'verdict': 'VALID' if valid else 'NOT_VERIFIED (contended or unsuccessful)'}, indent=2) + '\n')
    return code if code else (0 if valid else 75)


if __name__ == '__main__':
    raise SystemExit(main())
