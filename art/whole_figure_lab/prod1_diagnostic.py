# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy"]
# ///
"""Supplemental rejected-set review, never a certified lab or formal R1 result."""
from __future__ import annotations

import json
import sys
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Final

from PIL import Image, ImageDraw

from art.pipeline3.capture_lab import gif_from_frames

from .build import sha
from .prod1 import FACINGS, stage
from .samples import FRAMES, PEOPLE, STATES, Person

STATUS: Final = 'DIAGNOSTIC_REJECTED_SET_NOT_CERTIFIED'


@dataclass(frozen=True, slots=True)
class Selection:
    """One fixed identity's best attempt; missing identities remain missing."""
    look: str
    directory: Path | None
    attempt: int | None
    passing_panels: int
    summary_json: str


def select(production: Path, person: Person) -> Selection:
    """Use the successful review replacement, else maximize panels with earliest ties."""
    choices = []
    review_replacements = []
    for path in sorted((production / person.look).glob('try*/sheet_checks.json')):
        raw = path.read_text()
        summary = json.loads(raw)
        if summary['look'] != person.look or type(summary['attempt']) is not int:
            raise ValueError(f'Invalid diagnostic attempt identity: {path}')
        if summary['attempt'] not in (1, 2, 3):
            raise ValueError(f'Attempt exceeds original ceiling: {path}')
        passing = summary.get('passing_panels', 0)
        if type(passing) is not int or passing not in range(11):
            raise ValueError(f'Invalid passing panel count: {path}')
        choice = Selection(person.look, path.parent, summary['attempt'], passing, raw)
        choices.append(choice)
        reason = summary.get('review_reason')
        if summary.get('passed') is True and isinstance(reason, str) and reason.strip():
            review_replacements.append(choice)
    if len(review_replacements) > 1:
        raise ValueError(f'Multiple successful review replacements: {person.look}')
    if review_replacements:
        return review_replacements[0]
    if not choices:
        return Selection(person.look, None, None, 0, '{}')
    return max(choices, key=lambda value: (value.passing_panels, -(value.attempt or 0)))


def missing(state: str) -> Image.Image:
    """Visible absence marker on an otherwise blank conforming canvas."""
    image = Image.new('RGBA', (112, 80) if state == 'sleep' else (80, 112))
    ImageDraw.Draw(image).text((8, 32), 'MISSING', fill='#18110a')
    return image


def load(selection: Selection, facing: str, state: str) -> tuple[Image.Image, dict[str, str | int | bool | None]]:
    """Read normalized pixels unchanged; preserve rejection JSON instead of rewriting it."""
    record: dict[str, str | int | bool | None] = {
        'dataset_status': STATUS, 'formal_R1_eligible': False, 'look': selection.look,
        'attempt': selection.attempt, 'facing': facing, 'frame': state,
        'source': None, 'source_sha256': None, 'sidecar_json_verbatim': None,
        'sidecar_sha256': None, 'display': 'MISSING', 'reason': 'No attempt available',
    }
    if selection.directory is None:
        return missing(state), record
    path = selection.directory / facing / f'{state}.png'
    sidecar = path.with_suffix('.json')
    record['source'] = str(path.resolve())
    if sidecar.exists():
        record['sidecar_json_verbatim'] = sidecar.read_text()
        record['sidecar_sha256'] = sha(sidecar)
    if not path.exists():
        record['reason'] = 'Normalized frame absent; raw/rejected-source images never substituted'
        return missing(state), record
    record['source_sha256'] = sha(path)
    if not sidecar.exists():
        record['reason'] = 'Normalized sidecar absent; source identity cannot be verified'
        return missing(state), record
    metadata = json.loads(sidecar.read_text())
    if ((metadata['look'], metadata['facing'], metadata['frame'], metadata['attempt'])
            != (selection.look, facing, state, selection.attempt)
            or metadata['image_sha256'] != sha(path)):
        raise ValueError(f'Diagnostic source identity/hash mismatch: {path}')
    with Image.open(path) as image:
        expected = (112, 80) if state == 'sleep' else (80, 112)
        if image.mode != 'RGBA' or image.size != expected:
            record['reason'] = 'Nonconforming canvas; no resize, crop, or raw fallback permitted'
            return missing(state), record
        record['display'] = 'NORMALIZED_REJECTED_SET_FRAME'
        record['reason'] = 'Diagnostic display only; original verdict retained in sidecar and sheet summary'
        return image.copy(), record


