#!/usr/bin/env python3
"""Extract nonsecret item timing/temperature evidence from completed safe logs."""
import json
import re
import sys
from pathlib import Path

root = Path(sys.argv[1]).resolve(strict=True)
records = []
for path in sorted(root.rglob('*attempt*.log')):
    text = path.read_text()
    admission = re.findall(r'(\S+) admission temperature=(\d+) load=([0-9.]+)', text)
    finish = re.findall(r'(\S+) finished status=(\d+) elapsed_seconds=(\d+)', text)
    output = re.findall(r'OUTPUT=(\S+) ELAPSED_SECONDS=([0-9.]+)', text)
    temperatures = [int(value) for value in re.findall(r'(?:admission|running) temperature=(\d+)', text)]
    records.append({
        'log': str(path.relative_to(root)),
        'admitted_at': admission[0][0] if admission else None,
        'admission_temperature_c': int(admission[0][1]) if admission else None,
        'admission_load_1m': float(admission[0][2]) if admission else None,
        'peak_observed_temperature_c': max(temperatures) if temperatures else None,
        'temperature_samples': len(temperatures),
        'finished_at': finish[-1][0] if finish else None,
        'exit_status': int(finish[-1][1]) if finish else None,
        'guarded_elapsed_seconds_including_30s_cooldown': int(finish[-1][2]) if finish else None,
        'pipeline_elapsed_seconds': float(output[-1][1]) if output else None,
        'raw_output': output[-1][0] if output else None,
        'successful_export': bool(output and finish and finish[-1][1] == '0'),
    })
print(json.dumps(records, indent=2) + '\n', end='')
