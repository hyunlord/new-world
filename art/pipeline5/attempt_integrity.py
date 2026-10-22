# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "scipy", "scikit-image"]
# ///
# How to run: imported by python -m art.pipeline5.process_sheets
"""Bind numeric verdicts and cached artifacts to actual immutable inputs."""
import hashlib
import json
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Final

from art.pipeline5.whole_checks import FRAMES, check_whole_path

ROOT: Final = Path(__file__).resolve().parents[2]
CALIBRATION: Final = ROOT/'docs/v8/arttrial2/JUDGE_V2_CALIBRATION.json'
CALIBRATION_SHA256: Final = '7218ac77979065ec6398c77cb6156bce5bd523979ed48732945f2ab642a5d5fd'


def digest(path: Path) -> str:
    """Hash exact bytes, including raw output and provenance records."""
    return hashlib.sha256(path.read_bytes()).hexdigest()


@dataclass(frozen=True, slots=True)
class JudgeBinding:
    raw_sha256: str
    judge_sha256: str
    calibration_sha256: str
    numeric_pass: bool


def verify_judge(path: Path, raw: Path) -> JudgeBinding:
    """Recompute the numeric verdict because legacy JSON has no calibration field."""
    from art.pipeline2.judge import judge, load_calibration
    if digest(CALIBRATION) != CALIBRATION_SHA256:
        message = 'judge_identity: frozen calibration hash changed'
        raise ValueError(message)
    supplied = json.loads(path.read_text())
    if supplied.get('sha256') != digest(raw) or supplied.get('family') != 'person':
        message = 'judge_identity: raw SHA or person family mismatch'
        raise ValueError(message)
    expected = asdict(judge(raw, load_calibration(CALIBRATION), family='person'))
    for key in ('sha256', 'family', 'metrics', 'numeric_checks', 'numeric_pass'):
        if supplied.get(key) != expected[key]:
            message = f'judge_identity: recomputed frozen verdict differs at {key}'
            raise ValueError(message)
    return JudgeBinding(digest(raw), digest(path), CALIBRATION_SHA256, supplied['numeric_pass'])


def validate_bundle(directory: Path, identity_path: Path) -> None:
    """Validate all five files against diagnostic sidecars and fresh geometry."""
    summary = json.loads(identity_path.read_text())
    expected_files = {frame+suffix for frame in FRAMES for suffix in ('.png', '.json')}
    if {p.name for p in directory.iterdir()} != expected_files:
        message = 'delivery_identity: expected exactly five PNG/JSON pairs'
        raise ValueError(message)
    for frame in FRAMES:
        path = directory/(frame+'.png')
        sidecar = directory/(frame+'.json')
        data = json.loads(sidecar.read_text())
        original = identity_path.parent/(frame+'.json')
        if digest(sidecar) != digest(original) or data['image_sha256'] != digest(path):
            message = f'delivery_identity: hash mismatch for {frame}'
            raise ValueError(message)
        if data['raw_sha256'] != summary['raw_sha256'] or data['prompt_sha256'] != summary['prompt_sha256']:
            message = f'delivery_identity: source mismatch for {frame}'
            raise ValueError(message)
        if not check_whole_path(path, data['age'], frame).passed:
            message = f'delivery_identity: structure failure for {frame}'
            raise ValueError(message)


def audit_cached(record_path: Path, result_path: Path, output_root: Path) -> JudgeBinding:
    """Revalidate source, diagnostic and delivery identities without rewriting history."""
    record = json.loads(record_path.read_text())
    dispatch_path = result_path.with_name(result_path.name.removesuffix('_result.json')+'.json')
    dispatch = json.loads(dispatch_path.read_text())
    raw = Path(record['raw'])
    checks_path = Path(record['diagnostics'])/'sheet_checks.json'
    checks = json.loads(checks_path.read_text())
    binding = verify_judge(Path(record['judge']), raw)
    identities = {'result_sha256': digest(result_path), 'dispatch_sha256': digest(dispatch_path),
                  'raw_sha256': digest(raw), 'calibration_sha256': binding.calibration_sha256,
                  'prompt_sha256': hashlib.sha256(dispatch['prompt'].encode()).hexdigest()}
    for key, expected in identities.items():
        if record.get(key) != expected:
            message = f'cache_identity: {key} mismatch'
            raise ValueError(message)
    judge_path = Path(record['judge'])
    prompt_path = judge_path.with_name(judge_path.name.removesuffix('_judge.json')+'.txt')
    if digest(prompt_path) != identities['prompt_sha256']:
        message = 'cache_identity: saved prompt file differs'
        raise ValueError(message)
    if digest(Path(record['original'])) != binding.raw_sha256:
        message = 'cache_identity: original generated PNG differs'
        raise ValueError(message)
    if record['sheet_checks'] != checks:
        message = 'cache_identity: diagnostic JSON differs from cached record'
        raise ValueError(message)
    if record.get('panel_detection_version') != 3 or record.get('normalization_version') != 4:
        message = 'cache_identity: algorithm version mismatch'
        raise ValueError(message)
    for key in ('look', 'facing', 'attempt'):
        if record.get(key) != dispatch[key] or checks.get(key) != dispatch[key]:
            message = f'cache_identity: diagnostic {key} mismatch'
            raise ValueError(message)
    if record.get('age') != dispatch['age'] or checks.get('age', checks['look'].split('_')[0]) != dispatch['age']:
        message = 'cache_identity: diagnostic age mismatch'
        raise ValueError(message)
    if checks['raw_sha256'] != binding.raw_sha256 or checks['prompt_sha256'] != identities['prompt_sha256']:
        message = 'cache_identity: diagnostic input hashes mismatch'
        raise ValueError(message)
    if checks['judge'] != json.loads(Path(record['judge']).read_text()):
        message = 'cache_identity: diagnostic judge differs'
        raise ValueError(message)
    frames = checks['frames']
    if frames and (checks.get('panel_detection_version') != 3 or checks.get('normalization_version') != 4):
        message = 'cache_identity: diagnostic algorithm mismatch'
        raise ValueError(message)
    for frame in frames:
        if 'coverage' not in frame:
            continue
        name = frame['frame']
        image = checks_path.parent/(name+'.png')
        data = json.loads(image.with_suffix('.json').read_text())
        for key in ('look', 'age', 'facing', 'attempt'):
            if data.get(key) != dispatch[key]:
                message = f'cache_identity: frame {name} {key} mismatch'
                raise ValueError(message)
        result = asdict(check_whole_path(image, dispatch['age'], name))
        if (data['image_sha256'] != digest(image) or data['raw_sha256'] != binding.raw_sha256
                or data['prompt_sha256'] != identities['prompt_sha256'] or data['judge'] != checks['judge']
                or data['panel_detection_version'] != 3 or data['normalization_version'] != 4
                or json.loads(json.dumps(result)) != data['checks']):
            message = f'cache_identity: frame {name} hash, judge, algorithm or check mismatch'
            raise ValueError(message)
    accepted = bool(len(frames) == 5 and all(frame.get('passed') is True for frame in frames)
                    and binding.numeric_pass)
    if record['accepted'] != accepted or checks['passed'] != accepted:
        message = 'cache_identity: acceptance differs from bound verdict and complete frame set'
        raise ValueError(message)
    if accepted:
        validate_bundle(output_root/dispatch['look']/dispatch['facing'], checks_path)
    return binding
