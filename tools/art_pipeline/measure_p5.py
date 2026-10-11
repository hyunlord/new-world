# /// script
# requires-python = ">=3.10"
# dependencies = ["Pillow", "numpy"]
# ///
# Run: python3 tools/art_pipeline/measure_p5.py RAW_ART RAW_LONG GATE_DIR
"""Run unchanged gameplay thresholds plus the Pass 5 evidence checks."""
from __future__ import annotations

from dataclasses import asdict
import json
from pathlib import Path
import sys

from ingame_qa import run
from ingame_qa_cold_p5 import cold
from ingame_qa_types import load_capture


def main(raw: Path, long_raw: Path, gate: Path) -> None:
    """Write the aggregate audit without replacing prior-round evidence."""
    output = Path('docs/art/production5')
    blind = output / 'blind/result.json'
    checks = run(raw, output / 'ingame_qa.md', Path('.'), blind if blind.exists() else None, long_raw)
    frames = []
    for directory in (raw, long_raw):
        manifest = json.loads((directory / 'manifest.json').read_text())
        frames.extend(load_capture(directory / row['file']) for row in manifest['captures']
                      if not row['file'].startswith(('a_calib', 'a_lineup')))
    checks.append(cold(frames, gate))
    records = [asdict(check) for check in checks]
    integrity = output / 'harvest/integrity_final.json'
    if integrity.exists():
        proof = json.loads(integrity.read_text())
        records.append({'code': 'A20', 'status': ('PASS' if proof.get('passed', False) else 'FAIL'), 'detail': json.dumps(proof)})
    else:
        records.append({'code': 'A20', 'status': 'N/A', 'detail': 'Missing final installed-frame integrity audit'})
    (output / 'measurements.json').write_text(json.dumps(records, ensure_ascii=False, indent=2) + '\n')
    (output / 'measurements.md').write_text('# Art Pass 5 measurements\n\n' + '\n'.join(
        f'## {row["code"]}: {row["status"]}\n\n{row["detail"]}\n' for row in records))
    print('\n'.join(f'{row["code"]}\t{row["status"]}' for row in records))


if __name__ == '__main__':
    main(*(Path(argument) for argument in sys.argv[1:]))
