# /// script
# requires-python = ">=3.11"
# dependencies = ["numpy", "pillow", "pydantic"]
# ///
# How to run: python3 -m art.flora.summer
"""Registered whole-canopy Lab chroma correction, with immutable before images."""
from __future__ import annotations

import hashlib
from dataclasses import asdict
from pathlib import Path
from typing import ClassVar, Final

import numpy as np
from PIL import Image
from pydantic import BaseModel, ConfigDict

from art.flora.check_assets import SummerMetrics, summer_metrics
from art.flora.normalize import NormalizedRecord
from art.flora.resolve_flora import read_catalog
from art.flora.schema import SeasonTint
from art.flora.seasons import apply_tint, canopy_mask, read_palette
from art.materials.tone import rgb_to_lab

ROOT: Final = Path(__file__).resolve().parents[2]
RECORD: Final = ROOT / 'docs/v8/arttrees/SUMMER_TRANSFORMS.json'
FROZEN: Final = Path('/Users/rexxa/work/arttrees-data/pre_summer')


class SummerRecord(BaseModel):
    """Each output is bound to the exact immutable input and masked operation."""

    model_config: ClassVar[ConfigDict] = ConfigDict(frozen=True, extra='forbid')
    path: str
    before_sha256: str
    after_sha256: str
    preserved_before: str
    mask_sha256: str
    delta_lab: tuple[float, float, float]
    target_rgb: tuple[int, int, int]
    before: SummerMetrics
    after: SummerMetrics
    alpha_unchanged: bool
    unmasked_unchanged: bool


class SummerRecords(BaseModel):
    """Idempotent operation ledger; a normalized input may be re-exported safely."""

    model_config: ClassVar[ConfigDict] = ConfigDict(frozen=True, extra='forbid')
    method: str = 'Selected canopy only: retain L*, translate mean a*,b* to foliage_summer_mid. Identity when the original already passes. No resampling, alpha edit, pixel painting or geometry change.'
    assets: tuple[SummerRecord, ...] = ()
    superseded: tuple[SummerRecord, ...] = ()


class NormalizedInputs(BaseModel):
    """The normalizer ledger authorizes a new generated attempt, never arbitrary edits."""

    model_config: ClassVar[ConfigDict] = ConfigDict(frozen=True, extra="ignore")
    assets: tuple[NormalizedRecord, ...]


def center_chroma(source: Image.Image, mask: Image.Image, target: tuple[int, int, int]) -> tuple[Image.Image, tuple[float, float, float]]:
    """Translate only canopy Lab chroma to a fixed palette mean, retaining wash L*."""
    rgba = np.asarray(source.convert('RGBA'), dtype=np.uint8)
    selected = (np.asarray(mask.convert('L')) > 0) & (rgba[..., 3] >= 128)
    if not np.any(selected):
        return source.copy(), (0, 0, 0)
    lab = rgb_to_lab(rgba[selected, :3].astype(np.float64) / 255)
    target_lab = rgb_to_lab(np.array(target, dtype=np.float64) / 255)
    delta = (0.0, float(target_lab[1] - lab[:, 1].mean()), float(target_lab[2] - lab[:, 2].mean()))
    return apply_tint(source, mask, SeasonTint(delta_lab=delta)), delta


def process(path: Path, prior: SummerRecord | None) -> SummerRecord:
    """Refuse unrecorded rewrites; never stack a second correction on corrected RGB."""
    digest = hashlib.sha256(path.read_bytes()).hexdigest()
    if prior is not None:
        if digest == prior.after_sha256:
            return prior
        if digest != prior.before_sha256:
            message = f'Unrecognized canonical revision; preserve/review prior record before retry: {path}'
            raise ValueError(message)
    frozen = FROZEN / f'{path.stem}_{digest}.png'
    if frozen.exists() and hashlib.sha256(frozen.read_bytes()).hexdigest() != digest:
        message = f'Immutable pre-summer source was changed: {frozen}'
        raise ValueError(message)
    if not frozen.exists():
        _ = frozen.write_bytes(path.read_bytes())
    with Image.open(frozen) as opened:
        source = opened.convert('RGBA')
    mask = canopy_mask(source, ROOT)
    before = summer_metrics(source, mask, ROOT)
    target = read_palette(ROOT).foliage_summer_mid
    output, delta = (source.copy(), (0.0, 0.0, 0.0)) if before.passed else center_chroma(source, mask, target)
    original = np.asarray(source)
    transformed = np.asarray(output)
    unmasked = np.asarray(mask) == 0
    alpha_equal = bool(np.array_equal(original[..., 3], transformed[..., 3]))
    protected_equal = bool(np.array_equal(original[unmasked], transformed[unmasked]))
    if not alpha_equal or not protected_equal:
        message = f'Protected pixels changed during summer correction: {path}'
        raise ValueError(message)
    output.save(path, optimize=True)
    after = summer_metrics(output, canopy_mask(output, ROOT), ROOT)
    return SummerRecord(path=str(path.relative_to(ROOT)), before_sha256=digest,
                        after_sha256=hashlib.sha256(path.read_bytes()).hexdigest(),
                        preserved_before=str(frozen), mask_sha256=hashlib.sha256(mask.tobytes()).hexdigest(),
                        delta_lab=delta, target_rgb=target, before=before, after=after,
                        alpha_unchanged=alpha_equal, unmasked_unchanged=protected_equal)


def main() -> int:
    """Correct every currently available canonical and emit hashes/measurements."""
    prior = SummerRecords.model_validate_json(RECORD.read_bytes()) if RECORD.exists() else SummerRecords()
    previous = {row.path: row for row in prior.assets}
    superseded = list(prior.superseded)
    inputs = NormalizedInputs.model_validate_json((ROOT / "docs/v8/arttrees/GENERATION.json").read_bytes())
    authorized = {row.output: row.output_sha256 for row in inputs.assets}
    FROZEN.mkdir(parents=True, exist_ok=True)
    catalog = read_catalog(ROOT)
    records: list[SummerRecord] = []
    for species in catalog.species.species.values():
        if species.habit == 'shrub':
            continue
        for frames in species.sizes.values():
            for frame in frames.variants:
                path = ROOT / frame.path
                if path.exists():
                    previous_record = previous.get(frame.path)
                    current_hash = hashlib.sha256(path.read_bytes()).hexdigest()
                    if previous_record is not None and current_hash not in (previous_record.before_sha256, previous_record.after_sha256):
                        if authorized.get(frame.path) != current_hash:
                            message = f"Source revision absent from normalizer ledger: {frame.path}"
                            raise ValueError(message)
                        superseded.append(previous_record)
                        previous_record = None
                    records.append(process(path, previous_record))
    _ = RECORD.write_text(SummerRecords(assets=tuple(records), superseded=tuple(superseded)).model_dump_json(indent=2) + '\n')
    for record in records:
        print(record.path, asdict(record.before), '->', asdict(record.after))
    return 0 if all(record.after.passed for record in records) else 1


if __name__ == '__main__':
    raise SystemExit(main())
