# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "scipy", "scikit-image"]
# ///
# How to run: uv run --with pillow --with numpy --with scipy --with scikit-image python -m art.pipeline5.process_sheets [EXTERNAL_PRODUCTION_ROOT]
"""Reentrant processing of completed generation results; never generates images."""
from __future__ import annotations

import json
import re
import shutil
import subprocess
import sys
import time
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Final

from PIL import Image

from art.pipeline2.judge import judge, load_calibration
from art.pipeline5.attempt_integrity import audit_cached, digest, verify_judge
from art.pipeline5.sheet_geometry import split_panels
from art.pipeline5.whole_checks import FRAMES

ROOT: Final = Path(__file__).resolve().parents[2]
CALIBRATION: Final = ROOT/'docs/v8/arttrial2/JUDGE_V2_CALIBRATION.json'
DEFAULT: Final = Path('/Users/rexxa/work/arttrial5-data/production')


@dataclass(frozen=True, slots=True)
class Dispatch:
    look: str
    age: str
    facing: str
    attempt: int
    prompt: str

    @classmethod
    def read(cls, path: Path) -> Dispatch:
        """Require constrained identities before constructing output paths."""
        data = json.loads(path.read_text())
        item = cls(data['look'], data['age'], data['facing'], data['attempt'], data['prompt'])
        if (re.fullmatch(r'[A-Za-z0-9_]+', item.look) is None or item.age not in ('adult', 'child')
                or item.facing not in ('S', 'E') or type(item.attempt) is not int
                or item.attempt not in (1, 2) or not item.prompt.strip()):
            message = f'Invalid dispatch manifest: {path}'
            raise ValueError(message)
        return item


def panel_judges(raw: Path, directory: Path) -> Path:
    """Measure native whole-person crops; retain full-sheet gate unchanged."""
    output = directory/'native_panel_judges.json'
    if output.exists():
        return output
    directory.mkdir(parents=True, exist_ok=True)
    records = []
    with Image.open(raw) as image:
        if image.mode != 'RGBA' or image.size != (1536, 1024):
            message = f'Native panel judge requires1536x1024 RGBA; got{image.size} {image.mode}'
            raise ValueError(message)
        panels = split_panels(image)
    calibration = load_calibration(CALIBRATION)
    for frame, image, box in zip(FRAMES, panels.images, panels.boxes, strict=True):
        path = directory/(frame+'_native.png')
        image.save(path)
        try:
            verdict = asdict(judge(path, calibration, family='person'))
            records.append({'frame': frame, 'source_box': box, 'verdict': verdict})
        except (ValueError, OSError) as error:
            records.append({'frame': frame, 'source_box': box, 'error': str(error)})
    _ = output.write_text(json.dumps({
        'purpose': 'SUPPLEMENTARY_ONLY: raw full-sheet acceptance remains unchanged',
        'raw_sha256': digest(raw), 'calibration_sha256': digest(CALIBRATION),
        'records': records}, indent=2)+'\n')
    return output


