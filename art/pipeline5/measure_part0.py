# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "scipy", "scikit-image"]
# ///
# How to run: uv run --with pillow --with numpy --with scipy --with scikit-image python -m art.pipeline5.measure_part0
"""Record frozen person-family judge v2 numbers for every Part 0 generation."""
from __future__ import annotations

import hashlib
import json
import re
from dataclasses import asdict
from pathlib import Path
from typing import Final, TypedDict

from art.pipeline2.judge import Verdict, judge, load_calibration

ROOT: Final = Path(__file__).resolve().parents[2]
RAW: Final = Path('/Users/rexxa/work/arttrial5-data/generations')
CALIBRATION: Final = ROOT / 'docs/v8/arttrial2/JUDGE_V2_CALIBRATION.json'
OUTPUT: Final = ROOT / 'docs/v8/arttrial5/PART0_METRICS.json'


GenerationInput = TypedDict("GenerationInput", {"candidate": str, "try": int, "output_hint": str})

def main() -> None:
    """Measure unmodified raw outputs and retain missing model review explicitly."""
    calibration = load_calibration(CALIBRATION)
    records: list[tuple[str, int, Verdict]] = []
    for manifest in sorted(RAW.glob('try*.json')):
        rows: list[GenerationInput] = json.loads(manifest.read_text())
        for row in rows:
            match = re.search(r' as (.+\.png) by default', row['output_hint'])
            if match is None:
                message = f'Generator output path missing in {manifest}'
                raise ValueError(message)
            path = Path(match.group(1))
            records.append((row['candidate'], row['try'], judge(path, calibration, family='person')))
    if not records:
        message = f'No generation manifests under {RAW}'
        raise ValueError(message)
    records.append(('B', 1, judge(
        ROOT / 'art/character/candidates/B.png', calibration, family='person')))
    payload = {
        'calibration': str(CALIBRATION.relative_to(ROOT)),
        'calibration_sha256': hashlib.sha256(CALIBRATION.read_bytes()).hexdigest(),
        'note': 'Frozen judge v2 person family. Numeric measurements only; model review MISSING, total acceptance false. Failures retained.',
        'records': [{'name': name, 'attempt': attempt, **asdict(verdict)} for name, attempt, verdict in records],
    }
    _ = OUTPUT.write_text(json.dumps(payload, indent=2) + '\n')
    for name, attempt, verdict in records:
        print(name, attempt, verdict.numeric_pass, verdict.numeric_checks)


if __name__ == '__main__':
    main()
