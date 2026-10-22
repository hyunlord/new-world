#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "scipy", "scikit-image"]
# ///
# How to run: PYTHONPATH=. uv run art/pipeline5/prod1_recount.py SOURCE_DIR NEW_RUN_DIR
"""Recount the frozen 72 TRIAL5 round-one sheets without promoting assets."""
from __future__ import annotations

import hashlib
import json
import sys
from collections import Counter
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Final

from PIL import Image

from art.pipeline5.judge_panels import CALIBRATION, judge_panels
from art.pipeline5.sheet_geometry import normalize_panel, split_panels
from art.pipeline5.whole_checks import FRAMES, check_whole

ROOT: Final = Path(__file__).resolve().parents[2]


@dataclass(frozen=True, slots=True)
class FrameRecord:
    """One requested frame, including failures before a canvas could exist."""

    frame: str
    structural_pass: bool
    findings: tuple[str, ...]
    rotation_degrees: int = 0
    normalized_sha256: str | None = None


@dataclass(frozen=True, slots=True)
class SheetRecord:
    """Keep structure and numeric judge verdicts independent."""

    identity: str
    source: str
    source_sha256: str
    sliced: bool
    structural_pass: bool
    panel_pass: bool
    passing_panels: int
    scale: float | None
    frames: tuple[FrameRecord, ...]
    error: str | None = None


@dataclass(frozen=True, slots=True)
class FileHash:
    path: str
    sha256: str


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def recount_sheet(source: Path, output: Path) -> SheetRecord:
    """Judge native panels and normalize all five, retaining failed evidence."""
    identity = source.stem.removesuffix('_try2')
    output.mkdir(parents=True, exist_ok=False)
    source_sha = digest(source)
    try:
        with Image.open(source) as raw:
            if raw.mode != 'RGBA' or raw.size != (1536, 1024):
                message = f'expected 1536x1024 RGBA, got {raw.size} {raw.mode}'
                raise ValueError(message)
            panels = split_panels(raw, rows=1)
    except (ValueError, OSError) as error:
        return SheetRecord(identity, str(source), source_sha, False, False, False,
                           0, None, (), str(error))
    _ = (output / 'slicing.json').write_text(json.dumps({
        'source_sha256': source_sha, 'boxes': panels.boxes,
        'split_columns': panels.split_columns, 'idle_height': panels.idle_height,
        'ownership': asdict(panels.ownership)}, indent=2) + '\n')
    judged = judge_panels(source, panels.images, output / 'judge', expected_count=5)
    age = identity.split('_', 1)[0]
    scale = (40 if age == 'child' else 56) / panels.idle_height
    frames: list[FrameRecord] = []
    for frame, panel in zip(FRAMES, panels.images, strict=True):
        try:
            normalized = normalize_panel(panel, scale, frame == 'sleep')
        except ValueError as error:
            panel.save(output / f'{frame}_normalization_failed_source.png')
            frames.append(FrameRecord(frame, False, (str(error),)))
            continue
        result = check_whole(normalized.image, age, frame)
        image_path = output / f'{frame}.png'
        normalized.image.save(image_path)
        _ = image_path.with_suffix('.json').write_text(json.dumps({
            'source_sha256': source_sha, 'scale': scale,
            'normalization': asdict(normalized.transform),
            'checks': asdict(result)}, indent=2) + '\n')
        frames.append(FrameRecord(frame, result.passed, result.findings,
                                  normalized.transform.rotation_degrees, digest(image_path)))
    return SheetRecord(identity, str(source), source_sha, True,
                       all(frame.structural_pass for frame in frames), judged.numeric_pass,
                       sum(panel.verdict.numeric_pass for panel in judged.panels),
                       scale, tuple(frames))


