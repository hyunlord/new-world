# /// script
# requires-python = ">=3.10"
# dependencies = ["Pillow", "numpy"]
# ///
# How to run: python3 -m unittest discover -s tools/art_pipeline -p test_regions_v2.py
"""Pose-independent anatomical colour tables frozen from reviewed references."""
from __future__ import annotations

from dataclasses import dataclass
from typing import Final, TypedDict

import numpy as np
from PIL import Image

from palette_tools import Color, rgb_to_lab

HAIR_HEIGHT: Final = {'adult_a': 0.37, 'adult_b': 0.37, 'child': 0.4}


class ColorEntryData(TypedDict):
    rgb: list[int]
    region: int
    counts: list[int]


class TableData(TypedDict):
    body: str
    colors: list[ColorEntryData]


@dataclass(frozen=True, slots=True)
class PaletteRegion:
    """One reference colour, its chosen region, and complete ambiguity evidence."""
    rgb: Color
    region: int
    counts: tuple[int, ...]


@dataclass(frozen=True, slots=True)
class ColorRegionTable:
    """Per-body reference evidence; ordering is canonical RGB lexical order."""
    body: str
    colors: tuple[PaletteRegion, ...]


class RegionTableError(ValueError):
    """A reference or serialized table violates its anatomical contract."""


def build_table(reference: Image.Image, approved: Image.Image, body: str) -> ColorRegionTable:
    """Count opaque reference colours; ties choose the smallest region integer."""
    if body not in HAIR_HEIGHT or reference.size != approved.size:
        raise RegionTableError('Unknown body or reference/map size mismatch')
    rgba = np.asarray(reference.convert('RGBA'))
    indices = np.asarray(approved.convert('RGBA'))
    mask = rgba[..., 3] > 0
    if not np.array_equal(mask, indices[..., 3] > 0):
        raise RegionTableError('Reference and approved map alpha differ')
    if np.any((indices[..., 0][mask] < 1) | (indices[..., 0][mask] > 5)):
        raise RegionTableError('Opaque region must be in 1..5')
    entries = []
    for color in np.unique(rgba[mask, :3], axis=0):
        selected = mask & np.all(rgba[..., :3] == color, axis=-1)
        counts = np.bincount(indices[..., 0][selected], minlength=6)
        entries.append(PaletteRegion(tuple(int(v) for v in color), int(np.argmax(counts)),
                                     tuple(int(v) for v in counts)))
    if not entries:
        raise RegionTableError('Reference contains no opaque colours')
    return ColorRegionTable(body, tuple(entries))


def table_to_data(table: ColorRegionTable) -> TableData:
    """Return JSON-ready evidence, including all non-winning region counts."""
    return {'body': table.body, 'colors': [
        {'rgb': list(entry.rgb), 'region': entry.region, 'counts': list(entry.counts)}
        for entry in table.colors]}


def table_from_data(data: TableData) -> ColorRegionTable:
    """Restore a canonical table and reject inconsistent majority evidence."""
    if data['body'] not in HAIR_HEIGHT or not data['colors']:
        raise RegionTableError('Unknown body or empty table')
    entries = []
    for row in data['colors']:
        rgb, counts, region = row['rgb'], row['counts'], row['region']
        if len(rgb) != 3 or any(type(v) is not int or not 0 <= v <= 255 for v in rgb):
            raise RegionTableError('Invalid RGB')
        if len(counts) != 6 or any(type(v) is not int or v < 0 for v in counts):
            raise RegionTableError('Invalid region counts')
        if counts[0] != 0 or sum(counts) == 0 or region != counts.index(max(counts)):
            raise RegionTableError('Region disagrees with reference majority')
        entries.append(PaletteRegion((rgb[0], rgb[1], rgb[2]), region, tuple(counts)))
    entries.sort(key=lambda entry: entry.rgb)
    if len({entry.rgb for entry in entries}) != len(entries):
        raise RegionTableError('Duplicate table colour')
    return ColorRegionTable(data['body'], tuple(entries))


def classify(image: Image.Image, table: ColorRegionTable, *, standing: bool = False) -> Image.Image:
    """Encode region/shade with exact lookup, then inclusive DeltaE76 <= 8."""
    rgba = np.asarray(image.convert('RGBA'))
    alpha = rgba[..., 3] > 0
    result = np.zeros_like(rgba)
    result[..., 3] = np.where(alpha, 255, 0)
    entries = sorted(table.colors, key=lambda entry: entry.rgb)
    labs = rgb_to_lab(np.asarray([entry.rgb for entry in entries]))
    exact = {entry.rgb: entry.region for entry in entries}
    regions = np.where(alpha, 5, 0).astype(np.uint8)
    for value in np.unique(rgba[alpha, :3], axis=0):
        color = tuple(int(v) for v in value)
        region = exact.get(color)
        if region is None:
            distances = np.linalg.norm(labs - rgb_to_lab(value), axis=1)
            nearest = int(np.argmin(distances))
            region = entries[nearest].region if distances[nearest] <= 8 else 5
        regions[alpha & np.all(rgba[..., :3] == value, axis=-1)] = region
    luma = rgb_to_lab(rgba[..., :3])[..., 0]
    if standing:
        rows = (np.arange(image.height)[:, None] + 0.5) / image.height
        regions[(regions == 5) & (rows < HAIR_HEIGHT[table.body]) & (luma < 38)] = 3
    for region in (2, 3, 4):
        mask = regions == region
        if mask.any():
            unique = np.unique(luma[mask])
            rank = np.searchsorted(unique, luma[mask])
            result[..., 1][mask] = 2 - np.rint(rank / max(1, len(unique) - 1) * 2).astype(np.uint8)
    result[..., 0] = regions
    return Image.fromarray(result)
