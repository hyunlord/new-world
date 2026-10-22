# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "scipy", "scikit-image"]
# ///
# How to run: python -m art.pipeline5.check_set ROUTE_ROOT OUTPUT_JSON
"""Independently audit complete paired-facing delivery and 48 distinct samples."""
from __future__ import annotations

import hashlib
import json
import re
import sys
import tempfile
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Final, NotRequired, TypedDict

from PIL import Image

from art.pipeline5.attempt_integrity import digest
from art.pipeline5.judge_panels import CALIBRATION_SHA256, SheetVerdict, judge_panels
from art.pipeline5.sheet_geometry import normalize_panel, split_panels
from art.pipeline5.whole_checks import FRAMES, check_whole_path

SEED: Final = 'ART-PROD1-check-set-48-v1'
LOOK: Final = re.compile(r'(adult|child)_[MF]_(loincloth|hide_wrap|fur_cloak)_(short|long_tied|bun)')
FACINGS: Final = ('S', 'E')


class NumericRecord(TypedDict):
    candidate: str
    sha256: str
    family: str
    metrics: dict[str, float]
    numeric_checks: dict[str, bool]
    numeric_pass: bool


class Metadata(TypedDict):
    look: str
    age: str
    facing: str
    frame: str
    attempt: int
    raw: str
    raw_sha256: str
    prompt_sha256: str
    image_sha256: str
    scale: float
    standing_scale: NotRequired[float]
    sleep_scale: NotRequired[float]
    panel_index: int
    rotation: int
    source_box: list[int]
    normalization: NotRequired[dict[str, int | float | list[int]]]
    calibration_sha256: str
    judge: NumericRecord
    checks: dict[str, str | bool | int | float | list[str]]


class Manifest(TypedDict):
    look: str
    age: str
    sex: str
    top: str
    hair: str
    attempt: int
    scale: float
    standing_scale: NotRequired[float]
    sleep_scale: NotRequired[float]
    raw: str
    raw_sha256: str
    prompt_sha256: str
    calibration_sha256: str
    files: dict[str, str]


@dataclass(frozen=True, slots=True)
class SetVerdict:
    """All accepted frames audited; samples name unique look/facing identities."""

    passed: bool
    accepted_looks: int
    audited_frames: int
    sampled: tuple[str, ...]
    findings: tuple[str, ...]
    seed: str = SEED
    sampling: str = 'SHA256(seed:identity) order, round-robin age/sex strata, without replacement'


def sample_identities(identities: tuple[str, ...]) -> tuple[str, ...]:
    """Select up to 48 unique identities evenly across existing age/sex strata."""
    strata: dict[str, list[str]] = {}
    for identity in sorted(set(identities), key=lambda name: hashlib.sha256((SEED + ':' + name).encode()).hexdigest()):
        strata.setdefault('_'.join(identity.split('_')[:2]), []).append(identity)
    selected: list[str] = []
    while len(selected) < 48 and any(strata.values()):
        for key in sorted(strata):
            if strata[key] and len(selected) < 48:
                selected.append(strata[key].pop(0))
    return tuple(selected)


def audit_frame(path: Path, expected: tuple[str, str, str, int], fresh: SheetVerdict) -> Metadata:
    """Bind image, structural checks and numeric verdict to freshly sliced source."""
    data: Metadata = json.loads(path.with_suffix('.json').read_text())
    look, _facing, frame, index = expected
    if (data['look'], data['facing'], data['frame'], data['panel_index']) != expected:
        message = 'frame identity mismatch'
        raise ValueError(message)
    if (data['age'] != look.split('_')[0] or data['image_sha256'] != digest(path)
            or data['calibration_sha256'] != CALIBRATION_SHA256
            or data['raw_sha256'] != fresh.source_sha256 or data['raw'] != fresh.source):
        message = 'frame image/source/calibration hash or age mismatch'
        raise ValueError(message)
    checks = check_whole_path(path, data['age'], frame)
    if not checks.passed or data['checks'] != json.loads(json.dumps(asdict(checks))):
        message = 'fresh structural checks fail or differ from recorded checks'
        raise ValueError(message)
    numeric = data['judge']
    actual = asdict(fresh.panels[index].verdict)
    if any(numeric[key] != actual[key] for key in ('sha256', 'family', 'metrics', 'numeric_checks', 'numeric_pass')):
        message = 'source-panel numeric verdict mismatch'
        raise ValueError(message)
    if not numeric['numeric_pass'] or not all(numeric['numeric_checks'].values()):
        message = 'source-panel numeric failure'
        raise ValueError(message)
    if digest(Path(numeric['candidate'])) != numeric['sha256']:
        message = 'saved source-panel candidate hash mismatch'
        raise ValueError(message)
    return data


def audit_manifest(directory: Path, first: Metadata) -> None:
    """Require exact manifest coverage and independently hashed image/JSON pairs."""
    path = directory / 'MANIFEST.json'
    if not path.is_file():
        message = 'missing required MANIFEST.json'
        raise ValueError(message)
    manifest: Manifest = json.loads(path.read_text())
    if ('standing_scale' in manifest) != ('standing_scale' in first):
        message = 'manifest and sidecar scale schema mismatch'
        raise ValueError(message)
    for key in ('look', 'age', 'attempt', 'scale', 'raw', 'raw_sha256', 'prompt_sha256', 'calibration_sha256'):
        if manifest[key] != first[key]:
            message = f'manifest identity mismatch: {key}'
            raise ValueError(message)
    if 'standing_scale' in first:
        if manifest.get('standing_scale') != first['standing_scale']:
            message = 'manifest standing scale mismatch'
            raise ValueError(message)
        for facing in FACINGS:
            sleep = json.loads((directory / facing / 'sleep.json').read_text())
            if json.loads(path.read_text()).get('sleep_scales', {}).get(facing) != sleep['sleep_scale']:
                message = 'manifest sleep scale mismatch'
                raise ValueError(message)
    parts = first['look'].split('_')
    hair = 'long_tied' if first['look'].endswith('_long_tied') else parts[-1]
    top = first['look'][len(parts[0]) + len(parts[1]) + 2:-(len(hair) + 1)]
    if (manifest['sex'], manifest['top'], manifest['hair']) != (parts[1], top, hair):
        message = 'manifest look attributes mismatch'
        raise ValueError(message)
    expected = {f'{facing}/{frame}{suffix}' for facing in FACINGS for frame in FRAMES for suffix in ('.png', '.json')}
    if set(manifest['files']) != expected:
        message = 'manifest file population mismatch'
        raise ValueError(message)
    if any(digest(directory / relative) != sha for relative, sha in manifest['files'].items()):
        message = 'manifest file hash mismatch'
        raise ValueError(message)


