# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run: python3 -m art.whole_figure_lab.select PRODUCTION OUTPUT_JSON
"""Freeze one complete attempt per preregistered identity using structural counts."""
import json
import sys
from pathlib import Path

from .build import sha
from .samples import PEOPLE, STATES


def select(production: Path, output: Path) -> None:
    """No visual ranking, frame mixing or substitution; missing identities abort."""
    keys = {(p.look, facing) for p in PEOPLE for facing in ('S', 'E')}
    keys.update({('adult_M_hide_wrap_short','S'),('adult_M_hide_wrap_short','E'),
                 ('adult_M_hide_wrap_long_tied','S'),('adult_M_loincloth_short','S'),
                 ('adult_M_fur_cloak_short','E'),('adult_M_hide_wrap_long_tied','E')})
    sheets = {}
    natives = {}
    records = []
    missing = []
    for look, facing in sorted(keys):
        candidates = []
        for path in sorted((production/'processed-v4').glob(f'{look}_{facing}_try*.json')):
            raw = json.loads(path.read_text())
            if (raw['look'], raw['facing']) != (look, facing):
                continue
            directory = Path(raw['diagnostics'])
            normalized = sum((directory/f'{state}.png').exists() for state in STATES)
            passed = sum(row.get('passed') is True for row in raw['sheet_checks']['frames'])
            native_manifest = Path(raw.get('native_panel_judges',''))
            if not native_manifest.is_file() or not all((native_manifest.parent/f'{s}_native.png').is_file() for s in STATES):
                continue
            candidates.append((normalized, passed, -raw['attempt'], path, raw))
        if not candidates:
            missing.append(f'{look}/{facing}')
            continue
        normalized, passed, _, path, raw = max(candidates,key=lambda c:c[:3])
        key=f'{look}/{facing}'
        sheets[key]=raw['diagnostics'];natives[key]=str(Path(raw['native_panel_judges']).parent)
        records.append({'key':key,'attempt':raw['attempt'],'normalized':normalized,'structural_pass':passed,
                        'processed':str(path),'sha256':sha(path)})
    if missing:
        raise SystemExit('Missing complete owned-native identity: '+', '.join(missing))
    sample_keys={f'{p.look}/{f}' for p in PEOPLE for f in ('S','E')}
    mode='diagnostic' if all(r['normalized']==5 for r in records if r['key'] in sample_keys) else 'diagnostic_nonconforming_canvas'
    if output.exists():
        raise SystemExit(f'Frozen selection exists; refusing overwrite: {output}')
    output.write_text(json.dumps({'rule':'max normalized count; then max structure passes; then lower attempt; whole sheet only',
                                 'mode':mode,'sheets':sheets,'native_sheets':natives,'records':records},indent=2)+'\n')
    print(mode)


if __name__=='__main__':
    select(Path(sys.argv[1]),Path(sys.argv[2]))
