"""Mechanical asset checks; visual style and semantic geometry remain human gates."""
from collections.abc import Sequence
from dataclasses import dataclass
from pathlib import Path
from typing import assert_never

import numpy as np
from numpy.typing import NDArray
from PIL import Image

from .color import rgb_to_lab, vector_length
from .models import RGB, Asset
from .outline import OUTLINE, silhouette_boundary


@dataclass(frozen=True, slots=True)
class CheckResult:
    name: str
    passed: bool
    errors: tuple[str, ...]
    visible_pixels: int
    palette_compliance: float
    component_sizes: tuple[int, ...]
    reference_delta_e: float | None

def component_sizes(mask: NDArray[np.bool_]) -> tuple[int, ...]:
    """Count eight-connected components; sizes cannot establish semantic validity."""
    remaining = {(y, x) for y in range(len(mask)) for x in range(len(mask[0, :])) if mask[y, x]}
    sizes: list[int] = []
    while remaining:
        seed = min(remaining)
        remaining.remove(seed)
        stack = [seed]
        count = 0
        while stack:
            y, x = stack.pop()
            count += 1
            for dy in (-1, 0, 1):
                for dx in (-1, 0, 1):
                    point = (y + dy, x + dx)
                    if point in remaining:
                        remaining.remove(point)
                        stack.append(point)
        sizes.append(count)
    return tuple(sorted(sizes, reverse=True))

def reference_distance(image: Image.Image, reference: Image.Image) -> float | None:
    """Symmetric occupancy-weighted nearest-colour Lab distance, not shape similarity."""
    first: NDArray[np.uint8] = np.array(image.convert('RGBA'))
    second: NDArray[np.uint8] = np.array(reference.convert('RGBA'))
    a, ca = np.unique(first[first[:, :, 3] > 0, :3], axis=0, return_counts=True)
    b, cb = np.unique(second[second[:, :, 3] > 0, :3], axis=0, return_counts=True)
    if len(a) == 0 or len(b) == 0:
        return None
    delta: NDArray[np.float64] = vector_length(rgb_to_lab(a)[:, None] - rgb_to_lab(b)[None], axis=2)
    first_min = np.empty(len(a), dtype=np.float64)
    second_min = np.empty(len(b), dtype=np.float64)
    _ = np.min(delta, axis=1, out=first_min)
    _ = np.min(delta, axis=0, out=second_min)
    return float((np.average(first_min, weights=ca) +
                  np.average(second_min, weights=cb)) / 2)

def check_asset(asset: Asset, colors: Sequence[RGB], reference: Image.Image | None = None) -> CheckResult:
    """Reject noise below declared component size; retain legitimate detached parts.

    Components are never removed or treated as proof of correctness. Layers may
    declare min_component_pixels=1 for reviewed single-pixel details. Ghosts
    require blue alpha-80 pixels; dashed geometry needs visual confirmation.
    """
    pixels: NDArray[np.uint8] = np.array(asset.image.convert('RGBA'))
    visible: NDArray[np.bool_] = pixels[:, :, 3] > 0
    count = int(np.count_nonzero(visible))
    errors: list[str] = []
    if asset.image.size != asset.box:
        errors.append('size differs from declared box')
    if count == 0:
        errors.append('empty foreground')
    membership: NDArray[np.bool_] = np.zeros(visible.shape, dtype=np.bool_)
    for color in colors:
        matches = np.equal(pixels[:, :, :3], color)
        color_matches = np.empty(visible.shape, dtype=np.bool_)
        _ = np.all(matches, axis=2, out=color_matches)
        membership |= color_matches
    compliant = int(np.count_nonzero(visible & membership))
    if compliant != count:
        errors.append('visible RGB outside master palette')
    if np.any(np.not_equal(pixels[~visible, :3], 0)):
        errors.append('hidden RGB in transparent pixels')
    match asset.kind:
        case 'object' | 'layer':
            silhouette: NDArray[np.bool_] = visible
            if asset.outline_context is not None:
                if asset.outline_context.size != asset.image.size:
                    errors.append('outline context size mismatch')
                else:
                    context: NDArray[np.uint8] = np.array(asset.outline_context.convert('RGBA'))
                    silhouette = visible | (context[:, :, 3] > 0)
            boundary: NDArray[np.bool_] = silhouette_boundary(silhouette) & visible
            if np.any(np.not_equal(pixels[boundary, :3], OUTLINE)):
                errors.append('silhouette boundary lacks dark outline')
            alpha = 255
        case 'ground':
            alpha = 255
        case 'ghost':
            alpha = 80
            if np.any(np.not_equal(pixels[visible, :3], (111, 176, 232))):
                errors.append('ghost must use blueprint blue')
        case 'shadow':
            alpha = 90
        case _:
            assert_never(asset.kind)
    if np.any(np.not_equal(pixels[visible, 3], alpha)):
        errors.append(f'unclean alpha: expected 0 or {alpha}')
    sizes = component_sizes(visible)
    if asset.min_component_pixels < 1:
        errors.append('component threshold must be positive')
    if any(size < asset.min_component_pixels for size in sizes):
        errors.append('isolated component below declared minimum; review source')
    distance = None if reference is None else reference_distance(asset.image, reference)
    return CheckResult(asset.name, not errors, tuple(errors), count,
                       compliant / count if count else 0.0, sizes, distance)

