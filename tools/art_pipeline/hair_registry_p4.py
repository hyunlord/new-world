"""Resolve exact-pixel frozen hair maps or classify newly installed pixels."""
from __future__ import annotations

from dataclasses import dataclass
import hashlib
import json
from pathlib import Path

import numpy as np
from PIL import Image

from hair_maps import HairPalette, classify


class ReviewedMapError(ValueError):
    """Frozen hair evidence or its compatibility with source pixels is invalid."""


@dataclass(frozen=True, slots=True)
class HairFrame:
    """Image plus its anatomical interpretation for a single runtime frame."""

    body: str
    activity: str
    image: Image.Image


@dataclass(frozen=True, slots=True)
class ResolvedHair:
    """A runtime map and the frozen provenance used to select it."""

    image: Image.Image
    source: str
    path: str = ''
    sha256: str = ''
    verdict: str = 'pending'


@dataclass(frozen=True, slots=True)
class HairRegistry:
    """Loaded, tamper-checked masks indexed by body and exact source RGBA hash."""

    maps: dict[tuple[str, str, str], ResolvedHair]
    palettes: dict[str, HairPalette]

    @classmethod
    def load(cls, root: Path, registry_path: Path | None = None) -> HairRegistry:
        """Read frozen evidence once before any production writes can occur."""
        maps: dict[tuple[str, str, str], ResolvedHair] = {}
        registry = registry_path or root / 'tools/art_pipeline/reviewed_hair_maps_p4.json'
        if registry_path is not None or registry.exists():
            for record in json.loads(registry.read_text())['frames'].values():
                path = root / record['mask_path']
                if hashlib.sha256(path.read_bytes()).hexdigest() != record['mask_sha256']:
                    raise ReviewedMapError(f'Reviewed mask file hash mismatch: {path}')
                with Image.open(path) as opened:
                    image = opened.convert('RGBA')
                if list(image.size) != record['size']:
                    raise ReviewedMapError(f'Reviewed mask size mismatch: {path}')
                pixels = np.asarray(image)
                hair = pixels[..., 0] == 3
                if (not np.all(np.isin(pixels[..., 0], [0, 3])) or
                        np.any(pixels[..., 1][~hair]) or np.any(pixels[..., 1] > 2) or
                        np.any(pixels[..., 2]) or np.any(hair & (pixels[..., 3] == 0))):
                    raise ReviewedMapError(f'Reviewed mask is not hair-only: {path}')
                key = (record['body'], record['activity'], record['rgba_sha256'])
                resolved = ResolvedHair(image, record['source'], record['mask_path'],
                                        record['mask_sha256'], record['verdict'])
                previous = maps.get(key)
                if previous is not None and (previous.image.size != image.size or
                                             previous.image.tobytes() != image.tobytes()):
                    raise ReviewedMapError(f'Conflicting reviewed masks: {key}')
                maps[key] = previous if previous is not None and previous.verdict == 'reject' else resolved
        palettes: dict[str, HairPalette] = {}
        palette_path = root / 'tools/art_pipeline/hair_palette_p4.json'
        if palette_path.exists():
            for body, row in json.loads(palette_path.read_text())['bodies'].items():
                palettes[body] = HairPalette(body,
                    frozenset((int(r), int(g), int(b)) for r, g, b in row['seen_rgb']),
                    frozenset((int(r), int(g), int(b)) for r, g, b in row['hair_rgb']))
        return cls(maps, palettes)

    def resolve(self, frame: HairFrame) -> ResolvedHair:
        """Never reuse an ID's former mask after its source RGBA has changed."""
        rgba = frame.image.convert('RGBA')
        key = (frame.body, frame.activity, hashlib.sha256(rgba.tobytes()).hexdigest())
        frozen = self.maps.get(key)
        if frozen is not None:
            if frozen.verdict == 'reject':
                raise ReviewedMapError(f'Reviewed hair mask was rejected: {key}')
            if frozen.image.size != rgba.size or not np.array_equal(
                    np.asarray(frozen.image)[..., 3], np.asarray(rgba)[..., 3]):
                raise ReviewedMapError(f'Reviewed mask sprite size/alpha mismatch: {key}')
            result = frozen
        else:
            palette = self.palettes.get(frame.body, HairPalette(frame.body, frozenset(), frozenset()))
            result = ResolvedHair(classify(rgba, palette, activity=frame.activity), 'hair_maps')
        bbox = rgba.getchannel('A').getbbox()
        excluded = (frame.body == 'child' or frame.activity in ('asleep', 'lying') or
                    bbox is None or bbox[2] - bbox[0] > bbox[3] - bbox[1])
        count = int(np.count_nonzero(np.asarray(result.image)[..., 0] == 3))
        if excluded and count:
            # Identical pixels can be reused as an asleep/lying fallback: the pose
            # contract takes priority over the source frame's former hair mask.
            pixels = np.zeros_like(np.asarray(rgba))
            pixels[..., 3] = np.asarray(rgba)[..., 3]
            return ResolvedHair(Image.fromarray(pixels), 'excluded_pose')
        if frame.body.startswith('adult') and not excluded and count == 0:
            raise ReviewedMapError(f'Adult frame requires a hair mask: {key}')
        return result
