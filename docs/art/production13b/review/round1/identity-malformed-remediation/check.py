# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run: python3 check.py BEFORE_OR_AFTER
"""Offline CLI rejection checks using deliberately incomplete synthetic evidence."""
import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent
VALIDATOR = Path('/Users/rexxa/work/new-world-art-prod/tools/art_pipeline/validate_walker_identity_p13b.py')
phase = sys.argv[1]
results = []
for case in ('missing', 'bad_schedule', 'bad_running_series'):
    base = ROOT / case
    folders = [base / name for name in ('a', 'aprime', 'b', 'off', 'on')]
    for folder in folders:
        folder.mkdir(parents=True, exist_ok=True)
    if case == 'bad_schedule':
        (folders[0] / 'P1.json').write_text(json.dumps({'capture_frame': 1, 'state_change_frame': 0, 'tick': 1, 'seed': 1, 'fixed_fps': 60}))
    if case == 'bad_running_series':
        (folders[3] / 'W0.json').write_text(json.dumps({'populations': None}))
        (folders[3] / 'W0.jsonl').write_text('')
    output = base / phase
    completed = subprocess.run([sys.executable, '-B', str(VALIDATOR), *map(str, folders), str(output)], capture_output=True, text=True, check=False)
    (base / f'{phase}.stdout').write_text(completed.stdout)
    (base / f'{phase}.stderr').write_text(completed.stderr)
    report = output / 'walker-identity.json'
    data = json.loads(report.read_text()) if report.is_file() else {}
    success = completed.returncode == 1 and data.get('passed') is False and bool(data.get('errors'))
    results.append({'case': case, 'exit': completed.returncode, 'report_written': report.is_file(), 'correct_failure': success})
(ROOT / f'{phase}.json').write_text(json.dumps(results, indent=2) + '\n')
print(json.dumps(results, indent=2))
raise SystemExit(0 if all(row['correct_failure'] for row in results) else 1)
