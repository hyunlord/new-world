# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "pydantic", "typer"]
# ///
# How to run: python3 -m art.compositor.render --help.
"""Render the full real-scene/declared-overlay rehearsal matrix and provenance."""
from __future__ import annotations

import hashlib
import json
from dataclasses import dataclass
from pathlib import Path
from typing import Final

import typer
from PIL import Image, ImageDraw, ImageFont
from pydantic import JsonValue

from art.compositor.assets import REPO, Catalog, load_catalog
from art.compositor.drawing import ForegroundPainter
from art.compositor.geometry import HUD_HEIGHT, WINDOW, Viewport
from art.compositor.ground import GroundPainter
from art.compositor.measurements import ground_evidence
from art.compositor.models import Overlay, Scene
from art.compositor.native_layers import DESTINATION, bridge_sprites, verified_layer
from art.compositor.placements import (
    RenderPerson,
    Sprite,
    overlay_sprites,
    real_sprites,
    render_people,
)
from art.compositor.tone import Condition, NightLight, night_value

FILE_CAP: Final = 1_500_000
SCENE_DIR: Final = REPO / "art/compositor/scenes"
DOCS: Final = REPO / "docs/v8/artenv1"
EXTERNAL: Final = Path("/Users/rexxa/work/artenv1-data/compositor")
FONT: Final = ImageFont.truetype("/System/Library/Fonts/Supplemental/Arial.ttf", 13)


@dataclass(frozen=True, slots=True)
class RenderInputs:
    scene: Scene
    overlay: Overlay
    catalog: Catalog
    hud_path: Path


@dataclass(frozen=True, slots=True)
class RenderSettings:
    zoom: float
    condition: Condition
    lod: bool

    @property
    def suffix(self) -> str:
        zoom = {1.0: "1x", .5: "half", .25: "quarter"}[self.zoom]
        return f"{self.condition.value}_{zoom}_{'lod' if self.lod and self.zoom <= .5 else 'figures'}"


@dataclass(frozen=True, slots=True)
class RenderProduct:
    image: Image.Image
    evidence: dict[str, JsonValue]


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def compose(inputs: RenderInputs, settings: RenderSettings) -> RenderProduct:
    scene = inputs.scene
    viewport = Viewport(scene.camp, settings.zoom)
    native = verified_layer(scene)
    real = tuple(row for row in real_sprites(scene) if row.asset in {"oak_large", "stump", "hut"}) + bridge_sprites(native, inputs.catalog)
    declared = overlay_sprites(scene, inputs.overlay)
    sprites = real + declared
    floors = tuple(row for row in sprites if row.category == "floor" and not row.site)
    ground = GroundPainter(inputs.catalog, viewport, settings.condition)
    image = ground.paint(scene, floors)
    hearths = tuple(viewport.project(row.position) for row in sprites if row.asset in {"hearth_1", "hearth_pots"})
    ground_lit = night_value(image, NightLight(hearths, viewport.tile)) if settings.condition is Condition.NIGHT else image
    measured_ground = ground_evidence(ground_lit, scene, viewport, ground.limit)
    if not measured_ground["registered_pass"]:
        raise ValueError("Final illuminated ground violates registered tile limits")
    coverage = Image.new("L", WINDOW)
    painter = ForegroundPainter(image, viewport, inputs.catalog, settings.lod, coverage)
    object_rows: list[dict[str, JsonValue]] = []
    for zone in (row for row in sprites if row.zone):
        drawn = painter.sprite(zone)
        if drawn is not None:
            object_rows.append(drawn)
    people_rows: list[dict[str, JsonValue]] = []
    actors = render_people(scene, inputs.overlay)
    labels = {actor.person.id: index + 1 for index, actor in enumerate(actors)}
    layers: list[Sprite | RenderPerson] = [row for row in sprites if (row.category != "floor" or row.site) and not row.zone]
    layers.extend(actors)
    for row in sorted(layers, key=lambda layer: (layer.position[1], layer.position[0], isinstance(layer, RenderPerson))):
        if isinstance(row, Sprite):
            drawn = painter.sprite(row)
            if drawn is not None:
                object_rows.append(drawn)
        else:
            drawn = painter.person(row, labels[row.person.id])
            if drawn is not None:
                people_rows.append(drawn)
    coverage.paste(0, (0, 0, WINDOW[0], HUD_HEIGHT))
    coverage.paste(0, (0, WINDOW[1] - 23, WINDOW[0], WINDOW[1]))
    counts = coverage.histogram()
    for row in people_rows:
        person_id = row["id"]
        if not isinstance(person_id, int):
            raise TypeError("Person evidence lost its integer identity")
        row["visible_opaque_pixels"] = counts[labels[person_id]]
        row["pixel_visible"] = counts[labels[person_id]] > 0
        observed = next(person for person in native.people if person.agent_id == person_id)
        row["source_routine_activity_kind"] = observed.activity_kind
        row["source_job_kind"] = observed.job
    if settings.condition is Condition.NIGHT:
        image = night_value(image, NightLight(hearths, viewport.tile))
    with Image.open(inputs.hud_path) as hud:
        if hud.size != (WINDOW[0], HUD_HEIGHT):
            message = f"Authentic HUD must be 1600x112, received {hud.size}: {inputs.hud_path}"
            raise ValueError(message)
        image.paste(hud.convert("RGB"), (0, 0))
    draw = ImageDraw.Draw(image)
    legend = ("REHEARSAL | REAL: terrain, IDs, captured layers | DECLARED: west house/store, dug+ford+clay, fixtures+work context,"
              " 2 wolves+3 deer, 7 apparel/actions | Candidate art; no owner approval")
    draw.rectangle((0, WINDOW[1] - 23, WINDOW[0], WINDOW[1]), fill=(25, 28, 26, 245))
    draw.text((10, WINDOW[1] - 19), legend, font=FONT, fill=(211, 211, 197))
    store = next(row for row in inputs.overlay.structures if row.id == "declared_storehouse")
    sx, sy = viewport.project((scene.camp[0] + store.offset[0], scene.camp[1] + store.offset[1]))
    evidence: dict[str, JsonValue] = {
        "scene_id": scene.scene_id, "seed": scene.seed, "tick": scene.tick,
        "readability_repair_round": 1,
        "source_display_day": scene.display_day, "source_clock_minute": scene.clock_minute,
        "source_season": scene.season, "render_condition": settings.condition.value,
        "zoom": settings.zoom, "lod": settings.lod, "window": list(WINDOW),
        "hud_path": str(inputs.hud_path.relative_to(REPO)), "hud_sha256": sha256(inputs.hud_path),
        "camp_screen": list(viewport.project(scene.camp)), "source_population": len(scene.people),
        "anchor_visible_population": len(people_rows),
        "visible_population": sum(bool(row["pixel_visible"]) for row in people_rows),
        "visible_carriers": sum(row["activity"] == "carry" and bool(row["pixel_visible"]) for row in people_rows),
        "people": [row for row in people_rows], "objects": [row for row in object_rows],
        "ground": {key: value for key, value in ground.measurements.items()},
        "hearth_screen": [list(point) for point in hearths],
        "storehouse_bbox": [sx - viewport.tile // 2, sy - viewport.tile // 2,
                            sx + (store.size[0] - .5) * viewport.tile,
                            sy + (store.size[1] - .5) * viewport.tile],
        "provenance": scene.provenance, "source_limitations": list(scene.limitations),
        "asset_input_sha256": {key: value for key, value in inputs.catalog.hashes.items()},
        "route_i_manifest_sha256": sha256(REPO / "art/character/route_i/MANIFEST.json"),
        "palette_sha256": sha256(REPO / "art/compositor/top_colours.json"),
        "overlay_sha256": sha256(SCENE_DIR / "m5_overlay.json"),
        "native_extension_path": str(DESTINATION.relative_to(REPO)), "native_extension_sha256": sha256(DESTINATION),
        "lighting": "night value 0.55 + linear hearth restore within 5 tiles; foreground hue preserved",
        "ground_final_pixels": measured_ground,
        "visibility_rule": "Anchor within map plus at least one retained alpha>=128 figure/marker pixel after depth, HUD and footer; marks/glyphs/shadows excluded",
        "depth_rule": "Sort all objects and people together by world y, then x; people last only on exact same point",
    }
    return RenderProduct(image.convert("RGB"), evidence)