@dataclass(frozen=True, slots=True)
class CheckRecord:
    """One real registry entry; family/source make submitted scope auditable."""
    result: CheckResult
    family: str
    source: str

def write_checks(records: Sequence[CheckRecord], path: Path, *, final: bool = False) -> None:
    """Write submitted-asset evidence; family P90 distance is a flag, never a gate."""
    from .models import PipelineError
    if not records or len({entry.result.name for entry in records}) != len(records):
        raise PipelineError('Checks need nonempty, uniquely named registry records')
    if final and any(entry.result.reference_delta_e is None for entry in records):
        raise PipelineError('Final checks require measured references for every record, including first-family self references')
    families = sorted({entry.family for entry in records})
    thresholds: dict[str, float] = {}
    for family in families:
        scores = [entry.result.reference_delta_e for entry in records
                  if entry.family == family and entry.result.reference_delta_e is not None]
        if scores:
            thresholds[family] = float(np.percentile(scores, 90))
    lines = [
        '# ART-TRIAL mechanical checks', '',
        'Evidence mode: FINAL (all submitted references measured).' if final else
        'Evidence mode: PROVISIONAL — not final evidence; references may be missing.',
        f'Scope: {len(records)} submitted registry assets only; this is not a complete-set claim.',
        'PASS denotes the checks below, not visual acceptance, semantic correctness, or integration.', '',
        '| Asset | Family | Result | Visible px | Palette | Components | Reference ΔE | P90 flag | Errors | Source |',
        '|---|---|---|---:|---:|---|---:|---|---|---|',
    ]
    for entry in sorted(records, key=lambda entry: entry.result.name):
        result = entry.result
        score = result.reference_delta_e
        distance = 'NOT_MEASURED' if score is None else f'{score:.3f}'
        flag = 'NOT_MEASURED' if score is None else ('ABOVE_P90' if score > thresholds[entry.family] else 'within P90')
        fields = [result.name, entry.family, 'PASS' if result.passed else 'FAIL',
                  str(result.visible_pixels), f'{result.palette_compliance:.2%}',
                  str(result.component_sizes), distance, flag,
                  '; '.join(result.errors) or 'none', entry.source]
        lines.append('| ' + ' | '.join(value.replace('|', '\\|').replace('\n', ' ') for value in fields) + ' |')
    lines += ['', '## Class rules and limits', '',
              '- All visible pixels, including partial alpha, must use the master palette; transparent RGB must be zero.',
              '- Object: alpha 0/255 and dark inner silhouette boundary. Ground: alpha 0/255, no outline requirement.',
              '- Layer: alpha 0/255; outline at exposed air using supplied registered body context. Without context, all layer edges are checked.',
              '- Ghost: exact blueprint blue #6FB0E8 and alpha 0/80. Dashed geometry requires visual verification.',
              '- Shadow: palette RGB and alpha 0/90; silhouette shape requires visual verification.',
              '- Components below each asset\'s declared size threshold fail; detached components are retained. Component counts do not prove meaningful geometry.',
              '- Outline writing preserves the silhouette and does not thicken it. Pre-existing thick interior dark marks require visual review.',
              '- Reference score is symmetric frequency-weighted nearest-colour Lab distance, not shape/style proof. Same colour support may score zero despite differing proportions.',
              '- Family 90th percentiles are computed only from submitted measured references; ABOVE_P90 is descriptive and does not fail an asset.',
              '- Alignment drift, 3–4 material values, lighting, readability, tiling and generation provenance need separate evidence.', '']
    path.parent.mkdir(parents=True, exist_ok=True)
    _ = path.write_text('\n'.join(lines), encoding='utf-8')