def audit_look(directory: Path, temporary: Path) -> int:
    """Audit all ten frames, common scale and one shared original generation."""
    first_path = directory / 'S/idle.json'
    if not first_path.is_file():
        message = 'missing S/idle.json'
        raise ValueError(message)
    first: Metadata = json.loads(first_path.read_text())
    raw = Path(first['raw'])
    with Image.open(raw) as image:
        if image.mode != 'RGBA' or image.size != (1536, 1024):
            message = 'source must be 1536x1024 RGBA'
            raise ValueError(message)
        panels = split_panels(image, rows=2)
    fresh = judge_panels(raw, panels.images, temporary / directory.name)
    if not fresh.numeric_pass:
        message = 'accepted sheet has failing source panel judge'
        raise ValueError(message)
    age = directory.name.split('_')[0]
    scale = (40 if age == 'child' else 56) / panels.idle_height
    if first['scale'] != scale or type(first['attempt']) is not int or first['attempt'] not in (1, 2, 3):
        message = 'incorrect S-idle-derived common scale or sheet attempt'
        raise ValueError(message)
    for facing_index, facing in enumerate(FACINGS):
        expected_files = {frame + suffix for frame in FRAMES for suffix in ('.png', '.json')}
        if not (directory / facing).is_dir() or {p.name for p in (directory / facing).iterdir()} != expected_files:
            message = f'missing or unexpected {facing} frame population'
            raise ValueError(message)
        for frame_index, frame in enumerate(FRAMES):
            data = audit_frame(directory / facing / (frame + '.png'),
                               (directory.name, facing, frame, facing_index * 5 + frame_index), fresh)
            index = facing_index * 5 + frame_index
            target = (40 if age == 'child' else 56) if 'standing_scale' in first else None
            normalized = normalize_panel(panels.images[index], scale, frame == 'sleep', sleep_length=target)
            if target is not None:
                sleep_normalized = normalize_panel(panels.images[facing_index * 5 + 4], scale, True, sleep_length=target)
                if (data.get('standing_scale') != scale
                        or data.get('sleep_scale') != sleep_normalized.transform.scale
                        or data['scale'] != normalized.transform.scale):
                    message = 'incorrect standing, sleep or actual frame scale'
                    raise ValueError(message)
            if target is not None and data.get('normalization') != json.loads(json.dumps(asdict(normalized.transform))):
                message = 'normalization transform differs from regenerated source'
                raise ValueError(message)
            with Image.open(directory / facing / (frame + '.png')) as saved:
                same_pixels = saved.tobytes() == normalized.image.tobytes()
            if (not same_pixels or data['rotation'] != normalized.transform.rotation_degrees
                    or data['source_box'] != list(panels.boxes[index])):
                message = 'delivery pixels, source box or rotation differ from raw source normalization'
                raise ValueError(message)
            shared = ('attempt', 'raw_sha256', 'prompt_sha256')
            if target is None:
                shared += ('scale',)
            if any(data[key] != first[key] for key in shared):
                message = 'ten frames do not share source try, prompt, raw hash and scale'
                raise ValueError(message)
    audit_manifest(directory, first)
    return 10


def check_set(root: Path) -> SetVerdict:
    """Fail closed on incomplete delivery, malformed metadata or absent population."""
    findings: list[str] = []
    identities: list[str] = []
    frames = 0
    directories = sorted(path for path in root.iterdir() if path.is_dir() and not path.name.startswith('.')) if root.is_dir() else []
    with tempfile.TemporaryDirectory(prefix='prod1-set-audit-') as temp:
        for directory in directories:
            if LOOK.fullmatch(directory.name) is None:
                findings.append(f'{directory.name}: invalid look identity')
                continue
            try:
                frames += audit_look(directory, Path(temp))
                identities.extend(f'{directory.name}/{facing}' for facing in FACINGS)
            except (OSError, ValueError, KeyError, TypeError, IndexError) as error:
                findings.append(f'{directory.name}: {error}')
    sampled = sample_identities(tuple(identities))
    if len(sampled) != 48:
        findings.append(f'need 48 distinct look/facing identities; found {len(sampled)}')
    return SetVerdict(not findings, frames // 10, frames, sampled, tuple(findings))


def main() -> int:
    """Write audit evidence even when the delivery is empty or incomplete."""
    if len(sys.argv) != 3:
        print('Usage: python -m art.pipeline5.check_set ROUTE_ROOT OUTPUT_JSON')
        return 2
    result = check_set(Path(sys.argv[1]))
    _ = Path(sys.argv[2]).write_text(json.dumps(asdict(result), indent=2) + '\n')
    print(json.dumps(asdict(result)))
    return 0 if result.passed else 1


if __name__ == '__main__':
    raise SystemExit(main())