def save_product(product: RenderProduct, path: Path) -> dict[str, JsonValue]:
    path.parent.mkdir(parents=True, exist_ok=True)
    product.image.save(path, optimize=True, compress_level=9)
    size = path.stat().st_size
    if size > FILE_CAP:
        message = f"Lossless render exceeds committed file cap: {path}: {size}"
        raise ValueError(message)
    EXTERNAL.mkdir(parents=True, exist_ok=True)
    evidence_path = EXTERNAL / f"{path.stem}.json"
    _ = evidence_path.write_text(json.dumps(product.evidence, indent=2) + "\n")
    return {"path": str(path.relative_to(REPO)), "sha256": sha256(path), "bytes": size,
            "evidence": str(evidence_path), "evidence_sha256": sha256(evidence_path),
            "scene_id": product.evidence["scene_id"], "condition": product.evidence["render_condition"],
            "zoom": product.evidence["zoom"], "lod": product.evidence["lod"],
            "visible_population": product.evidence["visible_population"],
            "visible_carriers": product.evidence["visible_carriers"]}


def main(
    scene_name: str = "all",
    condition_name: str = "all",
    zoom: float = 0,
    hud_directory: Path = DOCS / "reference",
    output: Path = DOCS / "renders",
) -> None:
    catalog = load_catalog((DOCS / "built_assets.json", DOCS / "items_manifest.json"))
    overlay = Overlay.read(SCENE_DIR / "m5_overlay.json")
    scenes = sorted(SCENE_DIR.glob("scene_*.json")) if scene_name == "all" else [SCENE_DIR / f"scene_{scene_name}.json"]
    conditions = tuple(Condition) if condition_name == "all" else (Condition(condition_name),)
    zooms = (1.0, .5, .25) if zoom == 0 else (zoom,)
    results: list[dict[str, JsonValue]] = []
    for path in scenes:
        scene = Scene.read(path)
        for condition in conditions:
            for scale in zooms:
                hud_zoom = "1x" if scale == 1 else "half"
                hud_path = hud_directory / f"current_{scene.scene_id.removeprefix('scene_')}_{hud_zoom}_hud.png"
                inputs = RenderInputs(scene, overlay, catalog, hud_path)
                for lod in ((True, False) if scale == .5 else (True,)):
                    settings = RenderSettings(scale, condition, lod)
                    product = compose(inputs, settings)
                    target = output / f"{scene.scene_id}_{settings.suffix}.png"
                    row = save_product(product, target)
                    results.append(row)
                    print(f"RENDER {target.name} {row['bytes']} bytes people={row['visible_population']}", flush=True)
    changed = [path for path, digest in catalog.hashes.items() if sha256(REPO / path) != digest]
    if changed:
        raise ValueError(f"Asset inputs changed during matrix rendering; rerun after freeze: {changed}")
    _ = (DOCS / "render_manifest.json").write_text(json.dumps({"schema_version": 1, "renders": results}, indent=2) + "\n")


if __name__ == "__main__":
    typer.run(main)
