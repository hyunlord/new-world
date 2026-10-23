"""Export optional item candidates without changing the main generation ledger.

Run with the same ephemeral dependencies as pipeline_struct.normalize.
Owner item height bands are unspecified: sizes below are provisional exports.
"""
from __future__ import annotations

import json
from dataclasses import asdict

import numpy as np
from PIL import Image
from pydantic import TypeAdapter

from art.compositor.models import AssetEntry, AssetManifest
from art.pipeline2.judge import judge, load_calibration
from art.pipeline_struct.normalize import (
    CALIBRATION,
    DATA,
    REPO,
    Generation,
    digest,
    normalize_sprite,
)

REJECTED = {"spear": "Brown halo outside spear.", "wolf_hide": "Brown halo outside folded pelt.", "hide": "Brown halo outside folded hide.", "berry_basket": "Brown halo outside basket."}


def tile(source: Image.Image, height: int) -> Image.Image:
    """Uniform fit with an opaque base at y=43; never repaint source pixels."""
    alpha: np.ndarray[tuple[int, ...], np.dtype[np.uint8]] = np.asarray(source.convert("RGBA"))[:, :, 3]
    ys, xs = np.nonzero(alpha >= 128)
    if not ys.size:
        raise ValueError("Missing opaque item")
    fit_height = max(1, min(height, int(54 * (int(ys.max()) - int(ys.min()) + 1) / (int(xs.max()) - int(xs.min()) + 1))))
    normalized = normalize_sprite(source, fit_height)
    result = Image.new("RGBA", (64, 64))
    result.alpha_composite(normalized.image, (32 - normalized.anchor[0], 43 - normalized.anchor[1]))
    return result


def count_tile(source: Image.Image, count: int) -> Image.Image:
    """Declare counts by compositing identical uniform-scale generated cutouts."""
    if count == 1:
        return source.copy()
    if count not in (3, 6):
        raise ValueError("Count composition supports 1, 3, or 6")
    bounds = source.getbbox()
    if bounds is None:
        raise ValueError("Empty item")
    cut = source.crop(bounds)
    scale = 0.65 if count == 3 else 0.5
    cut = cut.resize((max(1, round(cut.width * scale)), max(1, round(cut.height * scale))), Image.Resampling.LANCZOS)
    offsets = ((-8, -5), (8, -3), (0, 0)) if count == 3 else ((-10, -8), (0, -8), (10, -8), (-10, -4), (10, -4), (0, 0))
    result = Image.new("RGBA", (64, 64))
    opaque_rows: np.ndarray[tuple[int, ...], np.dtype[np.intp]] = np.nonzero(np.asarray(cut)[:, :, 3] >= 128)[0]
    base = int(opaque_rows.max())
    for dx, dy in offsets:
        result.alpha_composite(cut, (32 - cut.width // 2 + dx, 43 - base + dy))
    return result


def main() -> None:
    ledger = DATA / "items_ledger.json"
    rows = TypeAdapter(list[Generation]).validate_json(ledger.read_text())
    output = REPO / "art/style05/struct/items"
    output.mkdir(parents=True, exist_ok=True)
    calibration = load_calibration(CALIBRATION)
    entries: list[AssetEntry] = []
    readings: list[dict[str, object]] = []
    for row in rows:
        raw = DATA / f"{row.asset}_try{row.attempt}.png"
        if digest(raw) != digest(row.source):
            raise ValueError(f"Raw identity mismatch: {row.asset}")
        verdict = judge(raw, calibration, family="items")
        readable = row.asset not in REJECTED
        reading = REJECTED.get(row.asset, "Front thickness and compressed upper surface visible; clean isolated cutout.")
        if row.asset == "log_pile_large":
            reading += " Generated support posts are an extra semantic detail."
        record: dict[str, object] = {"asset": row.asset, "attempt": row.attempt, "source": str(row.source), "raw_sha256": digest(raw), "readable": readable, "reading": reading, "owner_projection_acceptance": "N/A: no explicit item height band; flat tools have projected footprint height", "judge_v2_raw": asdict(verdict), "status": "candidate" if readable and verdict.numeric_pass else "rejected_visual" if not readable else "rejected_numeric", "operations": ["alpha bounds crop plus fringe", "uniform LANCZOS fit max54px width and provisional height", "transparent padding to64x64 with base43", "count variants1/3/6 same-source copies at scales1/.65/.5"]}
        if readable and verdict.numeric_pass:
            with Image.open(raw) as image:
                normalized = tile(image, row.height or 16)
            path = output / f"{row.asset}.png"
            normalized.save(path)
            buckets: list[str] = []
            for label, count in (("1", 1), ("2_8", 3), ("9_plus", 6)):
                bucket = output / f"{row.asset}_count_{label}.png"
                count_tile(normalized, count).save(bucket)
                buckets.append(str(bucket.relative_to(REPO)))
            entries.append(AssetEntry(id=row.asset, path=str(path.relative_to(REPO)), width=64, height=64, anchor=(32, 43), family="item", status="candidate", count_buckets=tuple(buckets)))
            record["export_sha256"] = digest(path)
        readings.append(record)
    docs = REPO / "docs/v8/artstruct"
    _ = (docs / "item_assets.json").write_text(AssetManifest(assets=tuple(entries)).model_dump_json(indent=2) + "\n")
    _ = (docs / "newitem_readings.json").write_text(json.dumps({"ledger_sha256": digest(ledger), "acceptance": "Candidate only; numeric and executor reading are not owner approval", "attempts": readings}, indent=2) + "\n")
    print(f"{len(rows)} attempts; {len(entries)} candidates; {len(rows) - len(entries)} excluded attempts")


if __name__ == "__main__":
    main()
