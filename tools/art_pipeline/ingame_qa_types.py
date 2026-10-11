"""Typed capture records and shared image geometry for real framebuffer QA."""
from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
import json
import numpy as np
from numpy.typing import NDArray
from PIL import Image

Array = NDArray[np.uint8]
FloatArray = NDArray[np.float64]
BoolArray = NDArray[np.bool_]

@dataclass(frozen=True, slots=True)
class Entity:
    kind: str
    ident: int
    rect: tuple[float, float, float, float]
    visible: bool
    sprite: str
    frame: str
    size: tuple[float, float]
    body: str
    variant: tuple[int, int]
    tile: tuple[int, int]
    activity: str
    facing: str
    role: str
    building_kind: int
    flip: bool
    texture: str
    region: tuple[int, int, int, int]
    name: str
    shown: bool = True
    alpha: float = 1.0
    world_anchor: tuple[float,float] = (0.0,0.0)
    draw_order: tuple[int,int] = (0,0)
    cue_rect: tuple[float,float,float,float] | None = None
    legacy_variant: tuple[int, int, int] | None = None

@dataclass(frozen=True, slots=True)
class Label:
    text: str
    font: str
    name: str
    rect: tuple[float, float, float, float]

@dataclass(frozen=True, slots=True)
class Capture:
    stem: str
    path: Path
    entities: tuple[Entity, ...]
    labels: tuple[Label, ...]
    zoom: float
    scale: float
    selected: int
    seek: int | None
    trails: int | None
    dots: int | None
    icons: int | None
    icon_ids: tuple[int, ...]
    need_ids: tuple[int, ...]
    timestamp: int

    def image(self, standard: bool = True) -> Array:
        with Image.open(self.path) as source:
            image = source.convert('RGB')
            if standard:
                image = image.resize((1920, 1080), Image.Resampling.NEAREST)
            return np.asarray(image)


def load_capture(path: Path) -> Capture:
    """Read the capture's original lossless framebuffer and entity sidecar."""
    data = json.loads(path.with_suffix('.entities.json').read_text())
    entities = []
    for row in data['entities']:
        variant = row.get('variant', {})
        entities.append(Entity(str(row['kind']), int(row.get('id', 0)), tuple(row['rect']),
            bool(row.get('visible', False)), str(row.get('sprite_id', '')), str(row.get('frame_id', '')),
            tuple(row.get('frame_size', [0, 0])), str(row.get('body', '')),
            (int(variant.get('tone', -1)), int(variant.get('hair', -1))),
            (int(row.get('tile_x', -999)), int(row.get('tile_y', -999))), str(row.get('activity', '')),
            str(row.get('facing', '')), str(row.get('calibration_kind', '')), int(row.get('building_kind', -1)),
            bool(row.get('flip_h', False)), str(row.get('texture_path', '')), tuple(int(x) for x in row.get('region', [0, 0, 0, 0])), str(row.get('name', '')), bool(row.get('shown', row.get('visible', False))),
            float(row.get('modulate_alpha',1))*float(row.get('self_modulate_alpha',1)),
            tuple(row.get('world_anchor',[0,0])),(int(row.get('z_index',0)),int(row.get('sibling_index',0))),tuple(row['cue_rect']) if 'cue_rect' in row else None,
            (int(variant['skin']), int(variant['hair']), int(variant['clothing']))
            if all(key in variant for key in ('skin', 'hair', 'clothing')) else None))
    snap = data.get('agent_snapshot', {})
    ids = snap.get('agent_ids', [])
    needs = tuple(int(agent) for i, agent in enumerate(ids)
                  if max(snap['hungers'][i], snap['thirsts'][i], snap['sleeps'][i]) >= 75
                  and any(e.kind == 'agent' and e.ident == int(agent) and e.shown for e in entities))
    labels = tuple(Label(str(r['text']), str(r['font']), str(r['name']), tuple(r['rect'])) for r in data.get('labels', []))
    return Capture(path.stem, path, tuple(entities), labels, float(data['zoom'][0]),
        float(data.get('framebuffer_scale', 1)), int(data.get('selected_entity_bits', -1)),
        data.get('seek_segments'), data.get('trail_segments'), data.get('head_dots'), data.get('need_icons'),
        tuple(data.get('need_icon_agent_ids') or []), needs, int(data.get('real_time_usec', 0)))


def box(entity: Entity, scale: float = 1.0) -> tuple[int, int, int, int]:
    x, y, w, h = (round(value / scale) for value in entity.rect)
    return x, y, x + w, y + h


def luma(rgb: Array) -> FloatArray:
    return np.asarray(rgb, dtype=np.float64) @ np.array([0.2126, 0.7152, 0.0722]) / 255.0


def sprite_pixels(entity: Entity, repo: Path, variants: bool = False) -> Array | None:
    path = repo / entity.texture.removeprefix('res://')
    if not entity.texture or not path.is_file():
        return None
    with Image.open(path) as source:
        image = source.convert('RGBA')
        x, y, w, h = entity.region
        if w > 0 and h > 0:
            image = image.crop((x, y, x + w, y + h))
        pixels = np.array(image)
    regions = path.with_name(path.stem + '_regions.png')
    ramps = repo / 'assets/art/villager_ramps.png'
    if variants and regions.is_file() and ramps.is_file() and min(entity.variant) >= 0:
        with Image.open(regions) as source:
            region = np.asarray(source.convert('RGBA').crop((x, y, x + w, y + h)))
        with Image.open(ramps) as source:
            palette = np.asarray(source.convert('RGB'))
        hair = region[..., 0] == 3
        natural = ~hair & (luma(pixels[..., :3]) >= .10)
        tone_code, hair_code = entity.variant
        tone = (.88, .96, 1.04, 1.12)[min(tone_code, 3)]
        pixels[natural, :3] = np.clip(
            np.floor(pixels[natural, :3].astype(np.float64) * tone + .5), 0, 255
        ).astype(np.uint8)
        shades = np.clip(region[..., 1], 0, 2)
        pixels[hair, :3] = palette[4 + min(hair_code, 6), shades[hair]]
    return pixels[:, ::-1].copy() if entity.flip else pixels


def placed(entity: Entity, capture: Capture, repo: Path, raw: bool = False, variants: bool = False) -> tuple[Array, BoolArray]:
    """Place source texels at their measured drawn rectangle, with viewport clipping."""
    scale = 1.0 if raw else capture.scale
    width, height = (round(1920 * capture.scale), round(1080 * capture.scale)) if raw else (1920, 1080)
    rgb = np.zeros((height, width, 3), dtype=np.uint8)
    mask = np.zeros((height, width), dtype=bool)
    pixels = sprite_pixels(entity, repo, variants)
    x0, y0, x1, y1 = box(entity, scale)
    if pixels is None or x1 <= x0 or y1 <= y0:
        return rgb, mask
    enlarged = np.asarray(Image.fromarray(pixels).resize((x1-x0, y1-y0), Image.Resampling.NEAREST))
    left, top, right, bottom = max(0, x0), max(0, y0), min(width, x1), min(height, y1)
    if right > left and bottom > top:
        clipped = enlarged[top-y0:bottom-y0, left-x0:right-x0]
        rgb[top:bottom, left:right] = clipped[..., :3]
        mask[top:bottom, left:right] = clipped[..., 3] > 0
    return rgb, mask
