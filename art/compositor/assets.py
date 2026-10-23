# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "pydantic"]
# ///
# How to run: imported by python3 -m art.compositor.render.
"""Artifact-backed asset lookup and adopted Route I identity selection."""
from __future__ import annotations

import hashlib
from dataclasses import dataclass, field
from functools import lru_cache
from pathlib import Path
from typing import Final

from PIL import Image, ImageChops, ImageStat

from art.compositor.models import (
    AssetEntry,
    AssetManifest,
    InputModel,
    LookManifest,
    Person,
)

ROOT: Final = Path(__file__).resolve().parents[1]
REPO: Final = ROOT.parent
ROUTE: Final = ROOT / "character/route_i"


class ExistingStatus(InputModel):
    status: str = "UNVERIFIED"
    anchor_point: tuple[float, float] | None = None


class MissingAssetError(Exception):
    def __init__(self, asset_id: str) -> None:
        self.asset_id: str = asset_id
        super().__init__(f"Required art asset is unavailable: {asset_id}")


@dataclass(frozen=True, slots=True)
class Catalog:
    entries: dict[str, AssetEntry]
    hashes: dict[str, str] = field(default_factory=dict)

    def asset(self, asset_id: str) -> AssetEntry:
        try:
            return self.entries[asset_id]
        except KeyError as error:
            raise MissingAssetError(asset_id) from error

    def image(self, asset_id: str) -> Image.Image:
        return read_image(REPO / self.asset(asset_id).path)

    def stack_asset(self, asset_id: str, count: int | None) -> str:
        entry = self.asset(asset_id)
        if count is None or not entry.count_buckets:
            return asset_id
        bucket = 0 if count == 1 else 1 if count <= 8 else 2
        return f"{asset_id}:count_bucket_{bucket}"


@lru_cache(maxsize=512)
def read_image(path: Path) -> Image.Image:
    with Image.open(path) as source:
        return source.convert("RGBA")


def load_catalog(manifests: tuple[Path, ...]) -> Catalog:
    entries: dict[str, AssetEntry] = {}
    for path in sorted((ROOT / "style05/env").glob("*/*.png")):
        sidecar = path.with_suffix(".json")
        metadata = ExistingStatus.model_validate_json(sidecar.read_text())
        image = read_image(path)
        entries[path.stem] = AssetEntry(id=path.stem, path=str(path.relative_to(REPO)),
                                       width=image.width, height=image.height,
                                       footprint=(image.width / 64, image.height / 64),
                                       anchor=metadata.anchor_point,
                                       family=path.parent.name, status=metadata.status)
    for path in manifests:
        manifest = AssetManifest.model_validate_json(path.read_text())
        for entry in manifest.assets:
            if not (REPO / entry.path).is_file():
                raise MissingAssetError(entry.id)
            entries[entry.id] = entry
            if entry.count_buckets and len(entry.count_buckets) != 3:
                raise ValueError(f"Expected three count buckets for {entry.id}")
            for index, bucket_path in enumerate(entry.count_buckets):
                image = read_image(REPO / bucket_path)
                bucket_id = f"{entry.id}:count_bucket_{index}"
                entries[bucket_id] = entry.model_copy(update={"id": bucket_id, "path": bucket_path,
                                                             "width": image.width, "height": image.height, "count_buckets": ()})
    for species in ("wolf", "deer"):
        animal_manifest = ROOT / "animal" / species / "MANIFEST.json"
        animal_status = ExistingStatus.model_validate_json(animal_manifest.read_text()).status
        for path in sorted((ROOT / "animal" / species).glob("*/*.png")):
            image = read_image(path)
            asset_id = f"{species}_{path.parent.name}_{path.stem}"
            entries[asset_id] = AssetEntry(id=asset_id, path=str(path.relative_to(REPO)),
                                         width=image.width, height=image.height,
                                         anchor=(48, 56 if path.stem == "dead" else 72),
                                         family="animal", status=animal_status)
    hashes = {entry.path: hashlib.sha256((REPO / entry.path).read_bytes()).hexdigest() for entry in entries.values()}
    hashes.update({str(path.relative_to(REPO)): hashlib.sha256(path.read_bytes()).hexdigest() for path in manifests})
    for species in ("wolf", "deer"):
        path = ROOT / "animal" / species / "MANIFEST.json"
        hashes[str(path.relative_to(REPO))] = hashlib.sha256(path.read_bytes()).hexdigest()
    return Catalog(entries, hashes)


@lru_cache(maxsize=1)
def adopted_looks() -> tuple[str, ...]:
    manifest = LookManifest.model_validate_json((ROUTE / "MANIFEST.json").read_text())
    return tuple(sorted(row.look for row in manifest.accepted_looks))


def choose_look(person: Person, top: str | None = None) -> str:
    if person.female is None:
        raise ValueError(f"Native sex metadata is unavailable for agent {person.id}")
    age = "child" if person.life_stage == "child" else "adult"
    sex = "F" if person.female else "M"
    candidates = [look for look in adopted_looks() if look.startswith(f"{age}_{sex}_")]
    if top is not None:
        candidates = [look for look in candidates if f"_{top}_" in look]
    if not candidates:
        message = f"No adopted look for real age/sex/top: {age}/{sex}/{top}"
        raise ValueError(message)
    return candidates[person.id % len(candidates)]


def frame_path(look: str, frame: str) -> Path:
    return ROUTE / look / "S" / f"{frame}.png"


@lru_cache(maxsize=36)
def top_colour(look: str) -> tuple[int, int, int]:
    """Median actual torso RGB; sample location follows the visible body bbox."""
    image = read_image(frame_path(look, "idle"))
    opaque = image.getchannel("A").point([255 if value >= 128 else 0 for value in range(256)])
    bbox = opaque.getbbox()
    if bbox is None:
        message = f"Empty adopted idle frame: {look}"
        raise ValueError(message)
    left, upper, right_edge, lower_edge = bbox
    right, lower = right_edge - 1, lower_edge - 1
    box = (round(left + (right - left) * .25), round(upper + (lower - upper) * .45),
           round(left + (right - left) * .75), round(upper + (lower - upper) * .70))
    cropped = image.crop(box).convert("RGB")
    mask = opaque.crop(box)
    red, green, blue = cropped.split()
    value = ImageChops.lighter(ImageChops.lighter(red, green), blue)
    brightness = value.point([255 if 50 <= number <= 225 else 0 for number in range(256)])
    selected = ImageChops.multiply(mask, brightness)
    if selected.getbbox() is None:
        selected = mask
    colour = ImageStat.Stat(cropped, selected).median
    return (round(colour[0]), round(colour[1]), round(colour[2]))
