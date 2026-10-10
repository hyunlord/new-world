# /// script
# requires-python = ">=3.10"
# dependencies = ["Pillow", "numpy"]
# ///
# How to run: python3 tools/art_pipeline/salvage_review.py --candidates PATH
"""Render isolated visual-review evidence without numerical acceptance hints."""
from __future__ import annotations

import argparse
import hashlib
import json
from dataclasses import dataclass
from pathlib import Path
from typing import Final

import numpy as np
from PIL import Image, ImageDraw, ImageFont

from production_sheets import CellSpec, definitions
from fill_catalog import load_catalog, review_definitions
from regions import DISPLAY, classify

ROOT: Final = Path(__file__).resolve().parents[2]
BACKGROUND: Final = (38, 41, 45, 255)
TEXT: Final = (235, 235, 229, 255)


@dataclass(frozen=True, slots=True)
class ReviewCandidate:
    """Only visual provenance crosses the blind-review boundary."""
    identifier: str
    slot_id: str
    body: str
    raw: Path | None
    unfitted: Path | None
    native: Path | None
    sprite: Path | None
    priority: tuple[int, int, float] = (0, 0, 0.0)
    target_slot: str = ''


@dataclass(frozen=True, slots=True)
class ReviewInput:
    """Visual candidates and selected identities, excluding all guard results."""
    candidates: tuple[ReviewCandidate, ...]
    selected: tuple[tuple[str, str], ...]
    references: tuple[tuple[str, str], ...]
    source_digest: str = ''
    catalog: dict[str,list[CellSpec]] | None = None


class ReviewError(ValueError):
    """Malformed review input with a stable diagnostic code."""
    code = 'invalid_review_input'


def load_review(path: Path, catalog_path: Path | None = None) -> ReviewInput:
    """Project candidate JSON onto the fields permitted on review sheets."""
    encoded = path.read_bytes()
    data = json.loads(encoded)
    if data.get('schema_version') != 1:
        raise ReviewError('Expected candidate schema version 1')
    candidates = []
    for entry in data['candidates']:
        paths = entry['images']
        candidates.append(ReviewCandidate(str(entry['id']), str(entry['slot_id']),
            str(entry.get('body') or ''),
            *(Path(paths[key]) if paths.get(key) else None
              for key in ('raw', 'unfitted', 'native', 'sprite')),
            (len(entry.get('guards', {}).get('errors', [])),
             len(entry.get('guards', {}).get('warnings', [])),
             float(entry.get('metrics', {}).get('mean_delta_e', 999) or 0)),
            str(entry.get('target_slot',entry['slot_id']))))
    catalog=load_catalog(catalog_path) if catalog_path else data.get('review_catalog')
    return ReviewInput(tuple(candidates), tuple(data['selected'].items()),
                       tuple(data['references'].items()), hashlib.sha256(encoded).hexdigest(),
                       review_definitions(catalog) if catalog else None)


def read_image(path: Path | None) -> Image.Image | None:
    """Detach a decoded image from its file handle, or preserve missing evidence."""
    if path is None or not path.is_file():
        return None
    with Image.open(path) as image:
        return image.convert('RGBA')


def enlarge(image: Image.Image, scale: int) -> Image.Image:
    """Use exact integer nearest-neighbour enlargement for every fitted view."""
    return image.resize((image.width * scale, image.height * scale), Image.Resampling.NEAREST)


def region_preview(image: Image.Image, body: str, *, view: str = '', activity: str = '') -> Image.Image:
    """Visualize the existing production classifier without writing its atlases."""
    indices = np.asarray(classify(image, body, view=view, activity=activity))
    rgba = np.empty_like(indices)
    rgba[..., :3] = DISPLAY[indices[..., 0]]
    rgba[..., 3] = indices[..., 3]
    return Image.fromarray(rgba)


def choose_candidate(review: ReviewInput, slot_id: str) -> ReviewCandidate | None:
    """Show selection first, otherwise preserve the first available visual evidence."""
    selected = dict(review.selected).get(slot_id)
    matching = sorted((candidate for candidate in review.candidates if candidate.slot_id == slot_id),
                      key=lambda candidate: (candidate.priority, candidate.identifier))
    return next((candidate for candidate in matching if candidate.identifier == selected),
                next((candidate for candidate in matching if candidate.sprite is not None),
                     matching[0] if matching else None))


def _font(size: int) -> ImageFont.FreeTypeFont:
    return ImageFont.truetype(str(ROOT / 'assets/fonts/galmuri/Galmuri11.ttf'), size)


