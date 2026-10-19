# /// script
# requires-python = ">=3.10"
# dependencies = ["Pillow", "numpy"]
# ///
# Run: python3 tools/art_pipeline/measure_p6.py RAW_ART RAW_LONG GATE_DIR RAW_GOD
"""Run unchanged gameplay thresholds plus the Pass 6 evidence checks."""
from __future__ import annotations

from dataclasses import asdict
import json
from pathlib import Path
import sys

from ingame_qa_p6 import run
from ingame_qa_cold_p6 import cold
from ingame_qa_types import load_capture
from ingame_qa_pixels import Check, result


def long_capture_set(stems: list[str]) -> Check:
    """Require the nine inherited frames and all four prescribed cold nights."""
    from capture_contract_p6 import LONG_STEMS
    actual = set(stems)
    return result('LONG_CAPTURE_SET', actual == LONG_STEMS and len(stems) == len(LONG_STEMS),
        f'{len(stems)}/{len(LONG_STEMS)} captures; missing={sorted(LONG_STEMS-actual)}; extra={sorted(actual-LONG_STEMS)}')


def main(raw: Path, long_raw: Path, gate: Path, god_raw: Path) -> None:
    """Write the aggregate audit without replacing prior-round evidence."""
    from capture_contract_p6 import LONG_STEMS, INHERITED_STEMS
    from ingame_qa_observed_p6 import observed_checks
    output = Path('docs/art/production6')
    blind = output / 'blind/result.json'
    checks = run(raw, output / 'ingame_qa.md', Path('.'), blind if blind.exists() else None, long_raw,
                 extra_long_stems=tuple(LONG_STEMS - INHERITED_STEMS))
    long_manifest = json.loads((long_raw / 'manifest.json').read_text())
    long_check = long_capture_set([Path(row['file']).stem for row in long_manifest['captures']])
    checks = [long_check if check.code == 'LONG_CAPTURE_SET' else check for check in checks]
    frames = []
    for directory in (raw, long_raw):
        manifest = json.loads((directory / 'manifest.json').read_text())
        frames.extend(load_capture(directory / row['file']) for row in manifest['captures']
                      if not row['file'].startswith(('a_calib', 'a_lineup')))
    checks.append(cold(frames, gate))
    checks.extend(observed_checks(frames, god_raw, gate))
    records = [asdict(check) for check in checks]
    integrity = output / 'harvest/integrity_final.json'
    if integrity.exists():
        proof = json.loads(integrity.read_text())
        records.append({'code': 'A20', 'status': ('PASS' if proof.get('passed', False) else 'FAIL'), 'detail': json.dumps(proof)})
    else:
        records.append({'code': 'A20', 'status': 'N/A', 'detail': 'Missing final installed-frame integrity audit'})
    (output / 'measurements.json').write_text(json.dumps(records, ensure_ascii=False, indent=2) + '\n')
    (output / 'measurements.md').write_text('# Art Pass 6 measurements\n\n' + '\n'.join(
        f'## {row["code"]}: {row["status"]}\n\n{row["detail"]}\n' for row in records))
    print('\n'.join(f'{row["code"]}\t{row["status"]}' for row in records))


if __name__ == '__main__':
    main(*(Path(argument) for argument in sys.argv[1:]))
