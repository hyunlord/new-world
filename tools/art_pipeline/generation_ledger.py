from __future__ import annotations

import json
from pathlib import Path
from PIL import Image

ROOT = Path(__file__).resolve().parents[2]


def refresh() -> None:
    path = ROOT / 'docs/art/production/IMAGE_BUDGET.json'
    ledger = json.loads(path.read_text())
    completed = 0
    for entry in ledger['entries']:
        sheet, attempt = entry['sheet'], entry['attempt']
        raw = Path('/Users/rexxa/work/art-production-raw') / sheet / f'attempt_{attempt}.png'
        report = ROOT / f'docs/art/production/sheets/{sheet}_attempt_{attempt}.json'
        if not raw.exists():
            continue
        completed += 1
        entry['status'] = 'generated'
        sidecar = raw.with_suffix('.json')
        provenance = json.loads(sidecar.read_text())
        with Image.open(raw) as image:
            provenance['actual_size'] = list(image.size)
        if report.exists():
            result = json.loads(report.read_text())
            entry.update(status='normalized', accepted=result['accepted'], rejected=result['rejected'])
            provenance['accepted_cells'] = [row['sprite_id'] for row in result['cells'] if row['accepted']]
        provenance['status'] = entry['status']
        sidecar.write_text(json.dumps(provenance, indent=2) + '\n')
    ledger['calls_completed'] = completed
    path.write_text(json.dumps(ledger, indent=2) + '\n')
    print(json.dumps({'calls': ledger['calls_started'], 'completed': completed,
                      'accepted': sum(row.get('accepted', 0) for row in ledger['entries'])}))


if __name__ == '__main__':
    refresh()
