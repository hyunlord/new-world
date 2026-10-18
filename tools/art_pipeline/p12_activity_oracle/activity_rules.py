"""Independent GOAL6.2 rules. Does not import or execute GDScript."""
from __future__ import annotations
import hashlib
import json
import math
from pathlib import Path
from dataclasses import dataclass

class EvidenceError(ValueError):
    """Incomplete or contradictory evidence cannot establish H7."""

@dataclass(frozen=True, slots=True)
class Frame:
    sprite_id: str
    body: str
    pose: str
    view: str
    page: str
    rect: tuple[int, ...]
    number: int
    basket: bool
    bush: bool
    cold_rejected: bool


def mix(value: int) -> int:
    value = (value ^ 0x9E3779B9) & 0xFFFFFFFF
    for _ in range(2):
        value = ((value ^ (value >> 16)) * 0x45D9F3B) & 0xFFFFFFFF
    return value ^ (value >> 16)


def body_for(agent: int, life_stage: int) -> str:
    return 'child' if life_stage == 1 else ('adult_b' if (mix(agent) >> 8) & 1 else 'adult_a')


def direction(delta: tuple[float, float], previous: str) -> str:
    x, y = delta
    if x*x + y*y < 0.000001:
        return previous
    if abs(x) >= abs(y):
        return 'W' if x < 0 else 'E'
    return 'N' if y < 0 else 'S'


def candidates(kind: int, body: str, facing: str, seconds: float, arrived: bool) -> list[tuple[str, str]]:
    if kind not in range(7) or body not in ('child', 'adult_a', 'adult_b') or facing not in 'EWNS':
        raise EvidenceError('invalid activity inputs')
    sit_view = 'S' if facing == 'N' and body != 'adult_b' else facing
    choices: list[tuple[str, str]] = []
    if not arrived:
        if kind == 5 and body == 'child' and facing in 'EW':
            choices.append(('run', facing))
        choices.append(('walk', facing))
    else:
        match kind:
            case 0:
                if body != 'child':
                    choices.append(('crouch', 'S'))
                choices.append(('sit', sit_view))
            case 1 | 3 | 6:
                choices.append(('sit', sit_view))
            case 2:
                choices.append(('build', 'S' if body == 'child' and facing == 'N' else facing))
            case 4:
                choices.append(('idle', facing))
            case 5:
                if body == 'child':
                    if math.floor(seconds * 2) % 2 and facing in 'EW':
                        choices.append(('run', facing))
                    choices.append(('hop', 'S'))
    return choices + [('idle', facing), ('idle', 'S')]


def daily(kind: int, body: str, facing: str, seconds: float, arrived: bool, atlas: tuple[Frame, ...]) -> tuple[str, str]:
    available = {(f.body, f.pose, f.view) for f in atlas}
    for pose, face in candidates(kind, body, facing, seconds, arrived):
        if (body, pose, 'E' if face == 'W' else face) in available:
            return pose, face
    raise EvidenceError('no explicit daily chain frame')


def final_frame(body: str, pose: str, facing: str, seconds: float, agent: int, atlas: tuple[Frame, ...]) -> Frame:
    view = 'E' if facing == 'W' else facing
    pairs = [(pose, view), (pose, 'S'), ('idle', view)]
    if pose == 'crossed':
        pairs = [('crossed', view), ('crossed', 'S'), ('sit', view), ('idle', view)]
    if pose == 'gather_wood':
        pairs = [(pose, d) for d in (view, 'S', 'N', 'E')] + [('crouch', d) for d in (view, 'S', 'E', 'N', 'W')] + [('idle', view), ('idle', 'S')]
    for activity, direction_view in pairs:
        frames = [f for f in atlas if (f.body, f.pose, f.view) == (body, activity, direction_view)
                  and not f.basket and not (pose == 'crossed' and f.cold_rejected)
                  and not (pose == 'gather_wood' and (f.bush or f.pose in ('gather', 'carry')))]
        if frames:
            frames.sort(key=lambda f: (f.number, f.sprite_id))
            fps = 8 if pose in ('walk', 'run', 'carry', 'carry_wood') else (1.5 if pose == 'idle' else 4)
            phase = ((mix(agent) >> 20) % 1000) / 1000
            return frames[math.floor((seconds + phase) * fps) % len(frames)]
    raise EvidenceError('no safe final frame')


def load_atlas(checkout: Path) -> tuple[Frame, ...]:
    # Pillow is already used by the repository art pipeline; no dependency installed here.
    from PIL import Image
    manifest = json.loads((checkout/'assets/art/atlas/world_atlas.json').read_text())
    audit = json.loads((checkout/'scripts/ui/cold_audit.json').read_text())
    result: list[Frame] = []
    for key, row in sorted(manifest['sprites'].items()):
        if key in manifest.get('suppressed', {}) or not row.get('drawable', True) or row.get('qa_status') not in ('accepted', 'derived', 'procedural', 'fallback') or row.get('body') not in ('adult_a', 'adult_b', 'child'):
            continue
        rejected = False
        if key in audit:
            x, y, w, h = row['rect']
            with Image.open(checkout/row['page'].removeprefix('res://')) as page:
                digest = hashlib.sha256(page.crop((x,y,x+w,y+h)).convert('RGBA').tobytes()).hexdigest()
            rejected = digest == audit[key]
        result.append(Frame(key, row['body'], row['activity'], row['view'], row['page'], tuple(row['rect']), int(row.get('frame',0)), bool(row.get('contains_basket',False)) or row.get('source_sprite') == 'direction_09_villager_gatherer', bool(row.get('contains_bush',False)), rejected))
    if not result:
        raise EvidenceError('empty atlas')
    return tuple(result)
