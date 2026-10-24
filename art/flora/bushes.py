# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "pydantic", "scipy", "scikit-image"]
# ///
# How to run: python3 -m art.flora.bushes /Users/rexxa/work/arttrees-data/bushes_completed.json
"""Whole-object shrub export retaining the failed historical plan-view evidence."""
from __future__ import annotations

import json
import shutil
import sys
from dataclasses import asdict
from pathlib import Path
from typing import Final

import numpy as np
from PIL import Image
from pydantic import TypeAdapter

from art.flora.check_assets import check_frame, summer_metrics
from art.flora.normalize import (
    GenerationInput,
    ProductionSpec,
    apply_transform,
    digest,
    plan_transform,
)
from art.flora.schema import Frame
from art.flora.seasons import canopy_mask, read_palette
from art.flora.summer import center_chroma

ROOT: Final = Path(__file__).resolve().parents[2]
DATA: Final = Path('/Users/rexxa/work/arttrees-data')
SPEC: Final = ProductionSpec(canvas=(96, 96), anchor=(48, 80), height=32)


def build_bushes(completed: Path) -> None:
    """Export selected shrub requests with unchanged alpha and protected RGB."""
    requests = TypeAdapter(list[GenerationInput]).validate_json(completed.read_bytes())
    report_path = ROOT / 'docs/v8/arttrees/BUSH_RECHECK.json'
    report = json.loads(report_path.read_text())
    generated = report.get('regenerated', [])
    for request in requests:
        source_hash = digest(request.source)
        preserved = DATA / 'originals' / f'{request.id}_try{request.attempt}_{source_hash[:12]}.png'
        if not preserved.exists():
            shutil.copyfile(request.source, preserved)
        with Image.open(preserved) as opened:
            raw = opened.convert('RGBA')
        transform = plan_transform(raw, SPEC)
        initial = apply_transform(raw, transform)
        before_path = DATA / 'originals' / f'{request.id}_try{request.attempt}_normalized.png'
        initial.save(before_path, optimize=True)
        mask = canopy_mask(initial, ROOT)
        before = summer_metrics(initial, mask, ROOT)
        target = read_palette(ROOT).foliage_summer_mid
        output, delta = (initial.copy(), (0.0, 0.0, 0.0)) if before.passed else center_chroma(initial, mask, target)
        before_pixels, after_pixels = np.asarray(initial), np.asarray(output)
        protected = np.asarray(mask) == 0
        alpha_equal = bool(np.array_equal(before_pixels[..., 3], after_pixels[..., 3]))
        protected_equal = bool(np.array_equal(before_pixels[protected], after_pixels[protected]))
        if not alpha_equal or not protected_equal:
            message = 'Summer correction changed protected shrub pixels'
            raise ValueError(message)
        path = ROOT / f'art/flora/sprites/{request.id}.png'
        output.save(path, optimize=True)
        frame = Frame(path=str(path.relative_to(ROOT)), canopy_mask=f'art/flora/masks/{request.id}.png',
                      canvas=SPEC.canvas, anchor=SPEC.anchor)
        check = check_frame(frame, (24, 40), summer=True)
        record = {
            'id': request.id, 'attempt': request.attempt, 'source': str(request.source),
            'source_sha256': source_hash, 'preserved_original': str(preserved),
            'provenance': request.model_dump(mode='json'), 'transform': transform.model_dump(mode='json'),
            'before_tone_path': str(before_path), 'before_tone_sha256': digest(before_path),
            'summer_before': asdict(before), 'delta_lab': delta,
            'alpha_unchanged': alpha_equal, 'unmasked_unchanged': protected_equal,
            'checks': asdict(check), 'visual_verdict': 'PENDING',
            'semantic_note': 'bare means fruit-depleted but leafy, not winter branches',
        }
        if not any(row['source_sha256'] == source_hash for row in generated):
            generated.append(record)
        print(request.id, json.dumps(asdict(check)))
    report['regenerated'] = generated
    report['status'] = 'REGENERATED_CANDIDATES_OWNER_PENDING'
    report['production_geometry'] = SPEC.model_dump(mode='json')
    report['production_geometry_note'] = 'Registered before generation: shrub32px silhouette, numeric band24..40,96x96 canvas,anchor48,80. Whole-object uniform reduction only.'
    report_path.write_text(json.dumps(report, indent=2) + '\n')


if __name__ == '__main__':
    build_bushes(Path(sys.argv[1]))
