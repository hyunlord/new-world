# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy"]
# ///
# How to run: imported by python -m art.pipeline3.check_layer
"""Sidecar trust boundary. All spatial declarations are integer game pixels."""
import json
from dataclasses import dataclass
from pathlib import Path
from typing import Literal
from .structure import Box, Point

Kind = Literal['body','top','hair','beard','headgear','item','carry','shadow']

@dataclass(frozen=True, slots=True)
class Layer:
    id: str
    image: Path
    mask: Path
    kind: str
    body_type: str
    direction: str
    frame: str
    scale: int
    operations: tuple[str,...]
    hands: tuple[Point,...] = ()
    grip: Point | None = None
    shoulder: Point | None = None
    extras: tuple[str,...] = ()
    variant: str = ''
    head_box: Box | None = None
    anatomical_mask: Path | None = None

def load_layer(path: Path) -> Layer:
    """Reject incomplete or unsupported declarations before inspecting pixels."""
    raw = json.loads(path.read_text())
    required = ('id','kind','body_type','direction','frame','scale','mask','operations')
    if raw.get('schema_version') != 1 or any(k not in raw for k in required):
        raise ValueError(f'{path}: missing required schema fields')
    if raw['scale'] not in (1,4) or raw['body_type'] not in ('adult','child','elder'):
        raise ValueError(f'{path}: invalid scale or body_type')
    if raw['kind'] not in ('body','top','hair','beard','headgear','item','carry','shadow'):
        raise ValueError(f'{path}: invalid kind')
    if raw['direction'] not in ('S','E','N') or raw['frame'] not in ('idle','walk_a','walk_b','work_a','work_b','sleep'):
        raise ValueError(f'{path}: invalid direction or frame')
    if not isinstance(raw['operations'],list) or not all(isinstance(v,str) for v in raw['operations']):
        raise ValueError(f'{path}: operations must be a string list')
    def point(value: list[int]) -> Point:
        if len(value) != 2 or not all(type(v) is int for v in value):
            raise ValueError(f'{path}: points must contain two integer game coordinates')
        return value[0],value[1]
    head_box = None
    if 'head_box' in raw:
        values = raw['head_box']
        if len(values) != 4 or not all(type(v) is int for v in values):
            raise ValueError(f'{path}: head_box requires four integer game coordinates')
        head_box = (values[0],values[1],values[2],values[3])
    return Layer(str(raw['id']),path.parent / raw.get('image',path.with_suffix('.png').name),
                 path.parent / raw['mask'],raw['kind'],raw['body_type'],raw['direction'],raw['frame'],
                 raw['scale'],tuple(raw['operations']),tuple(point(p) for p in raw.get('hands',[])),
                 point(raw['grip']) if 'grip' in raw else None,
                 point(raw['shoulder']) if 'shoulder' in raw else None,tuple(raw.get('extras',[])),str(raw.get('variant','')),head_box,
                 path.parent / raw['anatomical_mask'] if 'anatomical_mask' in raw else None)
