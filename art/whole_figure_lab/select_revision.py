# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run: python3 -m art.whole_figure_lab.select_revision PRODUCTION ROUND0_JSON ROUND1_JSON
"""Freeze the one authorized revision: exact prior S and whole E attempt two."""
import json
import sys
from pathlib import Path

from .build import sha
from .samples import PEOPLE, STATES


def select(production: Path, prior: Path, output: Path) -> None:
    """Never rank revision attempts or mix frames after observing review results."""
    previous = json.loads(prior.read_text())
    sheets = dict(previous['sheets'])
    natives = dict(previous['native_sheets'])
    records = []
    missing = []
    for key in sorted(sheets):
        look, facing = key.split('/')
        if facing == 'S':
            record = next(row for row in previous['records'] if row['key'] == key)
            if sha(Path(record['processed'])) != record['sha256']:
                raise SystemExit(f'Frozen round0 S evidence changed: {key}')
            records.append(record)
            continue
        path = production / 'processed-v4' / f'{look}_E_try2.json'
        if not path.is_file():
            missing.append(key)
            continue
        raw = json.loads(path.read_text())
        if (raw['look'], raw['facing'], raw['attempt']) != (look, 'E', 2):
            raise SystemExit(f'Revision identity mismatch: {path}')
        directory = Path(raw['diagnostics'])
        native_manifest = Path(raw.get('native_panel_judges', ''))
        if not native_manifest.is_file() or not all((native_manifest.parent / f'{s}_native.png').is_file() for s in STATES):
            missing.append(key)
            continue
        sheets[key] = str(directory)
        natives[key] = str(native_manifest.parent)
        records.append({'key': key, 'attempt': 2,
                        'normalized': sum((directory / f'{s}.png').exists() for s in STATES),
                        'structural_pass': sum(row.get('passed') is True for row in raw['sheet_checks']['frames']),
                        'processed': str(path), 'sha256': sha(path)})
    if missing:
        raise SystemExit('Revision E2 unavailable: ' + ', '.join(missing))
    sample_keys = {f'{p.look}/{f}' for p in PEOPLE for f in ('S', 'E')}
    mode = 'diagnostic' if all(r['normalized'] == 5 for r in records if r['key'] in sample_keys) else 'diagnostic_nonconforming_canvas'
    if output.exists():
        raise SystemExit(f'Frozen revision exists: {output}')
    output.write_text(json.dumps({'rule': 'exact round0 S; force whole E try2; no post-review ranking or mixing',
        'prior_selection': str(prior), 'prior_selection_sha256': sha(prior),
        'mode': mode, 'sheets': sheets, 'native_sheets': natives, 'records': records}, indent=2) + '\n')
    print(mode)


if __name__ == '__main__':
    select(Path(sys.argv[1]), Path(sys.argv[2]), Path(sys.argv[3]))