def process_result(result_path: Path, production: Path) -> bool:
    """Persist one terminal processing record, including retained errors."""
    base = result_path.name.removesuffix('_result.json')
    records = production/'processed-v4'
    records.mkdir(parents=True, exist_ok=True)
    record_path = records/(base+'.json')
    if record_path.exists():
        audit = {'record': str(record_path), 'record_sha256': digest(record_path), 'passed': False}
        try:
            binding = audit_cached(record_path, result_path, ROOT/'art/character/route_i')
            audit.update({'passed': True, 'judge_binding': asdict(binding)})
        except (OSError, ValueError, KeyError, TypeError) as error:
            audit['error'] = str(error)
        audits = production/'cache-audits-v4'/base
        audits.mkdir(parents=True, exist_ok=True)
        _ = (audits/f'{time.time_ns()}.json').write_text(json.dumps(audit, indent=2)+'\n')
        if not audit['passed']:
            print(base, 'CACHE_IDENTITY_ERROR', audit.get('error'), flush=True)
        return audit['passed'] is True
    record = {'result': str(result_path), 'result_sha256': digest(result_path),
              'status': 'ERROR', 'accepted': False, 'panel_detection_version': 3, 'normalization_version': 4}
    try:
        dispatch_path = result_path.with_name(base+'.json')
        item = Dispatch.read(dispatch_path)
        record.update({'look': item.look, 'age': item.age, 'facing': item.facing,
                       'attempt': item.attempt, 'dispatch_sha256': digest(dispatch_path)})
        result = json.loads(result_path.read_text())
        match = re.search(r' as (.+\.png) by default', result['output_hint'])
        if match is None:
            message = 'Missing generated PNG in output_hint'
            raise ValueError(message)
        original = Path(match.group(1))
        sheets = production/'sheets'
        sheets.mkdir(parents=True, exist_ok=True)
        raw = sheets/(base+'.png')
        if raw.exists():
            if digest(raw) != digest(original):
                message = f'Preserved raw copy differs from original: {raw}'
                raise ValueError(message)
        else:
            _ = shutil.copyfile(original, raw)
        if digest(raw) != digest(original):
            message = f'Byte-copy verification failed: {raw}'
            raise ValueError(message)
        prompt = sheets/(base+'.txt')
        if prompt.exists() and prompt.read_text() != item.prompt:
            message = f'Prompt changed: {prompt}'
            raise ValueError(message)
        if not prompt.exists():
            _ = prompt.write_text(item.prompt)
        judge_path = sheets/(base+'_judge.json')
        if not judge_path.exists():
            verdict = judge(raw, load_calibration(CALIBRATION), family='person')
            _ = judge_path.write_text(json.dumps(asdict(verdict), indent=2)+'\n')
        record['judge_binding'] = asdict(verify_judge(judge_path, raw))
        try:
            record['native_panel_judges'] = str(panel_judges(raw, production/'panel-judges-v4'/base))
        except (ValueError, OSError) as error:
            record['native_panel_judge_error'] = str(error)
        diagnostic = production/'slices-v4'/item.look/item.facing/f'try{item.attempt}'
        check_path = diagnostic/'sheet_checks.json'
        record.update({'raw': str(raw), 'raw_sha256': digest(raw), 'original': str(original),
                       'prompt_sha256': digest(prompt), 'judge': str(judge_path),
                       'calibration_sha256': digest(CALIBRATION), 'diagnostics': str(diagnostic)})
        if not check_path.exists():
            args = [sys.executable, '-m', 'art.pipeline5.sheet_slice', '--raw', str(raw),
                    '--look', item.look, '--age', item.age, '--facing', item.facing,
                    '--attempt', str(item.attempt), '--prompt', str(prompt), '--judge', str(judge_path),
                    '--output-root', str(ROOT/'art/character/route_i'),
                    '--diagnostics-root', str(production/'slices-v4')]
            run = subprocess.run(args, cwd=ROOT, text=True, capture_output=True, check=False)
            _ = (sheets/(base+'_slice_v4.log')).write_text(run.stdout+run.stderr)
            record['slice_exit_code'] = run.returncode
        if not check_path.exists():
            message = f'Slicer produced no complete diagnostic: {diagnostic}'
            raise ValueError(message)
        checks = json.loads(check_path.read_text())
        if checks['raw_sha256'] != digest(raw) or checks['prompt_sha256'] != digest(prompt):
            message = f'Existing diagnostics do not match source identity: {diagnostic}'
            raise ValueError(message)
        accepted = checks['passed'] is True
        record.update({'status': 'ACCEPTED' if accepted else 'REJECTED', 'accepted': accepted,
                       'numeric_pass': checks['judge']['numeric_pass'], 'sheet_checks': checks})
    except (OSError, ValueError, KeyError, TypeError) as error:
        record['error'] = str(error)
    _ = record_path.write_text(json.dumps(record, indent=2)+'\n')
    print(base, record['status'], flush=True)
    return process_result(result_path, production) if record['status'] != 'ERROR' else False


def main() -> int:
    """Process current results once; report missing and rejected attempts separately."""
    production = Path(sys.argv[1]) if len(sys.argv) > 1 else DEFAULT
    results = sorted((production/'attempts').glob('*_result.json'))
    audit_ok = [process_result(result, production) for result in results]
    dispatches = [p for p in sorted((production/'attempts').glob('*.json'))
                  if not p.name.endswith('_result.json')]
    missing = [str(p) for p in dispatches if not p.with_name(p.stem+'_result.json').exists()]
    records = [json.loads(p.read_text()) for p in sorted((production/'processed-v4').glob('*.json'))]
    summary = {'calibration': str(CALIBRATION), 'calibration_sha256': digest(CALIBRATION),
               'result_count': len(results), 'processed_count': len(records),
               'current_identity_error_count': sum(not passed for passed in audit_ok),
               'accepted_count': sum(row['accepted'] is True for row in records),
               'rejected_count': sum(row['status'] == 'REJECTED' for row in records),
               'error_count': sum(row['status'] == 'ERROR' for row in records),
               'missing_results': missing, 'attempts': records}
    _ = (production/'processing_summary_v4.json').write_text(json.dumps(summary, indent=2)+'\n')
    print(json.dumps({key: value for key, value in summary.items() if key != 'attempts'}))
    return 1 if summary['error_count'] or summary['current_identity_error_count'] else 0


if __name__ == '__main__':
    raise SystemExit(main())