def panel(selection: Selection, state: str, bob: int = 0) -> Image.Image:
    """Keep the production stage geometry, plus a separate diagnostic label footer."""
    result = Image.new('RGB', (224, 248), '#e0bd8e')
    for column, facing in enumerate(FACINGS):
        sprite, _ = load(selection, facing, state)
        for row, name in enumerate(('meadow', 'forest_floor')):
            result.paste(stage(sprite, name, bob), (column * 112, row * 112))
    draw = ImageDraw.Draw(result)
    draw.text((2, 225), 'DIAGNOSTIC - NOT CERTIFIED', fill='#18110a')
    draw.text((2, 237), 'formal_R1_eligible=false', fill='#18110a')
    return result


def build(production: Path, output: Path) -> None:
    """Build only after generation finishes; caller owns that prerequisite."""
    if output.resolve().is_relative_to(production.resolve()):
        raise ValueError('Diagnostic output must not be inside source production attempts')
    if 'route_i' in output.resolve().parts:
        raise ValueError('Diagnostic evidence must never be installed into route_i')
    selections = [select(production, person) for person in PEOPLE]
    records = []
    for selection in selections:
        for facing in FACINGS:
            for state in STATES:
                _, record = load(selection, facing, state)
                records.append(record)
    for name in ('gifs', 'stills', 'boards'):
        (output / name).mkdir(parents=True, exist_ok=True)
    answers = []
    for number, (person, selection) in enumerate(zip(PEOPLE, selections, strict=True), 1):
        for zoom in (1, 2):
            frames = output / f'frames_{number:02d}_{zoom}x'
            frames.mkdir(exist_ok=True)
            for index, (state, bob) in enumerate(FRAMES[person.activity]):
                image = panel(selection, state, bob)
                image.resize((224 * zoom, 248 * zoom), Image.Resampling.NEAREST).save(frames / f'frame_{index:02d}.png')
            target = output / 'gifs' / f'p1_{number:02d}_{zoom}x.gif'
            gif_from_frames(frames, target)
            answers.append({'dataset_status': STATUS, 'formal_R1_eligible': False,
                'number': number, 'zoom': zoom, **asdict(person),
                'item': {'work': 'stone_axe', 'carry': 'log_bundle'}.get(person.activity, 'none'),
                'file': target.name, 'sha256': sha(target), 'truth_basis': 'Intended preregistered attributes, not proof of rendered visibility'})
        state, bob = FRAMES[person.activity][1]
        panel(selection, state, bob).save(output / 'stills' / f'p1_{number:02d}.png')
    for zoom in (1, 2):
        image = Image.new('RGB', (896 * zoom, 744 * zoom))
        for index, (person, selection) in enumerate(zip(PEOPLE, selections, strict=True)):
            state, bob = FRAMES[person.activity][1]
            tile = panel(selection, state, bob).resize((224 * zoom, 248 * zoom), Image.Resampling.NEAREST)
            image.paste(tile, (index % 4 * 224 * zoom, index // 4 * 248 * zoom))
        image.save(output / 'boards' / f'lab_p1_diagnostic_{zoom}x.png', optimize=True)
    (output / 'DATASET_STATUS.txt').write_text(STATUS + '\nformal_R1_eligible=false\n')
    (output / 'answer_key.private.json').write_text(json.dumps(answers, indent=2) + '\n')
    (output / 'provenance.json').write_text(json.dumps({'dataset_status': STATUS,
        'formal_R1_eligible': False, 'selection_rule': 'Fixed TRIAL5 PEOPLE; unique passed=true attempt with nonempty review_reason replaces prior best; otherwise max passing_panels, earliest tie; no frame mixing',
        'selections': [{'look': s.look, 'directory': str(s.directory) if s.directory else None,
            'attempt': s.attempt, 'passing_panels': s.passing_panels, 'sheet_summary_json_verbatim': s.summary_json,
            'sheet_summary_sha256': sha(s.directory / 'sheet_checks.json') if s.directory else None} for s in selections],
        'frames': records, 'canvas': [80, 112], 'sleep_canvas': [112, 80],
        'limitations': 'Rejected or absent set. Supplemental review only. Neither certified lab nor formal R1 evidence.'}, indent=2) + '\n')


if __name__ == '__main__':
    if len(sys.argv) != 3:
        raise SystemExit('Usage after generation completion: python3 -m art.whole_figure_lab.prod1_diagnostic PRODUCTION OUTPUT')
    build(Path(sys.argv[1]), Path(sys.argv[2]))
