# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run: python3 -m art.pipeline5.prod1_prompts EXTERNAL_ROOT
"""Preregister all 36 two-facing prompts before production generation."""
from __future__ import annotations

import hashlib
import json
import sys
from itertools import product
from pathlib import Path

REFERENCE = Path('art/character/candidates/I-1.png')


def prompt_for(age: str, sex: str, top: str, hair: str) -> str:
    """Make attribute cues explicit while retaining one shared sheet scale."""
    build = ('adult male, full beard, broad shoulders, flat chest' if sex == 'M'
             else 'adult female, visible waist and hips, no beard, opaque garment covering chest')
    if age == 'child':
        build = ('young boy' if sex == 'M' else 'young girl') + ', round face, no beard, large head about 32% of height, simple opaque tunic or chest wrap'
    garment = {
        'loincloth': 'opaque loincloth and bare upper torso for adult male; adult female wears opaque chest band plus skirt; child wears opaque chest band plus skirt',
        'hide_wrap': 'one-shoulder asymmetric hide wrap with one visible bare shoulder, chest securely covered',
        'fur_cloak': 'wide fur collar and hood shape, cloak hanging past hips, clearly darker than skin and distinct in tone and contour from hair, chest covered',
    }[top]
    hairstyle = {
        'short': 'hair cropped close to skull, clean rounded silhouette',
        'long_tied': 'high ponytail standing clearly away from head outline, distinct from cloak',
        'bun': 'high compact bun on TOP of head, clearly above the head outline, not a ponytail',
    }[hair]
    return (
        'Create one 1536x1024 truly transparent RGBA character sprite sheet. Exactly TEN separate whole figures in TWO rows of FIVE, equal cell spacing, generous transparent gaps. '
        'Reference I-1 is the visual style and character proportion reference: fine controlled brown-black ink, stronger outer contour, restrained warm earth watercolor washes, upper-left light, simple readable forms. '
        'No text, borders, ground plane, backdrop, cast shadow, detached objects, pixel art, 3D render, dense hatching or extra figures. '
        f'Subject in all ten cells: {build}; {garment}; {hairstyle}. '
        'Both rows depict exactly the SAME person, same garments and hair and build. Row 1 faces S(front three-quarter seen 20 degrees above); row 2 faces E(the SAME person turned 90 degrees to viewer right). '
        'Columns in EACH row:1 standing idle,2 mid-stride walking,3 two-hand stone axe raised in a swing,4 log bundle across shoulders,5 lying asleep on side HEAD LEFT. '
        'Axe head as big as person head; log bundle wider than shoulders. Props must touch hands/body, connected silhouette. '
        'Same scale for all ten poses and one foot groundline per row. Entire raised axe and logs fit within their cells. Sleep on the side with legs extended, not curled: target sleep width48–60px when standing body is56px adult or40px child, using the same scale throughout. '
        'Strong readable hair, clothing and sex/age silhouettes at 56px adult or 40px child standing height. Fully visible head and toes, no overlap between people.'
    )


def main(root: Path) -> None:
    """Write immutable planned prompts, distinct from actual generation calls."""
    root.mkdir(parents=True, exist_ok=True)
    reference_hash = hashlib.sha256(REFERENCE.read_bytes()).hexdigest()
    records: list[dict[str, str]] = []
    for age, sex, top, hair in product(('adult', 'child'), ('M', 'F'),
                                      ('loincloth', 'hide_wrap', 'fur_cloak'),
                                      ('short', 'long_tied', 'bun')):
        look = f'{age}_{sex}_{top}_{hair}'
        prompt = prompt_for(age, sex, top, hair)
        records.append({'look': look, 'age': age, 'sex': sex, 'top': top, 'hair': hair,
                        'prompt': prompt, 'prompt_sha256': hashlib.sha256(prompt.encode()).hexdigest(),
                        'reference': str(REFERENCE.resolve()), 'reference_sha256': reference_hash})
    path = root / 'planned_prompts.json'
    if path.exists():
        raise FileExistsError(path)
    _ = path.write_text(json.dumps(records, indent=2) + '\n')


if __name__ == '__main__':
    main(Path(sys.argv[1]))