def run(sources: Path, output: Path) -> None:
    """Create one immutable run with exact input and instrument provenance."""
    sheets = sorted(sources.glob('*_try2.png'))
    identities = {path.stem.removesuffix('_try2') for path in sheets}
    if len(sheets) != 72 or len(identities) != 72:
        message = f'expected 72 distinct round-one try2 sources, got {len(sheets)}/{len(identities)}'
        raise ValueError(message)
    output.mkdir(parents=True, exist_ok=False)
    code_paths = sorted({CALIBRATION, *(ROOT / name for name in (
        'art/pipeline5/prod1_recount.py', 'art/pipeline5/judge_panels.py',
        'art/pipeline5/sheet_geometry.py', 'art/pipeline5/panel_ownership.py',
        'art/pipeline5/whole_checks.py', 'art/pipeline2/judge.py',
        'art/pipeline3/structure.py'))})
    code_hashes = tuple(FileHash(str(path.relative_to(ROOT)), digest(path)) for path in code_paths)
    for path in code_paths:
        snapshot = output / 'instruments' / path.relative_to(ROOT)
        snapshot.parent.mkdir(parents=True, exist_ok=True)
        _ = snapshot.write_bytes(path.read_bytes())
    records: list[SheetRecord] = []
    for index, source in enumerate(sheets):
        result = recount_sheet(source, output / source.stem)
        records.append(result)
        _ = (output / source.stem / 'result.json').write_text(json.dumps(asdict(result), indent=2) + '\n')
        print(f'{index + 1}/72 {result.identity}: structure={result.structural_pass} panels={result.passing_panels}/5', flush=True)
    changed = [record.path for record in code_hashes if digest(ROOT / record.path) != record.sha256]
    failure_counts = Counter(finding.split(':', 1)[0]
                             for record in records for frame in record.frames
                             for finding in frame.findings)
    summary = {
        'purpose': 'INFORMATION_ONLY; no TRIAL5 promotion; not PROD1 S1 acceptance',
        'cohort': 'TRIAL5 full production attempt2 cohort; distinct from actual reviewer selection',
        'source_directory': str(sources), 'run_directory': str(output),
        'denominator_look_facings': 72, 'denominator_frames': 360,
        'sliced_sheets': sum(record.sliced for record in records),
        'structural_all5_sheets': sum(record.structural_pass for record in records),
        'structural_all5_adult': sum(record.structural_pass and record.identity.startswith('adult_') for record in records),
        'structural_all5_child': sum(record.structural_pass and record.identity.startswith('child_') for record in records),
        'structural_frames_pass': sum(frame.structural_pass for record in records for frame in record.frames),
        'panel_all5_sheets': sum(record.panel_pass for record in records),
        'panel_frames_pass': sum(record.passing_panels for record in records),
        'combined_all5_sheets': sum(record.structural_pass and record.panel_pass for record in records),
        'rotations': dict(Counter(str(frame.rotation_degrees) for record in records for frame in record.frames if frame.normalized_sha256)),
        'failure_rule_counts': dict(sorted(failure_counts.items())),
        'failure_rule_counts_by_age': {age: dict(Counter(
            finding.split(':', 1)[0] for record in records
            if record.identity.startswith(age + '_')
            for frame in record.frames for finding in frame.findings))
            for age in ('adult', 'child')},
        'instrument_files_changed_during_run': changed,
        'instrument_hashes': [asdict(record) for record in code_hashes],
        'records': [asdict(record) for record in records],
    }
    _ = (output / 'summary.json').write_text(json.dumps(summary, indent=2) + '\n')
    if changed:
        message = 'Instrument code changed during run; results retained but must rerun: ' + ', '.join(changed)
        raise RuntimeError(message)
    print(json.dumps({key: value for key, value in summary.items() if key not in ('records', 'instrument_hashes')}))


if __name__ == '__main__':
    if len(sys.argv) != 3:
        raise SystemExit('usage: python -m art.pipeline5.prod1_recount SOURCE_DIR NEW_RUN_DIR')
    run(Path(sys.argv[1]).resolve(), Path(sys.argv[2]).resolve())