def _place(canvas: Image.Image, image: Image.Image | None, box: tuple[int, int, int, int],
           scale: int, thumbnail: bool = False) -> None:
    draw = ImageDraw.Draw(canvas)
    x, y, width, height = box
    draw.rectangle((x, y, x + width - 1, y + height - 1), outline=(80, 84, 86))
    if image is None:
        draw.text((x + 8, y + 10), 'Unavailable', font=_font(12), fill=TEXT)
        return
    shown = enlarge(image, scale)
    if thumbnail:
        shown.thumbnail((width - 8, height - 8), Image.Resampling.NEAREST)
    if shown.width > width - 4 or shown.height > height - 4:
        raise ReviewError(f'Panel would clip {shown.size} in {(width, height)}')
    canvas.alpha_composite(shown, (x + (width - shown.width) // 2,
                                   y + (height - shown.height) // 2))


def render_sheet(review: ReviewInput, sheet: str, output: Path,
                 grass: tuple[int, int, int]) -> Path:
    """Render every requested slot, with literal pixel scales and identity evidence."""
    specs = (review.catalog or definitions())[sheet]
    by_id = {candidate.identifier: candidate for candidate in review.candidates}
    refs = {body: by_id[identifier] for body, identifier in review.references if identifier in by_id}
    rows = []
    for cell, spec in enumerate(specs, 1):
        slot = f'{sheet}_{cell:02d}'
        candidate = choose_candidate(review, slot)
        images = [read_image(getattr(candidate, field)) if candidate else None
                  for field in ('raw', 'unfitted', 'sprite')]
        reference = refs.get(candidate.body) if candidate else None
        ref_image = read_image(reference.sprite) if reference else None
        region = region_preview(images[2], candidate.body, view=spec.view, activity=spec.activity) if candidate and candidate.body and images[2] else None
        rows.append((slot, spec.name, candidate, images + [images[2], ref_image, region]))
    scales = (1, 3, 6, 2, 6, 6)
    widths = [136, 192, 252, 146, 192, 192]
    height = 304
    for _, _, _, images in rows:
        for index, image in enumerate(images):
            if image is not None and index:
                widths[index] = max(widths[index], image.width * scales[index] + 12)
                height = max(height, image.height * scales[index] + 12)
    card_width = sum(widths) + 14 * 2 + 6 * 5
    card_height = height + 74
    canvas = Image.new('RGBA', (card_width * 2, 62 + card_height * 8), BACKGROUND)
    draw = ImageDraw.Draw(canvas)
    draw.text((14, 10), f'{sheet} | raw and native detail / fitted sprite comparison', font=_font(18), fill=TEXT)
    legend = 'Regions: outline=black   skin=peach   hair=blue   clothing=green   other=magenta'
    draw.text((14, 37), legend, font=_font(13), fill=TEXT)
    labels = ('Raw cell (overview)', 'Unfitted native 3x', 'Fitted 6x', 'Fitted 2x / grass', 'Body reference 6x', 'Region map 6x')
    for index, (slot, name, candidate, images) in enumerate(rows):
        left, top = (index % 2) * card_width, 62 + (index // 2) * card_height
        draw.text((left + 14, top + 3), f'{slot} | {name}', font=_font(15), fill=TEXT)
        target=f' | Target: {candidate.target_slot}' if candidate and candidate.target_slot and candidate.target_slot!=slot else ''
        draw.text((left + 14, top + 24), f'Candidate: {candidate.identifier if candidate else "none"}{target}', font=_font(12), fill=TEXT)
        x = left + 14
        for panel, (width, label, image, scale) in enumerate(zip(widths, labels, images, scales)):
            draw.text((x, top + 44), label, font=_font(12), fill=TEXT)
            if panel == 3:
                draw.rectangle((x, top + 64, x + width - 1, top + 64 + height - 1), fill=(*grass, 255))
            _place(canvas, image, (x, top + 64, width, height), scale, thumbnail=panel == 0)
            x += width + 6
    output.parent.mkdir(parents=True, exist_ok=True)
    canvas.convert('RGB').save(output, optimize=True)
    return output


def render_all(review: ReviewInput, output_dir: Path) -> tuple[Path, ...]:
    """Write the sixteen blind sheets and exact 8x body identity PNGs."""
    palette = json.loads((ROOT / 'tools/art_pipeline/production_palette.json').read_text())
    rgb = palette['rgb_ramps']['ground']['grass'][0]
    grass = (int(rgb[0]), int(rgb[1]), int(rgb[2]))
    outputs = [render_sheet(review, sheet, output_dir / f'{sheet}.png', grass) for sheet in (review.catalog or definitions())]
    by_id = {candidate.identifier: candidate for candidate in review.candidates}
    for body, identifier in review.references:
        candidate = by_id.get(identifier)
        reference = read_image(candidate.sprite) if candidate else None
        if reference is None:
            raise ReviewError(f'Missing reference image for {body}: {identifier}')
        path = output_dir / f'reference_{body}_8x.png'
        enlarge(reference, 8).save(path)
        outputs.append(path)
    preview = {f'{sheet}_{cell:02d}': candidate.identifier if candidate else None
               for sheet, specs in (review.catalog or definitions()).items() for cell in range(1, len(specs) + 1)
               for candidate in (choose_candidate(review, f'{sheet}_{cell:02d}'),)}
    (output_dir / 'preview_selection.json').write_text(json.dumps({
        'rule': 'selected; otherwise converted with fewest errors, warnings, then DeltaE; otherwise raw',
        'candidates_sha256': review.source_digest,
        'images_sha256': {path.name: hashlib.sha256(path.read_bytes()).hexdigest() for path in outputs},
        'previews': preview}, indent=2) + '\n')
    return tuple(outputs)


def main() -> None:
    """Render review-only artifacts from converter evidence."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--candidates', type=Path, required=True)
    parser.add_argument('--output', type=Path, default=ROOT / 'docs/art/production2/review')
    parser.add_argument('--catalog', type=Path)
    args = parser.parse_args()
    paths = render_all(load_review(args.candidates,args.catalog), args.output)
    print(f'Rendered {len(paths)} PNG files; {sum(path.stat().st_size for path in paths)} bytes')


if __name__ == '__main__':
    main()
